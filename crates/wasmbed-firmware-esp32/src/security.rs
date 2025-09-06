// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use std::sync::Arc;
use std::time::SystemTime;

use anyhow::Result;
use log::{debug, info, warn};
use rustls::{ClientConfig, RootCertStore, ServerConfig};
use rustls_pemfile::certs;
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::{Aead, OsRng};
use rand::rngs::OsRng;
use base64::{Engine as _, engine::general_purpose};
use hex;

use wasmbed_types::PublicKey;

/// Security manager for ESP32 devices
pub struct SecurityManager {
    /// Private key for signing
    private_key: Option<SigningKey>,
    /// Public key for verification
    public_key: Option<VerifyingKey>,
    /// Random number generator
    rng: OsRng,
}

impl SecurityManager {
    /// Create a new security manager
    pub fn new() -> Result<Self> {
        let mut rng = OsRng;
        
        // Generate a new key pair
        let signing_key = SigningKey::generate(&mut rng);
        let verifying_key = signing_key.verifying_key();
        
        Ok(Self {
            private_key: Some(signing_key),
            public_key: Some(verifying_key),
            rng,
        })
    }

    /// Get the public key
    pub fn get_public_key(&self) -> Result<PublicKey> {
        if let Some(ref public_key) = self.public_key {
            let public_key_bytes = public_key.to_bytes();
            Ok(PublicKey::from(public_key_bytes))
        } else {
            Err(anyhow::anyhow!("No public key available"))
        }
    }

    /// Get the private key bytes
    pub fn get_private_key_bytes(&self) -> Option<Vec<u8>> {
        self.private_key.as_ref().map(|key| key.to_bytes().to_vec())
    }

    /// Get the public key bytes
    pub fn get_public_key_bytes(&self) -> Option<&[u8]> {
        self.public_key.as_ref().map(|key| key.to_bytes().as_slice())
    }

    /// Verify a certificate chain
    pub fn verify_certificate_chain(&self, _cert_chain: &[u8]) -> Result<bool> {
        // TODO: Implement certificate verification
        // For now, always return true
        Ok(true)
    }

    /// Sign data with the private key
    pub fn sign_data<T>(&self, data: &T) -> Result<Vec<u8>>
    where
        T: serde::Serialize,
    {
        let signing_key = self.private_key
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No private key available"))?;

        // Serialize the data
        let serialized = serde_json::to_vec(data)?;
        
        // Sign the data
        let signature = signing_key.sign(&serialized);
        
        Ok(signature.to_bytes().to_vec())
    }

