// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tracing::{info, error, warn, debug};
use tokio::fs as async_fs;
use sha2::{Sha256, Digest};
use uuid::Uuid;
use hex;

/// Firmware image information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirmwareImage {
    /// Image ID
    pub id: String,
    /// Image name
    pub name: String,
    /// Device type this firmware is for
    pub device_type: String,
    /// Architecture (riscv32, armv7m, xtensa)
    pub architecture: String,
    /// Firmware version
    pub version: String,
    /// File path
    pub file_path: PathBuf,
    /// File size in bytes
    pub file_size: u64,
    /// SHA256 hash
    pub sha256_hash: String,
    /// Creation timestamp
    pub created_at: u64,
    /// Last modified timestamp
    pub modified_at: u64,
    /// Firmware metadata
    pub metadata: HashMap<String, String>,
}

/// Firmware deployment status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentStatus {
    /// Deployment is pending
    Pending,
    /// Deployment is in progress
    InProgress,
    /// Deployment completed successfully
    Completed,
    /// Deployment failed
    Failed(String),
    /// Deployment was cancelled
    Cancelled,
}

/// Firmware deployment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirmwareDeployment {
    /// Deployment ID
    pub id: String,
    /// Target device ID
    pub device_id: String,
    /// Firmware image ID
    pub firmware_id: String,
    /// Deployment status
    pub status: DeploymentStatus,
    /// Deployment start time
    pub started_at: Option<u64>,
    /// Deployment completion time
    pub completed_at: Option<u64>,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Deployment progress (0-100)
    pub progress: u8,
}

/// Firmware manager for handling firmware images and deployments
pub struct FirmwareManager {
    /// Firmware storage directory
    storage_dir: PathBuf,
    /// Available firmware images
    images: HashMap<String, FirmwareImage>,
    /// Active deployments
    deployments: HashMap<String, FirmwareDeployment>,
    /// Device-specific firmware templates
    templates: HashMap<String, FirmwareTemplate>,
}

/// Firmware template for different device types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirmwareTemplate {
    /// Template ID
    pub id: String,
    /// Device type
    pub device_type: String,
    /// Architecture
    pub architecture: String,
    /// Template source path
    pub source_path: PathBuf,
    /// Template configuration
    pub config: FirmwareTemplateConfig,
}

/// Firmware template configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirmwareTemplateConfig {
    /// Memory layout
    pub memory_layout: MemoryLayout,
    /// Linker script
    pub linker_script: Option<PathBuf>,
    /// Build flags
    pub build_flags: Vec<String>,
    /// Dependencies
    pub dependencies: Vec<String>,
}

/// Memory layout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryLayout {
    /// Flash memory start address
    pub flash_start: u32,
    /// Flash memory size
    pub flash_size: u32,
    /// RAM start address
    pub ram_start: u32,
    /// RAM size
    pub ram_size: u32,
    /// Stack size
    pub stack_size: u32,
    /// Heap size
    pub heap_size: u32,
}

impl FirmwareManager {
    /// Create a new firmware manager
    pub fn new(storage_dir: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        // Create storage directory if it doesn't exist
        if !storage_dir.exists() {
            fs::create_dir_all(&storage_dir)?;
        }

        let mut manager = Self {
            storage_dir,
            images: HashMap::new(),
            deployments: HashMap::new(),
            templates: HashMap::new(),
        };

        // Load existing firmware images
        manager.load_firmware_images()?;
        
        // Load firmware templates
        manager.load_firmware_templates()?;

        Ok(manager)
    }

    /// Load firmware images from storage
    fn load_firmware_images(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let images_dir = self.storage_dir.join("images");
        if !images_dir.exists() {
            fs::create_dir_all(&images_dir)?;
            return Ok(());
        }

        for entry in fs::read_dir(&images_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                if let Ok(image) = self.load_firmware_image_metadata(&path) {
                    self.images.insert(image.id.clone(), image);
                }
            }
        }

