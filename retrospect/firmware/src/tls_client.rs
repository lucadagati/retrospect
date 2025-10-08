// TLS client module
use heapless::{String, Vec};
use log::info;
use core::str::FromStr;

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
        // Simulate TLS handshake
        self.connected = true;
        self.gateway_endpoint.push_str(gateway_endpoint).map_err(|_| "Gateway endpoint too long")?;
        self.device_id.push_str(device_id).map_err(|_| "Device ID too long")?;
        info!("TLS connection established (simulated).");

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
        // Simulate sending encrypted data
        info!("TLS TX: {:?}", data);
        Ok(())
    }

    pub fn receive_message(&mut self) -> Result<Option<Message>, &'static str> {
        // Simulate receiving encrypted data
        // For now, we'll simulate a deployment message after some time
        // In a real scenario, this would read from the network/serial
        
        // Example: Simulate receiving a deploy application message
        // let raw_message = "DEPLOY:app1:0102030405"; // Example raw message
        // if raw_message.starts_with("DEPLOY:") {
        //     let parts: Vec<&str, 3> = raw_message.split(':').collect();
        //     if parts.len() == 3 {
        //         let app_id = String::from_str(parts[1]).map_err(|_| "App ID too long")?;
        //         let bytecode_hex = parts[2];
        //         let mut bytecode = Vec::<u8, 1024>::new();
        //         // Simulate hex to bytes conversion
        //         for i in (0..bytecode_hex.len()).step_by(2) {
        //             let byte_str = &bytecode_hex[i..i+2];
        //             let byte = u8::from_str_radix(byte_str, 16).map_err(|_| "Invalid bytecode hex")?;
        //             bytecode.push(byte).map_err(|_| "Bytecode too large")?;
        //         }
        //         return Ok(Some(Message::DeployApplication { app_id, bytecode }));
        //     }
        // }
        
        Ok(None)
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
