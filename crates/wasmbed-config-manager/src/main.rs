//! Wasmbed Configuration Manager
//! 
//! This binary provides a command-line interface for managing Wasmbed platform configuration.
//! It allows adding, removing, updating, and listing gateways, devices, and applications.

use clap::{Parser, Subcommand};
use wasmbed_config::{WasmbedConfig, GatewayConfig, DeviceConfig, ApplicationConfig};
use std::path::PathBuf;
use anyhow::{Result, Context};

#[derive(Parser)]
#[command(name = "wasmbed-config")]
#[command(about = "Wasmbed Configuration Manager")]
#[command(version = "1.0.0")]
struct Cli {
    /// Configuration file path
    #[arg(short, long, default_value = "config/wasmbed-config.yaml")]
    config: PathBuf,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Gateway management
    Gateway {
        #[command(subcommand)]
        action: GatewayAction,
    },
    /// Device management
    Device {
        #[command(subcommand)]
        action: DeviceAction,
    },
    /// Application management
    Application {
        #[command(subcommand)]
        action: ApplicationAction,
    },
    /// Show current configuration
    Show,
    /// Validate configuration
    Validate,
    /// Generate default configuration
    Init {
        /// Output file path
        #[arg(short, long, default_value = "config/wasmbed-config.yaml")]
        output: PathBuf,
    },
}

#[derive(Subcommand)]
enum GatewayAction {
    /// List all gateways
    List,
    /// Add a new gateway
    Add {
        /// Gateway name
        name: String,
        /// Gateway endpoint
        endpoint: String,
        /// TLS port
        #[arg(long, default_value = "30452")]
        tls_port: u16,
        /// HTTP port
        #[arg(long, default_value = "30453")]
        http_port: u16,
        /// Maximum devices
        #[arg(long, default_value = "50")]
        max_devices: u32,
        /// Region
        #[arg(long, default_value = "us-west-1")]
        region: String,
        /// Enable gateway
        #[arg(long, default_value = "true")]
        enabled: bool,
    },
    /// Remove a gateway
    Remove {
        /// Gateway name
        name: String,
    },
    /// Update gateway configuration
    Update {
        /// Gateway name
        name: String,
        /// New endpoint
        #[arg(long)]
        endpoint: Option<String>,
        /// New TLS port
        #[arg(long)]
        tls_port: Option<u16>,
        /// New HTTP port
        #[arg(long)]
        http_port: Option<u16>,
        /// New maximum devices
        #[arg(long)]
        max_devices: Option<u32>,
        /// New region
        #[arg(long)]
        region: Option<String>,
        /// Enable/disable gateway
        #[arg(long)]
        enabled: Option<bool>,
    },
    /// Show gateway details
    Show {
        /// Gateway name
        name: String,
    },
}

#[derive(Subcommand)]
enum DeviceAction {
    /// List all devices
    List,
    /// Add a new device
    Add {
        /// Device name
        name: String,
        /// Device type (ESP32, STM32, RISC-V)
        device_type: String,
        /// Architecture (xtensa, arm, riscv64)
        architecture: String,
        /// Gateway name
        gateway: String,
        /// Enable device
        #[arg(long, default_value = "true")]
        enabled: bool,
    },
    /// Remove a device
    Remove {
        /// Device name
        name: String,
    },
    /// Update device configuration
    Update {
        /// Device name
        name: String,
        /// New device type
        #[arg(long)]
        device_type: Option<String>,
        /// New architecture
        #[arg(long)]
        architecture: Option<String>,
        /// New gateway
        #[arg(long)]
        gateway: Option<String>,
        /// Enable/disable device
        #[arg(long)]
        enabled: Option<bool>,
    },
    /// Show device details
    Show {
        /// Device name
        name: String,
    },
}

