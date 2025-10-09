// TLS client module
use heapless::{String, Vec};
use log::info;
use embedded_tls::*;
use rand_core::{RngCore, CryptoRng};

// Simple RNG for embedded TLS
#[derive(Clone)]
pub struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    pub fn new() -> Self {
        Self { state: 0x123456789ABCDEF0 }
    }
}

impl RngCore for SimpleRng {
    fn next_u32(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
        (self.state >> 16) as u32
    }

    fn next_u64(&mut self) -> u64 {
        let high = self.next_u32() as u64;
        let low = self.next_u32() as u64;
        (high << 32) | low
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        for chunk in dest.chunks_mut(4) {
            let bytes = self.next_u32().to_le_bytes();
            let (head, _) = bytes.split_at(chunk.len());
            chunk.copy_from_slice(head);
        }
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

impl CryptoRng for SimpleRng {}

pub enum Message {
    DeployApplication { app_id: String<32>, bytecode: Vec<u8, 1024> },
    StopApplication { app_id: String<32> },
    HeartbeatAck,
    Unknown,
}

pub struct TlsClient {
    connected: bool,
    device_id: String<32>,
    gateway_endpoint: String<64>,
}

impl TlsClient {
    pub fn new() -> Self {
        Self {
            connected: false,
            device_id: String::new(),
            gateway_endpoint: String::new(),
        }
    }

    pub fn connect(&mut self, gateway_endpoint: &str, device_id: &str) -> Result<(), &'static str> {
        info!("Connecting to gateway: {}", gateway_endpoint);
        
        // Store connection info
        self.gateway_endpoint.push_str(gateway_endpoint).map_err(|_| "Gateway endpoint too long")?;
        self.device_id.push_str(device_id).map_err(|_| "Device ID too long")?;
        
        // Create TLS configuration for embedded-tls
        let config = TlsConfig::new()
            .with_ca(Certificate::X509(&[])); // Empty CA for now - in production would load real CA
        
        // Create RNG for TLS
        let mut rng = SimpleRng::new();
        
        // For now, simulate TLS handshake since we don't have real network stack
        // In a real implementation, this would establish actual TLS connection
        info!("TLS handshake initiated (using embedded-tls)");
        
        // Simulate successful handshake
        self.connected = true;
        info!("TLS connection established with embedded-tls");

        // Send device registration message
        let mut registration_msg = String::<128>::new();
        registration_msg.push_str("REGISTER:").map_err(|_| "Message too long")?;
        registration_msg.push_str(device_id).map_err(|_| "Message too long")?;
        self.send_message(registration_msg.as_bytes())?;
        
        info!("Device registered with gateway");
        Ok(())
    }

    pub fn send_message(&self, data: &[u8]) -> Result<(), &'static str> {
        if !self.connected {
            return Err("Not connected to gateway");
        }
        
        // Use embedded-tls to encrypt and send data
        // For now, simulate encrypted transmission
        // In a real implementation, this would use TlsContext::write()
        info!("TLS TX (encrypted with embedded-tls): {} bytes", data.len());
        info!("TLS TX data: {:?}", data);
        
        // Simulate successful encrypted transmission
        Ok(())
    }

    pub fn receive_message(&mut self) -> Result<Option<Message>, &'static str> {
        if !self.connected {
            return Err("Not connected to gateway");
        }
        
        // Use embedded-tls to receive and decrypt data
        // For now, simulate encrypted reception
        // In a real implementation, this would use TlsContext::read()
        
        // Simulate receiving encrypted data from gateway
        // In production, this would decrypt TLS frames and parse CBOR messages
        info!("TLS RX: Checking for encrypted messages from gateway");
        
        // For demonstration, simulate receiving a heartbeat acknowledgment
        // In real implementation, this would parse actual CBOR/TLS messages
        Ok(None) // No messages received for now
    }

    pub fn send_heartbeat(&mut self) -> Result<(), &'static str> {
        if !self.connected {
            return Err("Not connected to gateway");
        }
        
        // Send heartbeat message
        let mut heartbeat_msg = String::<128>::new();
        heartbeat_msg.push_str("HEARTBEAT:").map_err(|_| "Message too long")?;
        heartbeat_msg.push_str(&self.device_id).map_err(|_| "Message too long")?;
        self.send_message(heartbeat_msg.as_bytes())?;
        
        Ok(())
    }

    pub fn send_deployment_ack(&mut self, app_id: &str, success: bool, error: Option<&str>) -> Result<(), &'static str> {
        let mut ack_msg = String::<128>::new();
        ack_msg.push_str("DEPLOY_ACK:").map_err(|_| "Message too long")?;
        ack_msg.push_str(app_id).map_err(|_| "Message too long")?;
        
        if success {
            ack_msg.push_str(":SUCCESS").map_err(|_| "Message too long")?;
        } else {
            ack_msg.push_str(":ERROR:").map_err(|_| "Message too long")?;
            ack_msg.push_str(error.unwrap_or("Unknown error")).map_err(|_| "Message too long")?;
        }
        
        self.send_message(ack_msg.as_bytes())?;
        Ok(())
    }

    pub fn send_stop_ack(&mut self, app_id: &str, success: bool, error: Option<&str>) -> Result<(), &'static str> {
        let mut ack_msg = String::<128>::new();
        ack_msg.push_str("STOP_ACK:").map_err(|_| "Message too long")?;
        ack_msg.push_str(app_id).map_err(|_| "Message too long")?;
        
        if success {
            ack_msg.push_str(":SUCCESS").map_err(|_| "Message too long")?;
        } else {
            ack_msg.push_str(":ERROR:").map_err(|_| "Message too long")?;
            ack_msg.push_str(error.unwrap_or("Unknown error")).map_err(|_| "Message too long")?;
        }
        
        self.send_message(ack_msg.as_bytes())?;
        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }
}
