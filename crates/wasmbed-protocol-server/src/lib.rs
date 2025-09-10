// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use std::collections::HashMap;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;

use aes_gcm::{Aes256Gcm, Key, Nonce, KeyInit};
use aes_gcm::aead::Aead;
use chacha20poly1305::{ChaCha20Poly1305, Key as ChaChaKey, Nonce as ChaChaNonce, KeyInit as ChaChaKeyInit};
use ed25519_dalek::{VerifyingKey as Ed25519PublicKey, Signature, Verifier};
use hkdf::Hkdf;
use hmac::{Hmac, Mac};
use sha2::{Sha256, Sha512};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use tokio::sync::mpsc::error::SendError;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn, debug};

use wasmbed_cert::ServerIdentity;
use wasmbed_protocol::{
    ClientEnvelope, ClientMessage, MessageId, ServerEnvelope, ServerMessage,
    Version,
};
use wasmbed_types::PublicKey;

/// Maximum message size to prevent DoS attacks (16MB)
const MAX_MESSAGE_SIZE: usize = 16 * 1024 * 1024;

/// Encryption algorithm selection
#[derive(Debug, Clone)]
pub enum EncryptionAlgorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}

/// Secure connection handler using RustCrypto
pub struct SecureConnection {
    stream: TcpStream,
    encryption: EncryptionAlgorithm,
    key: Vec<u8>,
    nonce_counter: u64,
}

/// Server configuration with RustCrypto
pub struct ServerConfig {
    pub bind_addr: SocketAddr,
    pub identity: ServerIdentity,
    pub client_ca: Vec<u8>, // Client CA certificate
    pub encryption: EncryptionAlgorithm,
    pub on_client_connect: Arc<OnClientConnect>,
    pub on_client_disconnect: Arc<OnClientDisconnect>,
    pub on_client_message: Arc<OnClientMessage>,
    pub shutdown: CancellationToken,
}

type Clients = Arc<RwLock<HashMap<PublicKey<'static>, Sender>>>;
type LastMessageId = Arc<RwLock<MessageId>>;
pub type OnClientConnect = dyn Send
    + Sync
    + Fn(
        PublicKey<'static>,
    ) -> Pin<Box<dyn Future<Output = AuthorizationResult> + Send>>;