#[derive(Subcommand)]
enum ApplicationAction {
    /// List all applications
    List,
    /// Add a new application
    Add {
        /// Application name
        name: String,
        /// Application description
        description: String,
        /// WASM bytes (base64 encoded)
        wasm_bytes: String,
        /// Target devices (comma-separated)
        target_devices: String,
        /// Enable application
        #[arg(long, default_value = "true")]
        enabled: bool,
    },
    /// Remove an application
    Remove {
        /// Application name
        name: String,
    },
    /// Update application configuration
    Update {
        /// Application name
        name: String,
        /// New description
        #[arg(long)]
        description: Option<String>,
        /// New WASM bytes
        #[arg(long)]
        wasm_bytes: Option<String>,
        /// New target devices
        #[arg(long)]
        target_devices: Option<String>,
        /// Enable/disable application
        #[arg(long)]
        enabled: Option<bool>,
    },
    /// Show application details
    Show {
        /// Application name
        name: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Gateway { action } => handle_gateway_action(&cli.config, action)?,
        Commands::Device { action } => handle_device_action(&cli.config, action)?,
        Commands::Application { action } => handle_application_action(&cli.config, action)?,
        Commands::Show => show_configuration(&cli.config)?,
        Commands::Validate => validate_configuration(&cli.config)?,
        Commands::Init { output } => generate_default_config(&output)?,
    }
    
    Ok(())
}

fn handle_gateway_action(config_path: &PathBuf, action: GatewayAction) -> Result<()> {
    let mut config = load_config(config_path)?;
    
    match action {
        GatewayAction::List => {
            println!("Gateways:");
            for gateway in &config.gateways.instances {
                println!("  - {} ({}) - {} devices max", 
                    gateway.name, 
                    gateway.endpoint, 
                    gateway.max_devices
                );
            }
        },
        GatewayAction::Add { name, endpoint, tls_port, http_port, max_devices, region, enabled } => {
            let gateway = GatewayConfig {
                name: name.clone(),
                endpoint,
                tls_port,
                http_port,
                max_devices,
                region,
                enabled,
            };
            config.add_gateway(gateway);
            save_config(&config, config_path)?;
            println!("Gateway '{}' added successfully", name);
        },
        GatewayAction::Remove { name } => {
            if config.remove_gateway(&name) {
                save_config(&config, config_path)?;
                println!("Gateway '{}' removed successfully", name);
            } else {
                println!("Gateway '{}' not found", name);
            }
        },
        GatewayAction::Update { name, endpoint, tls_port, http_port, max_devices, region, enabled } => {
            if let Some(existing) = config.get_gateway(&name) {
                let mut updated = existing.clone();
                if let Some(ep) = endpoint { updated.endpoint = ep; }
                if let Some(tp) = tls_port { updated.tls_port = tp; }
                if let Some(hp) = http_port { updated.http_port = hp; }
                if let Some(md) = max_devices { updated.max_devices = md; }
                if let Some(r) = region { updated.region = r; }
                if let Some(e) = enabled { updated.enabled = e; }
                
                if config.update_gateway(&name, updated) {
                    save_config(&config, config_path)?;
                    println!("Gateway '{}' updated successfully", name);
                } else {
                    println!("Failed to update gateway '{}'", name);
                }
            } else {
                println!("Gateway '{}' not found", name);
            }
        },
        GatewayAction::Show { name } => {
            if let Some(gateway) = config.get_gateway(&name) {
                println!("Gateway: {}", gateway.name);
                println!("  Endpoint: {}", gateway.endpoint);
                println!("  TLS Port: {}", gateway.tls_port);
                println!("  HTTP Port: {}", gateway.http_port);
                println!("  Max Devices: {}", gateway.max_devices);
                println!("  Region: {}", gateway.region);
                println!("  Enabled: {}", gateway.enabled);
            } else {
                println!("Gateway '{}' not found", name);
            }
        },
    }
    
    Ok(())
}

