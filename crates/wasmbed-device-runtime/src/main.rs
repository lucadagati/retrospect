// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

#![no_std]
#![no_main]

use core::time::Duration;
use heapless::{String, Vec};
use log::{error, info, warn};

mod tls_client;
mod enrollment_client;
mod wasm_runtime;
mod keypair_generator;
mod common_device_runtime;

use tls_client::TlsClient;
use enrollment_client::EnrollmentClient;
use wasm_runtime::WasmRuntime;
use keypair_generator::KeypairGenerator;
use common_device_runtime::CommonDeviceRuntime;

/// Device configuration
#[derive(Debug, Clone)]
pub struct DeviceConfig {
    pub device_type: DeviceType,
    pub architecture: String<32>,
    pub capabilities: Vec<String<16>, 8>,
    pub gateway_endpoint: String<64>,
    pub device_id: Option<String<64>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeviceType {
    Mcu,
    Mpu,
    RiscV,
}

/// Device Runtime implementation
pub struct DeviceRuntime {
    config: DeviceConfig,
    tls_client: TlsClient,
    enrollment_client: EnrollmentClient,
    wasm_runtime: WasmRuntime,
    keypair_generator: KeypairGenerator,
    common_runtime: CommonDeviceRuntime,
}

impl DeviceRuntime {
    pub fn new(config: DeviceConfig) -> Self {
        Self {
            tls_client: TlsClient::new(),
            enrollment_client: EnrollmentClient::new(),
            wasm_runtime: WasmRuntime::new(),
            keypair_generator: KeypairGenerator::new(),
            common_runtime: CommonDeviceRuntime::new(),
            config,
        }
    }

    pub async fn run(&mut self) -> Result<(), &'static str> {
        info!("Starting Device Runtime...");

        // Initialize device
        self.initialize_device().await?;

        // Generate keypair if not already present
        if self.config.device_id.is_none() {
            self.generate_keypair().await?;
        }

        // Start enrollment process
        self.enroll_device().await?;

        // Establish TLS connection
        self.connect_to_gateway().await?;

        // Start main runtime loop
        self.runtime_loop().await?;

        Ok(())
    }

    async fn initialize_device(&mut self) -> Result<(), &'static str> {
        info!("Initializing device...");
        
        // Initialize common device runtime
        self.common_runtime.initialize().await?;
        
        info!("Device initialized successfully");
        Ok(())
    }

    async fn generate_keypair(&mut self) -> Result<(), &'static str> {
        info!("Generating device keypair...");
        
        let keypair = self.keypair_generator.generate().await?;
        
        // Store keypair in non-volatile memory
        self.common_runtime.store_keypair(&keypair).await?;
        
        info!("Keypair generated and stored");
        Ok(())
    }

    async fn enroll_device(&mut self) -> Result<(), &'static str> {
        info!("Starting device enrollment...");
        
        // Get stored keypair
        let keypair = self.common_runtime.get_keypair().await?;
        
        // Start enrollment process
        let device_id = self.enrollment_client.enroll(&keypair, &self.config.gateway_endpoint).await?;
        
        // Store device ID
        self.config.device_id = Some(device_id);
        self.common_runtime.store_device_id(&self.config.device_id.as_ref().unwrap()).await?;
        
        info!("Device enrolled successfully with ID: {}", self.config.device_id.as_ref().unwrap());
        Ok(())
    }

    async fn connect_to_gateway(&mut self) -> Result<(), &'static str> {
        info!("Connecting to gateway...");
        
        // Get stored keypair
        let keypair = self.common_runtime.get_keypair().await?;
        
        // Establish TLS connection
        self.tls_client.connect(&self.config.gateway_endpoint, &keypair).await?;
        
        info!("Connected to gateway successfully");
        Ok(())
    }

    async fn runtime_loop(&mut self) -> Result<(), &'static str> {
        info!("Starting main runtime loop...");
        
        loop {
            // Send heartbeat
            self.send_heartbeat().await?;
            
            // Process incoming messages
            self.process_messages().await?;
            
            // Run WASM applications
            self.run_wasm_applications().await?;
            
            // Wait before next iteration
            self.common_runtime.sleep(Duration::from_secs(30)).await;
        }
    }

    async fn send_heartbeat(&mut self) -> Result<(), &'static str> {
        self.tls_client.send_heartbeat().await?;
        Ok(())
    }

    async fn process_messages(&mut self) -> Result<(), &'static str> {
        if let Some(message) = self.tls_client.receive_message().await? {
            match message {
                tls_client::Message::DeployApplication { app_id, bytecode } => {
                    info!("Received deployment request for app: {}", app_id);
                    self.wasm_runtime.deploy_application(&app_id, &bytecode).await?;
                    self.tls_client.send_deployment_ack(&app_id, true, None).await?;
                }
                tls_client::Message::StopApplication { app_id } => {
                    info!("Received stop request for app: {}", app_id);
                    self.wasm_runtime.stop_application(&app_id).await?;
                    self.tls_client.send_stop_ack(&app_id, true, None).await?;
                }
                tls_client::Message::HeartbeatAck => {
                    // Heartbeat acknowledged
                }
                _ => {
                    warn!("Unknown message type received");
                }
            }
        }
        Ok(())
    }

    async fn run_wasm_applications(&mut self) -> Result<(), &'static str> {
        self.wasm_runtime.run_applications().await?;
        Ok(())
    }
}

#[no_mangle]
pub extern "C" fn main() -> i32 {
    // Initialize logging
    log::set_logger(&SimpleLogger).unwrap();
    log::set_max_level(log::LevelFilter::Info);

    // Create device configuration
    let mut config = DeviceConfig {
        device_type: DeviceType::Mcu,
        architecture: String::new(),
        capabilities: Vec::new(),
        gateway_endpoint: String::new(),
        device_id: None,
    };

    config.architecture.push_str("ARM_CORTEX_M").unwrap();
    config.gateway_endpoint.push_str("localhost:8443").unwrap();

    // Add capabilities
    let mut wasm_cap = String::new();
    wasm_cap.push_str("wasm").unwrap();
    let _ = config.capabilities.push(wasm_cap);
    
    let mut tls_cap = String::new();
    tls_cap.push_str("tls").unwrap();
    let _ = config.capabilities.push(tls_cap);

    // Create and run device runtime
    let mut runtime = DeviceRuntime::new(config);
    
    // Run the runtime (this will block)
    match futures::executor::block_on(runtime.run()) {
        Ok(_) => {
            info!("Device runtime completed successfully");
            0
        }
        Err(e) => {
            error!("Device runtime failed: {}", e);
            1
        }
    }
}

/// Simple logger implementation for no_std
struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        // In a real implementation, this would write to UART or other output
        // For now, we'll just ignore the logs
    }

    fn flush(&self) {}
}
