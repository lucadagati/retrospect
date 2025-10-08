// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use core::time::Duration;
use heapless::{String, Vec};
use log::{error, info, warn};

use crate::tls_client::Keypair;

/// Common Device Runtime for shared functionality
pub struct CommonDeviceRuntime {
    initialized: bool,
    stored_keypair: Option<Keypair>,
    stored_device_id: Option<String<64>>,
}

impl CommonDeviceRuntime {
    pub fn new() -> Self {
        Self {
            initialized: false,
            stored_keypair: None,
            stored_device_id: None,
        }
    }

    pub async fn initialize(&mut self) -> Result<(), &'static str> {
        info!("Initializing common device runtime...");
        
        // Initialize hardware components
        // In a real implementation, this would:
        // 1. Initialize UART for logging
        // 2. Initialize flash memory for storage
        // 3. Initialize network interface
        // 4. Initialize other peripherals
        
        self.initialized = true;
        info!("Common device runtime initialized");
        Ok(())
    }

    pub async fn store_keypair(&mut self, keypair: &Keypair) -> Result<(), &'static str> {
        info!("Storing keypair in non-volatile memory...");
        
        // In a real implementation, this would write to flash memory
        self.stored_keypair = Some(keypair.clone());
        
        info!("Keypair stored successfully");
        Ok(())
    }

    pub async fn get_keypair(&self) -> Result<Keypair, &'static str> {
        match &self.stored_keypair {
            Some(keypair) => Ok(keypair.clone()),
            None => Err("No keypair stored"),
        }
    }

    pub async fn store_device_id(&mut self, device_id: &str) -> Result<(), &'static str> {
        info!("Storing device ID: {}", device_id);
        
        // In a real implementation, this would write to flash memory
        let mut id = String::new();
        for c in device_id.chars() {
            if id.push(c).is_err() {
                return Err("Device ID too long");
            }
        }
        
        self.stored_device_id = Some(id);
        info!("Device ID stored successfully");
        Ok(())
    }

    pub async fn get_device_id(&self) -> Result<String<64>, &'static str> {
        match &self.stored_device_id {
            Some(id) => Ok(id.clone()),
            None => Err("No device ID stored"),
        }
    }

    pub async fn sleep(&self, duration: Duration) {
        // In a real implementation, this would use a proper sleep function
        // For now, we'll simulate a delay
        info!("Sleeping for {:?}", duration);
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}