fn handle_device_action(config_path: &PathBuf, action: DeviceAction) -> Result<()> {
    let mut config = load_config(config_path)?;
    
    match action {
        DeviceAction::List => {
            println!("Devices:");
            for device in &config.devices.instances {
                println!("  - {} ({}) - Gateway: {}", 
                    device.name, 
                    device.device_type, 
                    device.gateway
                );
            }
        },
        DeviceAction::Add { name, device_type, architecture, gateway, enabled } => {
            let device = DeviceConfig {
                name: name.clone(),
                device_type,
                architecture,
                gateway,
                enabled,
            };
            config.add_device(device);
            save_config(&config, config_path)?;
            println!("Device '{}' added successfully", name);
        },
        DeviceAction::Remove { name } => {
            if config.remove_device(&name) {
                save_config(&config, config_path)?;
                println!("Device '{}' removed successfully", name);
            } else {
                println!("Device '{}' not found", name);
            }
        },
        DeviceAction::Update { name, device_type, architecture, gateway, enabled } => {
            if let Some(existing) = config.get_device(&name) {
                let mut updated = existing.clone();
                if let Some(dt) = device_type { updated.device_type = dt; }
                if let Some(arch) = architecture { updated.architecture = arch; }
                if let Some(gw) = gateway { updated.gateway = gw; }
                if let Some(e) = enabled { updated.enabled = e; }
                
                if config.update_device(&name, updated) {
                    save_config(&config, config_path)?;
                    println!("Device '{}' updated successfully", name);
                } else {
                    println!("Failed to update device '{}'", name);
                }
            } else {
                println!("Device '{}' not found", name);
            }
        },
        DeviceAction::Show { name } => {
            if let Some(device) = config.get_device(&name) {
                println!("Device: {}", device.name);
                println!("  Type: {}", device.device_type);
                println!("  Architecture: {}", device.architecture);
                println!("  Gateway: {}", device.gateway);
                println!("  Enabled: {}", device.enabled);
            } else {
                println!("Device '{}' not found", name);
            }
        },
    }
    
    Ok(())
}

fn handle_application_action(config_path: &PathBuf, action: ApplicationAction) -> Result<()> {
    let mut config = load_config(config_path)?;
    
    match action {
        ApplicationAction::List => {
            println!("Applications:");
            for app in &config.applications.instances {
                println!("  - {} - {} devices", 
                    app.name, 
                    app.target_devices.len()
                );
            }
        },
        ApplicationAction::Add { name, description, wasm_bytes, target_devices, enabled } => {
            let targets: Vec<String> = target_devices.split(',').map(|s| s.trim().to_string()).collect();
            let app = ApplicationConfig {
                name: name.clone(),
                description,
                wasm_bytes,
                target_devices: targets,
                config: std::collections::HashMap::new(),
                enabled,
            };
            config.add_application(app);
            save_config(&config, config_path)?;
            println!("Application '{}' added successfully", name);
        },
        ApplicationAction::Remove { name } => {
            if config.remove_application(&name) {
                save_config(&config, config_path)?;
                println!("Application '{}' removed successfully", name);
            } else {
                println!("Application '{}' not found", name);
            }
        },
        ApplicationAction::Update { name, description, wasm_bytes, target_devices, enabled } => {
            if let Some(existing) = config.get_application(&name) {
                let mut updated = existing.clone();
                if let Some(desc) = description { updated.description = desc; }
                if let Some(wb) = wasm_bytes { updated.wasm_bytes = wb; }
                if let Some(td) = target_devices { 
                    updated.target_devices = td.split(',').map(|s| s.trim().to_string()).collect();
                }
                if let Some(e) = enabled { updated.enabled = e; }
                
                if config.update_application(&name, updated) {
                    save_config(&config, config_path)?;
                    println!("Application '{}' updated successfully", name);
                } else {
                    println!("Failed to update application '{}'", name);
                }
            } else {
                println!("Application '{}' not found", name);
            }
        },
        ApplicationAction::Show { name } => {
            if let Some(app) = config.get_application(&name) {
                println!("Application: {}", app.name);
                println!("  Description: {}", app.description);
                println!("  WASM Size: {} bytes", app.wasm_bytes.len());
                println!("  Target Devices: {}", app.target_devices.join(", "));
                println!("  Enabled: {}", app.enabled);
            } else {
                println!("Application '{}' not found", name);
            }
        },
    }
    
    Ok(())
}

fn show_configuration(config_path: &PathBuf) -> Result<()> {
    let config = load_config(config_path)?;
    
    println!("Wasmbed Platform Configuration");
    println!("==============================");
    println!("Platform: {} v{}", config.platform.name, config.platform.version);
    println!("Description: {}", config.platform.description);
    println!();
    
    println!("Gateways: {}", config.gateways.instances.len());
    for gateway in &config.gateways.instances {
        println!("  - {} ({})", gateway.name, gateway.endpoint);
    }
    println!();
    
    println!("Devices: {}", config.devices.instances.len());
    for device in &config.devices.instances {
        println!("  - {} ({})", device.name, device.device_type);
    }
    println!();
    
    println!("Applications: {}", config.applications.instances.len());
    for app in &config.applications.instances {
        println!("  - {} ({} devices)", app.name, app.target_devices.len());
    }
    
    Ok(())
}

