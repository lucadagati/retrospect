// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

#![no_std]
#![no_main]

use panic_halt as _;
use esp32_hal::entry;

mod wasmbed_client;
mod handlers;
mod memory;
mod wasm_runtime;
mod application_manager;
mod security;
mod allocator;
mod monitoring;
mod serial_interface;
mod wifi_manager;

use wasmbed_client::{WasmbedClient, ClientConfig};
use wasm_runtime::WasmRuntime;
use application_manager::ApplicationManager;
use monitoring::MonitoringSystem;
use serial_interface::SerialInterface;
use wifi_manager::WifiManager;

#[entry]
fn main() -> ! {
    // Initialize ESP32 peripherals
    let peripherals = esp32_hal::Peripherals::take().unwrap();
    let mut delay = esp32_hal::delay::Delay::new(&peripherals.SYSTIMER);
    
    // Initialize WiFi manager
    let mut wifi_manager = WifiManager::new(peripherals.WIFI, peripherals.RADIO_CLK);
    
    // Initialize WASM runtime
    let wasm_runtime = WasmRuntime::new();
    
    // Initialize application manager
    let mut app_manager = ApplicationManager::new(wasm_runtime);
    
    // Initialize monitoring system
    let mut monitoring_system = MonitoringSystem::new();
    
    // Initialize Wasmbed client with ESP32-specific configuration
    let config = ClientConfig {
        gateway_address: "172.19.0.2:30423",
        heartbeat_interval: 30,
        reconnect_delay: 5,
        max_reconnect_attempts: 10,
        wifi_enabled: true,
    };
    
    let mut client = WasmbedClient::with_config(config);
    
    // Initialize serial interface
    let mut serial = SerialInterface::new();
    
    // Connect to WiFi
    wifi_manager.connect("WasmbedESP32", "password123").unwrap();
    
    // Simple main loop with ESP32-specific features
    loop {
        // Simulate client operations
        client.simulate_operation();
        
        // Process WASM applications
        app_manager.process_applications();
        
        // Run monitoring tasks
        monitoring_system.collect_metrics();
        monitoring_system.run_health_checks();
        monitoring_system.process_alerts();
        monitoring_system.update_dashboard();
        
        // Process serial commands
        serial.process_commands(&mut client, &mut app_manager, &mut monitoring_system);
        
        // ESP32-specific delay
        delay.delay_ms(100u32);
    }
}