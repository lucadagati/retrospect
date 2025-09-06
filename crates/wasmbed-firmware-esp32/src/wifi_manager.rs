// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use std::time::{Duration, SystemTime};

use anyhow::Result;
use log::{debug, info, warn};

/// WiFi configuration
#[derive(Debug, Clone)]
pub struct WifiConfig {
    /// SSID of the WiFi network
    pub ssid: String,
    /// Password for the WiFi network
    pub password: String,
    /// Connection timeout
    pub timeout: Duration,
    /// Maximum reconnect attempts
    pub max_reconnect_attempts: u32,
}

impl Default for WifiConfig {
    fn default() -> Self {
        Self {
            ssid: "WasmbedNetwork".to_string(),
            password: "wasmbed123".to_string(),
            timeout: Duration::from_secs(30),
            max_reconnect_attempts: 5,
        }
    }
}

/// WiFi connection status
#[derive(Debug, Clone, PartialEq)]
pub enum WifiStatus {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Error(String),
}

/// WiFi manager for ESP32 devices
pub struct WifiManager {
    /// WiFi configuration
    config: WifiConfig,
    /// Current connection status
    status: WifiStatus,
    /// Current network name
    current_network: Option<String>,
    /// Connection start time
    connection_start_time: Option<SystemTime>,
    /// Number of reconnect attempts
    reconnect_attempts: u32,
    /// Maximum reconnect attempts
    max_reconnect_attempts: u32,
    /// Connection timeout
    timeout: Duration,
}

impl WifiManager {
    /// Create a new WiFi manager
    pub fn new(config: WifiConfig) -> Self {
        Self {
            config,
            status: WifiStatus::Disconnected,
            current_network: None,
            connection_start_time: None,
            reconnect_attempts: 0,
            max_reconnect_attempts: 5,
            timeout: Duration::from_secs(30),
        }
    }

    /// Connect to WiFi network
    pub async fn connect(&mut self) -> Result<()> {
        info!("Connecting to WiFi network: {}", self.config.ssid);
        
        self.status = WifiStatus::Connecting;
        self.connection_start_time = Some(SystemTime::now());
        
        // Simulate WiFi connection process
        tokio::time::sleep(Duration::from_millis(1000)).await;
        
        // For simulation purposes, always succeed
        self.status = WifiStatus::Connected;
        self.current_network = Some(self.config.ssid.clone());
        self.reconnect_attempts = 0;
        
        info!("Connected to WiFi network: {}", self.config.ssid);
        Ok(())
    }

    /// Disconnect from WiFi network
    pub async fn disconnect(&mut self) -> Result<()> {
        info!("Disconnecting from WiFi network");
        
        self.status = WifiStatus::Disconnected;
        self.current_network = None;
        self.connection_start_time = None;
        
        info!("Disconnected from WiFi network");
        Ok(())
    }

    /// Check if WiFi is connected
    pub fn is_connected(&self) -> bool {
        self.status == WifiStatus::Connected
    }

    /// Get current WiFi status
    pub fn get_status(&self) -> &WifiStatus {
        &self.status
    }

    /// Get current network name
    pub fn get_current_network(&self) -> Option<&String> {
        self.current_network.as_ref()
    }

    /// Get signal strength (simulated)
    pub fn get_signal_strength(&self) -> i32 {
        if self.is_connected() {
            -45 // Good signal strength
        } else {
            -100 // No signal
        }
    }

    /// Get IP address (simulated)
    pub fn get_ip_address(&self) -> Option<String> {
        if self.is_connected() {
            Some("192.168.1.100".to_string()) // Simulated IP
        } else {
            None
        }
    }

    /// Get MAC address (simulated)
    pub fn get_mac_address(&self) -> String {
        "AA:BB:CC:DD:EE:FF".to_string() // Simulated MAC
    }

    /// Scan for available networks
    pub async fn scan_networks(&self) -> Result<Vec<WifiNetwork>> {
        debug!("Scanning for WiFi networks");
        
        // Simulate network scan
        let networks = vec![
            WifiNetwork {
                ssid: "WasmbedNetwork".to_string(),
                signal_strength: -45,
                security: WifiSecurity::WPA2,
                frequency: 2412,
            },
            WifiNetwork {
                ssid: "GuestNetwork".to_string(),
                signal_strength: -60,
                security: WifiSecurity::Open,
                frequency: 2437,
            },
            WifiNetwork {
                ssid: "OfficeWiFi".to_string(),
                signal_strength: -70,
                security: WifiSecurity::WPA3,
                frequency: 2462,
            },
        ];
        
        Ok(networks)
    }

    /// Reconnect to WiFi
    pub async fn reconnect(&mut self) -> Result<()> {
        if self.reconnect_attempts >= self.max_reconnect_attempts {
            self.status = WifiStatus::Error("Max reconnect attempts reached".to_string());
            return Err(anyhow::anyhow!("Max reconnect attempts reached"));
        }

        self.reconnect_attempts += 1;
        self.status = WifiStatus::Reconnecting;
        
        info!("Reconnecting to WiFi (attempt {}/{})", 
              self.reconnect_attempts, self.max_reconnect_attempts);
        
        // Disconnect first
        self.disconnect().await?;
        
        // Wait a bit before reconnecting
        tokio::time::sleep(Duration::from_secs(2)).await;
        
        // Try to reconnect
        self.connect().await?;
        
        Ok(())
    }