fn validate_configuration(config_path: &PathBuf) -> Result<()> {
    let config = load_config(config_path)?;
    
    println!("Validating configuration...");
    
    // Validate gateways
    for gateway in &config.gateways.instances {
        if gateway.name.is_empty() {
            println!("❌ Gateway with empty name found");
            return Ok(());
        }
        if gateway.endpoint.is_empty() {
            println!("❌ Gateway '{}' has empty endpoint", gateway.name);
            return Ok(());
        }
        if gateway.tls_port == 0 || gateway.http_port == 0 {
            println!("❌ Gateway '{}' has invalid ports", gateway.name);
            return Ok(());
        }
    }
    
    // Validate devices
    for device in &config.devices.instances {
        if device.name.is_empty() {
            println!("❌ Device with empty name found");
            return Ok(());
        }
        if device.device_type.is_empty() {
            println!("❌ Device '{}' has empty type", device.name);
            return Ok(());
        }
        if device.architecture.is_empty() {
            println!("❌ Device '{}' has empty architecture", device.name);
            return Ok(());
        }
        if device.gateway.is_empty() {
            println!("❌ Device '{}' has empty gateway", device.name);
            return Ok(());
        }
        
        // Check if gateway exists
        if config.get_gateway(&device.gateway).is_none() {
            println!("❌ Device '{}' references non-existent gateway '{}'", device.name, device.gateway);
            return Ok(());
        }
    }
    
    // Validate applications
    for app in &config.applications.instances {
        if app.name.is_empty() {
            println!("❌ Application with empty name found");
            return Ok(());
        }
        if app.description.is_empty() {
            println!("❌ Application '{}' has empty description", app.name);
            return Ok(());
        }
        if app.wasm_bytes.is_empty() {
            println!("❌ Application '{}' has empty WASM bytes", app.name);
            return Ok(());
        }
        
        // Check if target devices exist
        for target in &app.target_devices {
            if config.get_device(target).is_none() {
                println!("❌ Application '{}' references non-existent device '{}'", app.name, target);
                return Ok(());
            }
        }
    }
    
    println!("✅ Configuration is valid");
    Ok(())
}

fn generate_default_config(output_path: &PathBuf) -> Result<()> {
    let config = WasmbedConfig::default();
    config.to_file(output_path)?;
    println!("Default configuration generated at: {:?}", output_path);
    Ok(())
}

fn load_config(config_path: &PathBuf) -> Result<WasmbedConfig> {
    if !config_path.exists() {
        anyhow::bail!("Configuration file not found: {:?}", config_path);
    }
    
    WasmbedConfig::from_file(config_path)
        .with_context(|| format!("Failed to load configuration from {:?}", config_path))
}