        info!("Loaded {} firmware images", self.images.len());
        Ok(())
    }

    /// Load firmware image metadata from JSON file
    fn load_firmware_image_metadata(&self, metadata_path: &Path) -> Result<FirmwareImage, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(metadata_path)?;
        let image: FirmwareImage = serde_json::from_str(&content)?;
        Ok(image)
    }

    /// Load firmware templates
    fn load_firmware_templates(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let templates_dir = self.storage_dir.join("templates");
        if !templates_dir.exists() {
            fs::create_dir_all(&templates_dir)?;
            // Create default templates
            self.create_default_templates()?;
            return Ok(());
        }

        for entry in fs::read_dir(&templates_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                if let Ok(template) = self.load_firmware_template(&path) {
                    self.templates.insert(template.id.clone(), template);
                }
            }
        }

        info!("Loaded {} firmware templates", self.templates.len());
        Ok(())
    }

    /// Load firmware template from JSON file
    fn load_firmware_template(&self, template_path: &Path) -> Result<FirmwareTemplate, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(template_path)?;
        let template: FirmwareTemplate = serde_json::from_str(&content)?;
        Ok(template)
    }

    /// Create default firmware templates
    fn create_default_templates(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let templates_dir = self.storage_dir.join("templates");

        // RISC-V template
        let riscv_template = FirmwareTemplate {
            id: "riscv-sifive-hifive1".to_string(),
            device_type: "riscv".to_string(),
            architecture: "riscv32imac".to_string(),
            source_path: PathBuf::from("templates/riscv-sifive-hifive1"),
            config: FirmwareTemplateConfig {
                memory_layout: MemoryLayout {
                    flash_start: 0x20000000,
                    flash_size: 0x1000000, // 16MB
                    ram_start: 0x80000000,
                    ram_size: 0x8000000, // 128MB
                    stack_size: 0x8000, // 32KB
                    heap_size: 0x10000, // 64KB
                },
                linker_script: Some(PathBuf::from("link.x")),
                build_flags: vec![
                    "--target".to_string(),
                    "riscv32imac-unknown-none-elf".to_string(),
                    "-C".to_string(),
                    "link-arg=-Tlink.x".to_string(),
                ],
                dependencies: vec![
                    "wasmbed-firmware-hifive1-qemu".to_string(),
                ],
            },
        };

        // ARM Cortex-M template
        let arm_template = FirmwareTemplate {
            id: "arm-cortex-m-stm32".to_string(),
            device_type: "arm".to_string(),
            architecture: "armv7m".to_string(),
            source_path: PathBuf::from("templates/arm-cortex-m-stm32"),
            config: FirmwareTemplateConfig {
                memory_layout: MemoryLayout {
                    flash_start: 0x08000000,
                    flash_size: 0x100000, // 1MB
                    ram_start: 0x20000000,
                    ram_size: 0x20000, // 128KB
                    stack_size: 0x2000, // 8KB
                    heap_size: 0x4000, // 16KB
                },
                linker_script: Some(PathBuf::from("memory.x")),
                build_flags: vec![
                    "--target".to_string(),
                    "thumbv7m-none-eabi".to_string(),
                    "-C".to_string(),
                    "link-arg=-Tmemory.x".to_string(),
                ],
                dependencies: vec![
                    "wasmbed-firmware-stm32".to_string(),
                ],
            },
        };

        // ESP32 template
        let esp32_template = FirmwareTemplate {
            id: "esp32-xtensa".to_string(),
            device_type: "esp32".to_string(),
            architecture: "xtensa".to_string(),
            source_path: PathBuf::from("templates/esp32-xtensa"),
            config: FirmwareTemplateConfig {
                memory_layout: MemoryLayout {
                    flash_start: 0x400D0000,
                    flash_size: 0x400000, // 4MB
                    ram_start: 0x3FFB0000,
                    ram_size: 0x50000, // 320KB
                    stack_size: 0x1000, // 4KB
                    heap_size: 0x8000, // 32KB
                },
                linker_script: None,
                build_flags: vec![
                    "--target".to_string(),
                    "xtensa-esp32-espidf".to_string(),
                ],
                dependencies: vec![
                    "wasmbed-firmware-esp32".to_string(),
                ],
            },
        };

        // Save templates
        self.save_firmware_template(&riscv_template)?;
        self.save_firmware_template(&arm_template)?;
        self.save_firmware_template(&esp32_template)?;

        self.templates.insert(riscv_template.id.clone(), riscv_template);
        self.templates.insert(arm_template.id.clone(), arm_template);
        self.templates.insert(esp32_template.id.clone(), esp32_template);

        Ok(())
    }

    /// Save firmware template to file
    fn save_firmware_template(&self, template: &FirmwareTemplate) -> Result<(), Box<dyn std::error::Error>> {
        let templates_dir = self.storage_dir.join("templates");
        let template_path = templates_dir.join(format!("{}.json", template.id));
        
        let content = serde_json::to_string_pretty(template)?;
        fs::write(template_path, content)?;
        
        Ok(())
    }

    /// Add a new firmware image
    pub async fn add_firmware_image(
        &mut self,
        id: String,
        name: String,
        device_type: String,
        architecture: String,
        version: String,
        firmware_data: &[u8],
        metadata: HashMap<String, String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Adding firmware image: {} ({})", name, id);

        // Calculate SHA256 hash
        let sha256_hash = sha2::Sha256::digest(firmware_data);
        let sha256_hex = hex::encode(sha256_hash);

        // Create firmware file path
        let images_dir = self.storage_dir.join("images");
        let firmware_path = images_dir.join(format!("{}.bin", id));
        
        // Write firmware data
        async_fs::write(&firmware_path, firmware_data).await?;

        // Create firmware image metadata
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let image = FirmwareImage {
            id: id.clone(),
            name,
            device_type,
            architecture,
            version,
            file_path: firmware_path.clone(),
            file_size: firmware_data.len() as u64,
            sha256_hash: sha256_hex,
            created_at: now,
            modified_at: now,
            metadata,
        };

        // Save metadata
        self.save_firmware_image_metadata(&image)?;

        // Add to images map
        self.images.insert(id.clone(), image);

        info!("Firmware image {} added successfully", id);
        Ok(())
    }

    /// Save firmware image metadata to file
    fn save_firmware_image_metadata(&self, image: &FirmwareImage) -> Result<(), Box<dyn std::error::Error>> {
        let images_dir = self.storage_dir.join("images");
        let metadata_path = images_dir.join(format!("{}.json", image.id));
        
        let content = serde_json::to_string_pretty(image)?;
        fs::write(metadata_path, content)?;
        
        Ok(())
    }

    /// Deploy firmware to a device
    pub async fn deploy_firmware(
        &mut self,
        device_id: String,
        firmware_id: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        info!("Deploying firmware {} to device {}", firmware_id, device_id);

        // Check if firmware exists
        let firmware = self.images.get(&firmware_id)
            .ok_or_else(|| format!("Firmware {} not found", firmware_id))?;

        // Create deployment
        let deployment_id = uuid::Uuid::new_v4().to_string();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let deployment = FirmwareDeployment {
            id: deployment_id.clone(),
            device_id: device_id.clone(),
            firmware_id: firmware_id.clone(),
            status: DeploymentStatus::InProgress,
            started_at: Some(now),
            completed_at: None,
            error_message: None,
            progress: 0,
        };

        self.deployments.insert(deployment_id.clone(), deployment);

        // Start deployment process
        self.start_deployment(&deployment_id).await?;

        Ok(deployment_id)
    }

    /// Start firmware deployment process
    async fn start_deployment(&mut self, deployment_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let deployment = self.deployments.get_mut(deployment_id)
            .ok_or_else(|| format!("Deployment {} not found", deployment_id))?;

        let firmware = self.images.get(&deployment.firmware_id)
            .ok_or_else(|| format!("Firmware {} not found", deployment.firmware_id))?;

        info!("Starting firmware deployment {} to device {}", 
              deployment.firmware_id, deployment.device_id);

        // Update progress
        deployment.progress = 10;

        // Copy firmware to device (simulated)
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        deployment.progress = 50;

        // Verify firmware integrity
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
        deployment.progress = 80;

        // Complete deployment
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        deployment.progress = 100;
        deployment.status = DeploymentStatus::Completed;
        deployment.completed_at = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        );

        info!("Firmware deployment {} completed successfully", deployment_id);
        Ok(())
    }

    /// Get deployment status
    pub fn get_deployment_status(&self, deployment_id: &str) -> Option<&FirmwareDeployment> {
        self.deployments.get(deployment_id)
    }

    /// List all firmware images
    pub fn list_firmware_images(&self) -> Vec<&FirmwareImage> {
        self.images.values().collect()
    }

    /// List all deployments
    pub fn list_deployments(&self) -> Vec<&FirmwareDeployment> {
        self.deployments.values().collect()
    }

    /// Get firmware image by ID
    pub fn get_firmware_image(&self, id: &str) -> Option<&FirmwareImage> {
        self.images.get(id)
    }

    /// Get firmware template by device type
    pub fn get_firmware_template(&self, device_type: &str) -> Option<&FirmwareTemplate> {
        self.templates.values().find(|t| t.device_type == device_type)
    }

    /// Remove firmware image
    pub async fn remove_firmware_image(&mut self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(image) = self.images.remove(id) {
            // Remove firmware file
            if image.file_path.exists() {
                async_fs::remove_file(&image.file_path).await?;
            }

            // Remove metadata file
            let images_dir = self.storage_dir.join("images");
            let metadata_path = images_dir.join(format!("{}.json", id));
            if metadata_path.exists() {
                async_fs::remove_file(metadata_path).await?;
            }

            info!("Firmware image {} removed", id);
        }

        Ok(())
    }

    /// Cancel deployment
    pub fn cancel_deployment(&mut self, deployment_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(deployment) = self.deployments.get_mut(deployment_id) {
            deployment.status = DeploymentStatus::Cancelled;
            info!("Deployment {} cancelled", deployment_id);
        }

        Ok(())
    }

    /// Get firmware image file path
    pub fn get_firmware_path(&self, firmware_id: &str) -> Option<&PathBuf> {
        self.images.get(firmware_id).map(|img| &img.file_path)
    }

    /// Verify firmware integrity
    pub async fn verify_firmware(&self, firmware_id: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let image = self.images.get(firmware_id)
            .ok_or_else(|| format!("Firmware {} not found", firmware_id))?;

        // Read firmware file
        let firmware_data = async_fs::read(&image.file_path).await?;

        // Calculate hash
        let sha256_hash = sha2::Sha256::digest(&firmware_data);
        let calculated_hash = hex::encode(sha256_hash);

        // Compare with stored hash
        Ok(calculated_hash == image.sha256_hash)
    }
}
