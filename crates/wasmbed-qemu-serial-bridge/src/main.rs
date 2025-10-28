// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use wasmbed_qemu_serial_bridge::QemuSerialBridge;
use clap::{Parser, Subcommand};
use tracing::{info, error};

#[derive(Parser)]
#[command(name = "wasmbed-qemu-serial-bridge")]
#[command(about = "TCP serial bridge for QEMU ARM Cortex-M communication")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the serial bridge server
    Start {
        #[arg(short, long, default_value = "30001")]
        external_port: u16,
        #[arg(short, long, default_value = "30000")]
        qemu_base_port: u16,
    },
    /// Send data to a device
    Send {
        #[arg(short, long)]
        device_id: String,
        #[arg(short, long)]
        data: String,
    },
    /// List connected devices
    List,
    /// Get device status
    Status {
        #[arg(short, long)]
        device_id: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Start { external_port, qemu_base_port } => {
            info!("Starting QEMU serial bridge...");
            info!("External port: {}", external_port);
            info!("QEMU base port: {}", qemu_base_port);

            let bridge = QemuSerialBridge::new(external_port, qemu_base_port);
            
            if let Err(e) = bridge.start().await {
                error!("Serial bridge error: {}", e);
                return Err(e);
            }
        }
        Commands::Send { device_id, data } => {
            info!("Sending data to device: {}", device_id);
            
            let bridge = QemuSerialBridge::new(30001, 30000);
            if let Err(e) = bridge.send_to_device(&device_id, data.as_bytes()).await {
                error!("Failed to send data to device {}: {}", device_id, e);
                return Err(e);
            }
            
            info!("Data sent successfully to device: {}", device_id);
        }
        Commands::List => {
            info!("Listing connected devices...");
            
            let bridge = QemuSerialBridge::new(30001, 30000);
            let devices = bridge.list_devices().await;
            
            if devices.is_empty() {
                println!("No devices connected");
            } else {
                for device in devices {
                    println!("Device: {} - State: {:?}", device.device_id, device.state);
                }
            }
        }
        Commands::Status { device_id } => {
            info!("Getting status for device: {}", device_id);
            
            let bridge = QemuSerialBridge::new(30001, 30000);
            if let Some(device) = bridge.get_device_status(&device_id).await {
                println!("Device: {}", device.device_id);
                println!("State: {:?}", device.state);
                println!("External addr: {:?}", device.external_addr);
                println!("QEMU addr: {:?}", device.qemu_addr);
                println!("Last heartbeat: {:?}", device.last_heartbeat);
            } else {
                println!("Device {} not found", device_id);
            }
        }
    }

    Ok(())
}