fn save_config(config: &WasmbedConfig, config_path: &PathBuf) -> Result<()> {
    // Create directory if it doesn't exist
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    config.to_file(config_path)
        .with_context(|| format!("Failed to save configuration to {:?}", config_path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_gateway_operations() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config = WasmbedConfig::default();
        config.to_file(temp_file.path()).unwrap();
        
        let config_path = temp_file.path().to_path_buf();
        
        // Test add gateway
        handle_gateway_action(&config_path, GatewayAction::Add {
            name: "test-gateway".to_string(),
            endpoint: "127.0.0.1:30452".to_string(),
            tls_port: 30452,
            http_port: 30453,
            max_devices: 50,
            region: "us-west-1".to_string(),
            enabled: true,
        }).unwrap();
        
        // Test list gateways
        handle_gateway_action(&config_path, GatewayAction::List).unwrap();
        
        // Test show gateway
        handle_gateway_action(&config_path, GatewayAction::Show {
            name: "test-gateway".to_string(),
        }).unwrap();
        
        // Test update gateway
        handle_gateway_action(&config_path, GatewayAction::Update {
            name: "test-gateway".to_string(),
            endpoint: Some("127.0.0.1:30454".to_string()),
            tls_port: None,
            http_port: None,
            max_devices: None,
            region: None,
            enabled: None,
        }).unwrap();
        
        // Test remove gateway
        handle_gateway_action(&config_path, GatewayAction::Remove {
            name: "test-gateway".to_string(),
        }).unwrap();
    }

    #[test]
    fn test_device_operations() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let mut config = WasmbedConfig::default();
        
        // Add a gateway first
        config.add_gateway(GatewayConfig {
            name: "test-gateway".to_string(),
            endpoint: "127.0.0.1:30452".to_string(),
            tls_port: 30452,
            http_port: 30453,
            max_devices: 50,
            region: "us-west-1".to_string(),
            enabled: true,
        });
        
        config.to_file(temp_file.path()).unwrap();
        let config_path = temp_file.path().to_path_buf();
        
        // Test add device
        handle_device_action(&config_path, DeviceAction::Add {
            name: "test-device".to_string(),
            device_type: "ESP32".to_string(),
            architecture: "xtensa".to_string(),
            gateway: "test-gateway".to_string(),
            enabled: true,
        }).unwrap();
        
        // Test list devices
        handle_device_action(&config_path, DeviceAction::List).unwrap();
        
        // Test show device
        handle_device_action(&config_path, DeviceAction::Show {
            name: "test-device".to_string(),
        }).unwrap();
        
        // Test update device
        handle_device_action(&config_path, DeviceAction::Update {
            name: "test-device".to_string(),
            device_type: Some("STM32".to_string()),
            architecture: Some("arm".to_string()),
            gateway: None,
            enabled: None,
        }).unwrap();
        
        // Test remove device
        handle_device_action(&config_path, DeviceAction::Remove {
            name: "test-device".to_string(),
        }).unwrap();
    }

    #[test]
    fn test_application_operations() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let mut config = WasmbedConfig::default();
        
        // Add a gateway and device first
        config.add_gateway(GatewayConfig {
            name: "test-gateway".to_string(),
            endpoint: "127.0.0.1:30452".to_string(),
            tls_port: 30452,
            http_port: 30453,
            max_devices: 50,
            region: "us-west-1".to_string(),
            enabled: true,
        });
        
        config.add_device(DeviceConfig {
            name: "test-device".to_string(),
            device_type: "ESP32".to_string(),
            architecture: "xtensa".to_string(),
            gateway: "test-gateway".to_string(),
            enabled: true,
        });
        
        config.to_file(temp_file.path()).unwrap();
        let config_path = temp_file.path().to_path_buf();
        
        // Test add application
        handle_application_action(&config_path, ApplicationAction::Add {
            name: "test-app".to_string(),
            description: "Test application".to_string(),
            wasm_bytes: "dGVzdA==".to_string(), // "test" in base64
            target_devices: "test-device".to_string(),
            enabled: true,
        }).unwrap();
        
        // Test list applications
        handle_application_action(&config_path, ApplicationAction::List).unwrap();
        
        // Test show application
        handle_application_action(&config_path, ApplicationAction::Show {
            name: "test-app".to_string(),
        }).unwrap();
        
        // Test update application
        handle_application_action(&config_path, ApplicationAction::Update {
            name: "test-app".to_string(),
            description: Some("Updated test application".to_string()),
            wasm_bytes: None,
            target_devices: None,
            enabled: None,
        }).unwrap();
        
        // Test remove application
        handle_application_action(&config_path, ApplicationAction::Remove {
            name: "test-app".to_string(),
        }).unwrap();
    }

    #[test]
    fn test_configuration_validation() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config = WasmbedConfig::default();
        config.to_file(temp_file.path()).unwrap();
        let config_path = temp_file.path().to_path_buf();
        
        // Test validation
        validate_configuration(&config_path).unwrap();
    }

    #[test]
    fn test_show_configuration() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config = WasmbedConfig::default();
        config.to_file(temp_file.path()).unwrap();
        let config_path = temp_file.path().to_path_buf();
        
        // Test show
        show_configuration(&config_path).unwrap();
    }
}
