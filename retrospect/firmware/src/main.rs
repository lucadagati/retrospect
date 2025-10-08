#![no_std]
#![no_main]

use cortex_m_rt::entry;
use cortex_m::asm;
use heapless::{String, Vec};
use log::{error, info, warn};
use core::str::FromStr;

// Global allocator for no_std
use core::alloc::{GlobalAlloc, Layout};

struct DummyAllocator;

unsafe impl GlobalAlloc for DummyAllocator {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        core::ptr::null_mut()
    }
    
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // No-op
    }
}

#[global_allocator]
static ALLOCATOR: DummyAllocator = DummyAllocator;

mod hardware;
mod network;
mod tls_client;
mod wasm_runtime;

use hardware::HardwareManager;
use network::NetworkManager;
use tls_client::TlsClient;
use wasm_runtime::WasmRuntime;

/// Firmware configuration
#[derive(Debug, Clone)]
pub struct FirmwareConfig {
    pub device_id: String<64>,
    pub gateway_endpoint: String<64>,
    pub network_config: NetworkConfig,
    pub wasm_config: WasmConfig,
}

#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub ip_address: [u8; 4],
    pub gateway: [u8; 4],
    pub netmask: [u8; 4],
    pub mac_address: [u8; 6],
}

#[derive(Debug, Clone)]
pub struct WasmConfig {
    pub max_applications: usize,
    pub max_memory_per_app: usize,
    pub enable_host_functions: bool,
}

/// Main firmware structure
pub struct Firmware {
    hardware: HardwareManager,
    network: NetworkManager,
    tls_client: TlsClient,
    wasm_runtime: WasmRuntime,
    device_id: String<32>,
    gateway_endpoint: String<64>,
}

impl Firmware {
    pub fn new(device_id: String<32>, gateway_endpoint: String<64>) -> Self {
        Self {
            hardware: HardwareManager::new(),
            network: NetworkManager::new(),
            tls_client: TlsClient::new(),
            wasm_runtime: WasmRuntime::new(),
            device_id,
            gateway_endpoint,
        }
    }

    pub fn run(&mut self) -> Result<(), &'static str> {
        info!("Starting Wasmbed Firmware...");

        // Initialize hardware
        self.hardware.init()?;
        info!("Hardware initialized.");

        // Initialize network
        self.network.init()?;
        info!("Network initialized.");

        // Connect to gateway (simulated for now)
        self.tls_client.connect(&self.gateway_endpoint, &self.device_id)?;
        info!("Connected to gateway (simulated).");

        // Initialize WASM runtime
        self.wasm_runtime.init()?;
        info!("WASM runtime initialized.");

        // Main loop
        loop {
            // Process network events
            self.network.poll()?;

            // Process TLS messages
            if let Some(message) = self.tls_client.receive_message()? {
                match message {
                    tls_client::Message::DeployApplication { app_id, bytecode } => {
                        info!("Received deployment request for app: {}", app_id);
                        self.wasm_runtime.deploy_application(&app_id, &bytecode)?;
                        self.tls_client.send_deployment_ack(&app_id, true, None)?;
                    }
                    tls_client::Message::StopApplication { app_id } => {
                        info!("Received stop request for app: {}", app_id);
                        self.wasm_runtime.stop_application(&app_id)?;
                        self.tls_client.send_stop_ack(&app_id, true, None)?;
                    }
                    tls_client::Message::HeartbeatAck => {
                        // Heartbeat acknowledged
                    }
                    tls_client::Message::Unknown => {
                        warn!("Unknown message type received");
                    }
                }
            }

            // Run WASM applications
            self.wasm_runtime.run_applications()?;

            // Send heartbeat
            self.tls_client.send_heartbeat()?;

            // Simulate some delay
            asm::delay(1_000_000); // ~1 second delay
        }
    }
}

#[entry]
fn main() -> ! {
    // Initialize logging
    log::set_logger(&SimpleLogger).unwrap();
    log::set_max_level(log::LevelFilter::Info);

    info!("Wasmbed Firmware starting...");

    let mut firmware = Firmware::new(
        String::from_str("mcu-device-001").unwrap(),
        String::from_str("192.168.1.100:8443").unwrap(),
    );

    match firmware.run() {
        Ok(_) => {
            info!("Firmware stopped gracefully (should not happen)");
        }
        Err(e) => {
            error!("Firmware critical error: {}", e);
        }
    }

    loop {
        asm::nop();
    }
}

/// Simple logger implementation for no_std
struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, _record: &log::Record) {
        // In a real implementation, this would write to UART or other output
        // For now, we'll just ignore the logs
    }

    fn flush(&self) {}
}

// Panic handler is provided by cortex-m-rt for no_std builds