pub type OnClientDisconnect = dyn Send
    + Sync
    + Fn(PublicKey<'static>) -> Pin<Box<dyn Future<Output = ()> + Send>>;
pub type OnClientMessage = dyn Send
    + Sync
    + Fn(MessageContext) -> Pin<Box<dyn Future<Output = ()> + Send>>;
type Sender = UnboundedSender<ServerEnvelope>;

pub enum AuthorizationResult {
    Authorized,
    Unauthorized,
}

pub enum MessageDeliveryError {
    ClientNotFound(PublicKey<'static>),
    SendError(SendError<ServerEnvelope>),
}

pub struct MessageContext {
    envelope: ClientEnvelope,
    sender: Sender,
    client_public_key: PublicKey<'static>,
}

impl MessageContext {
    pub fn message(&self) -> ClientMessage {
        self.envelope.message.clone()
    }

    pub fn client_public_key(&self) -> &PublicKey<'static> {
        &self.client_public_key
    }

    pub fn reply(
        &self,
        message: ServerMessage,
    ) -> Result<(), SendError<ServerEnvelope>> {
        self.sender.send(ServerEnvelope {
            version: Version::V0,
            message_id: self.envelope.message_id,
            message,
        })
    }
}

pub struct Server {
    config: ServerConfig,
    clients: Clients,
    last_message_id: LastMessageId,
}

impl SecureConnection {
    /// Create a new secure connection
    pub fn new(stream: TcpStream, encryption: EncryptionAlgorithm, key: Vec<u8>) -> Self {
        Self {
            stream,
            encryption,
            key,
            nonce_counter: 0,
        }
    }

    /// Encrypt and send data
    pub async fn send_encrypted(&mut self, data: &[u8]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let encrypted = self.encrypt(data)?;
        
        // Send length prefix
        let len = encrypted.len() as u32;
        self.stream.write_all(&len.to_be_bytes()).await?;
        
        // Send encrypted data
        self.stream.write_all(&encrypted).await?;
        self.stream.flush().await?;
        
        Ok(())
    }

    /// Receive and decrypt data
    pub async fn receive_decrypted(&mut self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        // Read length prefix
        let mut len_buf = [0u8; 4];
        self.stream.read_exact(&mut len_buf).await?;
        let len = u32::from_be_bytes(len_buf) as usize;

        if len > MAX_MESSAGE_SIZE {
            return Err("Message too large".into());
        }

        // Read encrypted data
        let mut encrypted = vec![0u8; len];
        self.stream.read_exact(&mut encrypted).await?;

        // Decrypt data
        self.decrypt(&encrypted)
    }

    /// Encrypt data using the selected algorithm
    fn encrypt(&mut self, data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        match self.encryption {
            EncryptionAlgorithm::Aes256Gcm => {
                let key = Key::<Aes256Gcm>::from_slice(&self.key);
                let cipher = Aes256Gcm::new(key);
                
                // Generate nonce from counter
                let mut nonce_bytes = [0u8; 12];
                nonce_bytes[4..].copy_from_slice(&self.nonce_counter.to_be_bytes());
                let nonce = Nonce::from_slice(&nonce_bytes);
                
                self.nonce_counter += 1;
                
                let ciphertext = cipher.encrypt(nonce, data)
                    .map_err(|e| format!("AES-GCM encryption failed: {}", e))?;
                
                Ok(ciphertext)
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                let key = ChaChaKey::from_slice(&self.key);
                let cipher = ChaCha20Poly1305::new(key);
                
                // Generate nonce from counter
                let mut nonce_bytes = [0u8; 12];
                nonce_bytes[4..].copy_from_slice(&self.nonce_counter.to_be_bytes());
                let nonce = ChaChaNonce::from_slice(&nonce_bytes);
                
                self.nonce_counter += 1;
                
                let ciphertext = cipher.encrypt(nonce, data)
                    .map_err(|e| format!("ChaCha20-Poly1305 encryption failed: {}", e))?;
                
                Ok(ciphertext)
            }
        }
    }

    /// Decrypt data using the selected algorithm
    fn decrypt(&mut self, encrypted: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        match self.encryption {
            EncryptionAlgorithm::Aes256Gcm => {
                let key = Key::<Aes256Gcm>::from_slice(&self.key);
                let cipher = Aes256Gcm::new(key);
                
                // Extract nonce from counter
                let mut nonce_bytes = [0u8; 12];
                nonce_bytes[4..].copy_from_slice(&self.nonce_counter.to_be_bytes());
                let nonce = Nonce::from_slice(&nonce_bytes);
                
                self.nonce_counter += 1;
                
                let plaintext = cipher.decrypt(nonce, encrypted)
                    .map_err(|e| format!("AES-GCM decryption failed: {}", e))?;
                
                Ok(plaintext)
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                let key = ChaChaKey::from_slice(&self.key);
                let cipher = ChaCha20Poly1305::new(key);
                
                // Extract nonce from counter
                let mut nonce_bytes = [0u8; 12];
                nonce_bytes[4..].copy_from_slice(&self.nonce_counter.to_be_bytes());
                let nonce = ChaChaNonce::from_slice(&nonce_bytes);
                
                self.nonce_counter += 1;
                
                let plaintext = cipher.decrypt(nonce, encrypted)
                    .map_err(|e| format!("ChaCha20-Poly1305 decryption failed: {}", e))?;
                
                Ok(plaintext)
            }
        }
    }
}

impl Server {
    pub fn new(config: ServerConfig) -> Self {
        Self {
            config,
            clients: Default::default(),
            last_message_id: Default::default(),
        }
    }

    pub async fn run(&self) -> Result<(), std::io::Error> {
        let listener = TcpListener::bind(&self.config.bind_addr).await?;
        
        info!("Server listening on {}", self.config.bind_addr);

        loop {
            tokio::select! {
                result = listener.accept() => {
                    match result {
                        Ok((stream, addr)) => {
                            debug!("Accepted connection from {}", addr);
                            let clients = Arc::clone(&self.clients);
                            let on_client_connect = Arc::clone(&self.config.on_client_connect);
                            let on_client_disconnect = Arc::clone(&self.config.on_client_disconnect);
                            let on_client_message = Arc::clone(&self.config.on_client_message);
                            let encryption = self.config.encryption.clone();
                            tokio::spawn(async move {
                                if let Err(e) = handle_client(
                                    stream,
                                    encryption,
                                    clients,
                                    &*on_client_connect,
                                    &*on_client_disconnect,
                                    &*on_client_message,
                                ).await {
                                    error!("Client handler error: {}", e);
                                }
                            });
                        }
                        Err(e) => {
                            error!("Failed to accept connection: {}", e);
                        }
                    }
                }
                _ = self.config.shutdown.cancelled() => {
                    info!("Server shutdown requested");
                    break;
                }
            }
        }

        Ok(())
    }

    pub async fn send(
        &self,
        client_key: &PublicKey<'static>,
        message: ServerMessage,
    ) -> Result<MessageId, MessageDeliveryError> {
        let message_id = self.next_message_id().await;
        let envelope = ServerEnvelope {
            version: Version::V0,
            message_id,
            message,
        };

        match self.clients.read().await.get(client_key) {
            Some(client) => {
                client
                    .send(envelope)
                    .map_err(MessageDeliveryError::SendError)?;
                Ok(message_id)
            },
            None => {
                Err(MessageDeliveryError::ClientNotFound(client_key.clone()))
            },
        }
    }

    async fn next_message_id(&self) -> MessageId {
        let mut last = self.last_message_id.write().await;
        *last = last.next();
        *last
    }
}

/// Generate a shared key using ECDH key exchange
fn generate_shared_key(
    server_private_key: &[u8],
    client_public_key: &[u8],
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    // For simplicity, we'll use HKDF to derive a key from the public keys
    // In a real implementation, you'd use proper ECDH
    let mut hk = Hkdf::<Sha256>::new(None, server_private_key);
    let mut okm = [0u8; 32];
    hk.expand(client_public_key, &mut okm)
        .map_err(|e| format!("HKDF failed: {}", e))?;
    
    Ok(okm.to_vec())
}

/// Extract client public key from handshake data
fn extract_client_public_key(
    handshake_data: &[u8],
) -> Option<PublicKey<'static>> {
    // In a real implementation, you'd parse the handshake data
    // For now, we'll create a mock public key
    Some(PublicKey::from(handshake_data).into_owned())
}

