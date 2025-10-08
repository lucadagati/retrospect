// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use wasmbed_qemu_manager::{QemuManager, QemuDevice, WasmConfig};
use clap::{Parser, Subcommand};
use serde_json;
use std::io::{self, Write};

#[derive(Parser)]
#[command(name = "wasmbed-qemu-manager")]
#[command(about = "QEMU device emulation manager for Wasmbed platform")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new QEMU device
    Create {
        #[arg(short, long)]
        id: String,
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        architecture: String,
        #[arg(short, long)]
        device_type: String,
        #[arg(short, long, default_value = "Mps2An385")]
        mcu_type: String,
    },
    /// Start a QEMU device
    Start {
        #[arg(short, long)]
        id: String,
    },
    /// Stop a QEMU device
    Stop {
        #[arg(short, long)]
        id: String,
    },
    /// Deploy WASM to a device
    Deploy {
        #[arg(short, long)]
        device_id: String,
        #[arg(short, long)]
        wasm_file: String,
        #[arg(short, long, default_value = "128")]
        memory_limit: u32,
        #[arg(short, long, default_value = "30")]
        execution_timeout: u32,
    },
    /// List all devices
    List,
    /// Get device status
    Status {
        #[arg(short, long)]
        id: String,
    },
    /// Run as daemon
    Daemon {
        #[arg(short, long, default_value = "qemu-system-arm")]
        qemu_binary: String,
        #[arg(short, long, default_value = "30000")]
        base_port: u16,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Create { id, name, architecture, device_type, mcu_type } => {
            let manager = QemuManager::new("qemu-system-arm".to_string(), 30000);
            
            // Parse MCU type
            let mcu_type_enum = match mcu_type.as_str() {
                "Mps2An385" => wasmbed_qemu_manager::McuType::Mps2An385,
                "Mps2An386" => wasmbed_qemu_manager::McuType::Mps2An386,
                "Mps2An500" => wasmbed_qemu_manager::McuType::Mps2An500,
                "Mps2An505" => wasmbed_qemu_manager::McuType::Mps2An505,
                "Stm32Vldiscovery" => wasmbed_qemu_manager::McuType::Stm32Vldiscovery,
                "OlimexStm32H405" => wasmbed_qemu_manager::McuType::OlimexStm32H405,
                _ => {
                    println!("Unknown MCU type: {}. Using default Mps2An385", mcu_type);
                    wasmbed_qemu_manager::McuType::Mps2An385
                }
            };
            
            let device = manager.create_device(id, name, architecture, device_type, mcu_type_enum, None).await?;
            println!("Created device: {}", serde_json::to_string_pretty(&device)?);
        }
        Commands::Start { id } => {
            let manager = QemuManager::new("qemu-system-arm".to_string(), 30000);
            manager.start_device(&id).await?;
            println!("Started device: {}", id);
        }
        Commands::Stop { id } => {
            let manager = QemuManager::new("qemu-system-arm".to_string(), 30000);
            manager.stop_device(&id).await?;
            println!("Stopped device: {}", id);
        }
        Commands::Deploy { device_id, wasm_file, memory_limit, execution_timeout } => {
            let manager = QemuManager::new("qemu-system-arm".to_string(), 30000);
            
            // Read WASM file
            let wasm_bytes = std::fs::read(&wasm_file)?;
            
            let config = WasmConfig {
                memory_limit,
                execution_timeout,
                host_functions: vec!["print_message".to_string(), "get_timestamp".to_string()],
            };
            
            manager.deploy_wasm(&device_id, wasm_bytes, config).await?;
            println!("Deployed WASM to device: {}", device_id);
        }
        Commands::List => {
            let manager = QemuManager::new("qemu-system-arm".to_string(), 30000);
            let devices = manager.list_devices().await;
            println!("Devices: {}", serde_json::to_string_pretty(&devices)?);
        }
        Commands::Status { id } => {
            let manager = QemuManager::new("qemu-system-arm".to_string(), 30000);
            if let Some(device) = manager.get_device(&id).await {
                println!("Device status: {}", serde_json::to_string_pretty(&device)?);
            } else {
                println!("Device not found: {}", id);
            }
        }
        Commands::Daemon { qemu_binary, base_port } => {
            println!("Starting QEMU Manager daemon...");
            println!("QEMU binary: {}", qemu_binary);
            println!("Base port: {}", base_port);
            
            let manager = QemuManager::new(qemu_binary, base_port);
            
            // Keep running
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
    }

    Ok(())
}
