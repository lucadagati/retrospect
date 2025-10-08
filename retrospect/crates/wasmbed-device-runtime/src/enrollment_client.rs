// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use heapless::{String, Vec};
use log::{error, info, warn};

use crate::tls_client::Keypair;

/// Enrollment Client for device registration
pub struct EnrollmentClient {
    enrolled: bool,
}

impl EnrollmentClient {
    pub fn new() -> Self {
        Self { enrolled: false }
    }

    pub async fn enroll(&mut self, keypair: &Keypair, gateway_endpoint: &str) -> Result<String<64>, &'static str> {
        info!("Starting enrollment process with gateway: {}", gateway_endpoint);
        
        // Simulate enrollment process
        // In a real implementation, this would:
        // 1. Connect to gateway
        // 2. Send enrollment request
        // 3. Send public key
        // 4. Receive device UUID
        // 5. Send acknowledgment
        
        info!("Enrollment request sent");
        
        // Simulate receiving device UUID
        let mut device_id = String::new();
        device_id.push_str("device-uuid-12345").unwrap();
        
        info!("Received device UUID: {}", device_id);
        
        info!("Sending enrollment acknowledgment");
        
        self.enrolled = true;
        info!("Enrollment completed successfully");
        
        Ok(device_id)
    }

    pub fn is_enrolled(&self) -> bool {
        self.enrolled
    }
}