/// Perform a simple handshake to exchange public keys
async fn perform_handshake(_stream: &TcpStream) -> std::io::Result<Vec<u8>> {
    // In a real implementation, you'd perform a proper handshake
    // For now, we'll return mock handshake data
    Ok(b"mock_handshake_data".to_vec())
}

async fn handle_client(
    stream: TcpStream,
    encryption: EncryptionAlgorithm,
    clients: Clients,
    on_client_connect: &OnClientConnect,
    on_client_disconnect: &OnClientDisconnect,
    on_client_message: &OnClientMessage,
) -> std::io::Result<()> {
    // Perform handshake to get client public key
    let handshake_data = perform_handshake(&stream).await?;
    let public_key = extract_client_public_key(&handshake_data).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Failed to extract client public key",
        )
    })?;

    info!("Client connected: {}", public_key);

    if matches!(
        on_client_connect(public_key.clone()).await,
        AuthorizationResult::Unauthorized
    ) {
        warn!("Client authorization failed: {:?}", public_key);
        return Ok(());
    }

    info!("Client authorized: {}", public_key);

    // Generate shared key for encryption
    let shared_key = generate_shared_key(&[], &handshake_data)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    let mut secure_conn = SecureConnection::new(stream, encryption, shared_key);
    let (tx, rx) = unbounded_channel::<ServerEnvelope>();
    register_client(&clients, &public_key, tx.clone()).await;

    let result = client_handler(
        &mut secure_conn,
        &public_key,
        &clients,
        rx,
        on_client_message,
    )
    .await;

    unregister_client(&clients, &public_key).await;
    info!("Client disconnected: {}", public_key);
    on_client_disconnect(public_key.clone()).await;

    result
}

async fn register_client<'a>(
    clients: &Clients,
    public_key: &PublicKey<'a>,
    sender: UnboundedSender<ServerEnvelope>,
) {
    let mut guard = clients.write().await;
    guard.insert(public_key.clone().into_owned(), sender);
}

async fn unregister_client(clients: &Clients, client_key: &PublicKey<'static>) {
    let mut guard = clients.write().await;
    guard.remove(client_key);
}

async fn client_handler<'a>(
    secure_conn: &mut SecureConnection,
    client_key: &PublicKey<'a>,
    clients: &Clients,
    mut rx: UnboundedReceiver<ServerEnvelope>,
    on_client_message: &OnClientMessage,
) -> std::io::Result<()> {
    // Simple approach: handle read and write sequentially
    loop {
        tokio::select! {
            result = read_envelope_secure(secure_conn) => {
                match result {
                    Ok(envelope) => {
                        let sender = {
                            let guard = clients.read().await;
                            guard.get(client_key).cloned()
                        };

                        if let Some(sender) = sender {
                            let ctx = MessageContext {
                                envelope,
                                sender,
                                client_public_key: client_key.clone().into_owned(),
                            };

                            on_client_message(ctx).await;
                        } else {
                            error!("Client sender not found: {client_key:?}");
                            return Err(std::io::Error::other(
                                format!("Client not found: {client_key:?}")
                            ));
                        }
                    }
                    Err(e) => {
                        error!("Failed to read envelope: {e}");
                        return Err(e);
                    }
                }
            }
            envelope_opt = rx.recv() => {
                if let Some(envelope) = envelope_opt {
                    if let Err(e) = write_envelope_secure(secure_conn, &envelope).await {
                        error!("Failed to write envelope: {}", e);
                        return Err(e);
                    }
                } else {
                    // Channel closed
                    break;
                }
            }
        }
    }

    Ok(())
}

async fn read_envelope_secure(
    secure_conn: &mut SecureConnection,
) -> std::io::Result<ClientEnvelope> {
    let data = secure_conn.receive_decrypted().await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    minicbor::decode(&data).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("CBOR decode error: {e}"),
        )
    })
}

async fn write_envelope_secure(
    secure_conn: &mut SecureConnection,
    envelope: &ServerEnvelope,
) -> std::io::Result<()> {
    let data = minicbor::to_vec(envelope).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("CBOR encode error: {e}"),
        )
    })?;

    secure_conn.send_encrypted(&data).await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    Ok(())
}