    /// Update WiFi configuration
    pub fn update_config(&mut self, config: WifiConfig) {
        self.config = config;
        info!("WiFi configuration updated");
    }

    /// Get connection duration
    pub fn get_connection_duration(&self) -> Option<Duration> {
        if let Some(start_time) = self.connection_start_time {
            start_time.elapsed().ok()
        } else {
            None
        }
    }

    /// Get reconnect attempts count
    pub fn get_reconnect_attempts(&self) -> u32 {
        self.reconnect_attempts
    }

    /// Reset reconnect attempts
    pub fn reset_reconnect_attempts(&mut self) {
        self.reconnect_attempts = 0;
    }

    /// Check if connection is stable
    pub fn is_connection_stable(&self) -> bool {
        if let Some(duration) = self.get_connection_duration() {
            duration > Duration::from_secs(30) // Stable if connected for more than 30 seconds
        } else {
            false
        }
    }

    /// Get WiFi statistics
    pub fn get_statistics(&self) -> WifiStatistics {
        WifiStatistics {
            status: self.status.clone(),
            current_network: self.current_network.clone(),
            signal_strength: self.get_signal_strength(),
            ip_address: self.get_ip_address(),
            mac_address: self.get_mac_address(),
            connection_duration: self.get_connection_duration(),
            reconnect_attempts: self.reconnect_attempts,
            is_stable: self.is_connection_stable(),
        }
    }
}

/// WiFi network information
#[derive(Debug, Clone)]
pub struct WifiNetwork {
    /// Network SSID
    pub ssid: String,
    /// Signal strength in dBm
    pub signal_strength: i32,
    /// Security type
    pub security: WifiSecurity,
    /// Frequency in MHz
    pub frequency: u32,
}

/// WiFi security types
#[derive(Debug, Clone, PartialEq)]
pub enum WifiSecurity {
    Open,
    WEP,
    WPA,
    WPA2,
    WPA3,
}

/// WiFi statistics
#[derive(Debug, Clone)]
pub struct WifiStatistics {
    /// Current status
    pub status: WifiStatus,
    /// Current network name
    pub current_network: Option<String>,
    /// Signal strength in dBm
    pub signal_strength: i32,
    /// IP address
    pub ip_address: Option<String>,
    /// MAC address
    pub mac_address: String,
    /// Connection duration
    pub connection_duration: Option<Duration>,
    /// Number of reconnect attempts
    pub reconnect_attempts: u32,
    /// Whether connection is stable
    pub is_stable: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wifi_manager_creation() {
        let config = WifiConfig::default();
        let wifi_manager = WifiManager::new(config);
        assert_eq!(wifi_manager.get_status(), &WifiStatus::Disconnected);
        assert!(!wifi_manager.is_connected());
    }

    #[test]
    fn test_wifi_config_default() {
        let config = WifiConfig::default();
        assert_eq!(config.ssid, "WasmbedNetwork");
        assert_eq!(config.password, "wasmbed123");
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.max_reconnect_attempts, 5);
    }

    #[tokio::test]
    async fn test_wifi_connection() {
        let config = WifiConfig::default();
        let mut wifi_manager = WifiManager::new(config);
        
        assert!(!wifi_manager.is_connected());
        
        let result = wifi_manager.connect().await;
        assert!(result.is_ok());
        assert!(wifi_manager.is_connected());
        assert_eq!(wifi_manager.get_status(), &WifiStatus::Connected);
    }

    #[tokio::test]
    async fn test_wifi_disconnection() {
        let config = WifiConfig::default();
        let mut wifi_manager = WifiManager::new(config);
        
        wifi_manager.connect().await.unwrap();
        assert!(wifi_manager.is_connected());
        
        wifi_manager.disconnect().await.unwrap();
        assert!(!wifi_manager.is_connected());
        assert_eq!(wifi_manager.get_status(), &WifiStatus::Disconnected);
    }

    #[tokio::test]
    async fn test_wifi_scan() {
        let config = WifiConfig::default();
        let wifi_manager = WifiManager::new(config);
        
        let networks = wifi_manager.scan_networks().await.unwrap();
        assert!(!networks.is_empty());
        
        let first_network = &networks[0];
        assert_eq!(first_network.ssid, "WasmbedNetwork");
    }

    #[test]
    fn test_wifi_statistics() {
        let config = WifiConfig::default();
        let wifi_manager = WifiManager::new(config);
        
        let stats = wifi_manager.get_statistics();
        assert_eq!(stats.status, WifiStatus::Disconnected);
        assert_eq!(stats.mac_address, "AA:BB:CC:DD:EE:FF");
        assert_eq!(stats.reconnect_attempts, 0);
    }
}