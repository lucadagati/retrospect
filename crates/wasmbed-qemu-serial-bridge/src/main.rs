// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use std::env;
use std::process;
use tracing::{info, error};

use wasmbed_qemu_serial_bridge::QemuSerialBridge;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    info!("Starting Wasmbed QEMU Serial Bridge");
    
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <device-id> <serial-socket-path> [gateway-host] [gateway-port]", args[0]);
        eprintln!("Example: {} qemu-device-1 /tmp/wasmbed-qemu-qemu-device-1.sock 172.19.0.2 30423", args[0]);
        process::exit(1);
    }
    
    let device_id = args[1].clone();
    let serial_socket_path = args[2].clone();
    let gateway_host = args.get(3).unwrap_or(&"172.19.0.2".to_string()).clone();
    let gateway_port = args.get(4).unwrap_or(&"30423".to_string()).parse::<u16>().unwrap_or(30423);
    
    info!("Device ID: {}", device_id);
    info!("Serial Socket: {}", serial_socket_path);
    info!("Gateway: {}:{}", gateway_host, gateway_port);
    
    // Create and run QEMU serial bridge
    let mut bridge = QemuSerialBridge::new(
        device_id.clone(),
        serial_socket_path,
        gateway_host,
        gateway_port,
    );
    
    // Run the device simulation
    if let Err(e) = bridge.run_device_simulation() {
        error!("QEMU serial bridge error: {}", e);
        process::exit(1);
    }
}