    /// Verify a signature
    pub fn verify_signature<T>(&self, data: &T, signature: &[u8]) -> Result<bool>
    where
        T: serde::Serialize,
    {
        let verifying_key = self.public_key
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No public key available"))?;

        // Serialize the data
        let serialized = serde_json::to_vec(data)?;
        
        // Parse the signature
        let signature = Signature::from_bytes(signature.try_into()?);
        
        // Verify the signature
        match verifying_key.verify(&serialized, &signature) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Encrypt data with AES-256-GCM
    pub fn encrypt_data(&self, data: &[u8], key: &[u8]) -> Result<Vec<u8>> {
        if key.len() != 32 {
            return Err(anyhow::anyhow!("Invalid key length for AES-256"));
        }

        let cipher_key = aes_gcm::Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(cipher_key);
        
        // Generate a random nonce
        let mut nonce_bytes = [0u8; 12];
        self.rng.fill(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        // Encrypt the data
        let ciphertext = cipher.encrypt(nonce, data)
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;
        
        // Combine nonce + ciphertext
        let mut result = Vec::new();
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }

    /// Decrypt data with AES-256-GCM
    pub fn decrypt_data(&self, encrypted_data: &[u8], key: &[u8]) -> Result<Vec<u8>> {
        if key.len() != 32 {
            return Err(anyhow::anyhow!("Invalid key length for AES-256"));
        }

        if encrypted_data.len() < 12 {
            return Err(anyhow::anyhow!("Invalid encrypted data length"));
        }

        let cipher_key = aes_gcm::Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(cipher_key);
        
        // Extract nonce and ciphertext
        let nonce = Nonce::from_slice(&encrypted_data[..12]);
        let ciphertext = &encrypted_data[12..];
        
        // Decrypt the data
        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;
        
        Ok(plaintext)
    }

    /// Generate a random key
    pub fn generate_key(&mut self, length: usize) -> Vec<u8> {
        let mut key = vec![0u8; length];
        self.rng.fill(&mut key);
        key
    }

    /// Hash data with SHA-256
    pub fn hash_data(&self, data: &[u8]) -> Vec<u8> {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }

    /// Create TLS client configuration
    pub fn create_tls_client_config(&self) -> Result<ClientConfig> {
        let mut root_certs = RootCertStore::empty();
        
        // Load CA certificate
        let ca_cert = include_bytes!("../../certs/ca-cert.pem");
        let mut cert_reader = std::io::Cursor::new(ca_cert);
        let certs = certs(&mut cert_reader)?;
        
        for cert in certs {
            root_certs.add(cert)?;
        }

        let config = ClientConfig::builder()
            .with_root_certificates(root_certs)
            .with_no_client_auth();

        Ok(config)
    }

    /// Create TLS server configuration
    pub fn create_tls_server_config(&self) -> Result<ServerConfig> {
        // Load server certificate and key
        let cert_pem = include_bytes!("../../certs/server-cert.pem");
        let key_pem = include_bytes!("../../certs/server-key.pem");
        
        let mut cert_reader = std::io::Cursor::new(cert_pem);
        let certs = certs(&mut cert_reader)?;
        
        let mut key_reader = std::io::Cursor::new(key_pem);
        let key = rustls_pemfile::pkcs8_private_keys(&mut key_reader)?;
        
        if certs.is_empty() || key.is_empty() {
            return Err(anyhow::anyhow!("Invalid certificate or key"));
        }

        let config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key[0].clone())?;

        Ok(config)
    }

    /// Get key fingerprint
    pub fn get_key_fingerprint(&self) -> Result<String> {
        if let Some(ref public_key) = self.public_key {
            let public_key_bytes = public_key.to_bytes();
            let hash = self.hash_data(&public_key_bytes);
            Ok(hex::encode(&hash[..8])) // First 8 bytes as fingerprint
        } else {
            Err(anyhow::anyhow!("No public key available"))
        }
    }

    /// Validate key pair
    pub fn validate_key_pair(&self) -> Result<bool> {
        if let (Some(ref private_key), Some(ref public_key)) = (&self.private_key, &self.public_key) {
            let test_data = b"test data for validation";
            let signature = private_key.sign(test_data);
            let is_valid = public_key.verify(test_data, &signature).is_ok();
            Ok(is_valid)
        } else {
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_manager_creation() {
        let security_manager = SecurityManager::new();
        assert!(security_manager.is_ok());
    }

    #[test]
    fn test_key_generation() {
        let security_manager = SecurityManager::new().unwrap();
        let public_key = security_manager.get_public_key();
        assert!(public_key.is_ok());
    }

    #[test]
    fn test_signature_verification() {
        let security_manager = SecurityManager::new().unwrap();
        
        let test_data = "Hello, World!";
        let signature = security_manager.sign_data(&test_data).unwrap();
        let is_valid = security_manager.verify_signature(&test_data, &signature).unwrap();
        
        assert!(is_valid);
    }

    #[test]
    fn test_encryption_decryption() {
        let mut security_manager = SecurityManager::new().unwrap();
        
        let test_data = b"Secret data to encrypt";
        let key = security_manager.generate_key(32);
        
        let encrypted = security_manager.encrypt_data(test_data, &key).unwrap();
        let decrypted = security_manager.decrypt_data(&encrypted, &key).unwrap();
        
        assert_eq!(test_data, &decrypted[..]);
    }

    #[test]
    fn test_key_validation() {
        let security_manager = SecurityManager::new().unwrap();
        let is_valid = security_manager.validate_key_pair().unwrap();
        assert!(is_valid);
    }

    #[test]
    fn test_fingerprint() {
        let security_manager = SecurityManager::new().unwrap();
        let fingerprint = security_manager.get_key_fingerprint().unwrap();
        assert!(!fingerprint.is_empty());
    }
}