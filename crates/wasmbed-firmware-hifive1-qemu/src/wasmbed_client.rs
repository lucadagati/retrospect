// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

/// Configuration for the Wasmbed client
pub struct ClientConfig {
    /// Gateway server address (for now, just a placeholder)
    pub gateway_address: &'static str,
    /// Heartbeat interval in seconds
    pub heartbeat_interval: u64,
    /// Reconnection delay in seconds
    pub reconnect_delay: u64,
    /// Maximum reconnection attempts
    pub max_reconnect_attempts: u32,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            gateway_address: "172.19.0.2:30423",
            heartbeat_interval: 30,
            reconnect_delay: 5,
            max_reconnect_attempts: 10,
        }
    }
}

/// Main Wasmbed client for MCU devices (simplified version)
pub struct WasmbedClient {
    config: ClientConfig,
    connection_state: ConnectionState,
    heartbeat_counter: u32,
    reconnect_attempts: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Enrolling,
    Enrolled,
}

impl WasmbedClient {
    /// Create a new Wasmbed client with default configuration
    pub fn new() -> Self {
        Self {
            config: ClientConfig::default(),
            connection_state: ConnectionState::Disconnected,
            heartbeat_counter: 0,
            reconnect_attempts: 0,
        }
    }

    /// Create a new Wasmbed client with custom configuration
    pub fn with_config(config: ClientConfig) -> Self {
        Self {
            config,
            connection_state: ConnectionState::Disconnected,
            heartbeat_counter: 0,
            reconnect_attempts: 0,
        }
    }

    /// Simulate client operations (non-async version)
    pub fn simulate_operation(&mut self) {
        match self.connection_state {
            ConnectionState::Disconnected => {
                self.simulate_connect();
            }
            ConnectionState::Connecting => {
                self.simulate_connecting();
            }
            ConnectionState::Connected => {
                self.simulate_connected();
            }
            ConnectionState::Enrolling => {
                self.simulate_enrolling();
            }
            ConnectionState::Enrolled => {
                self.simulate_enrolled();
            }
        }
    }

    /// Simulate connection attempt
    fn simulate_connect(&mut self) {
        if self.reconnect_attempts >= self.config.max_reconnect_attempts {
            return;
        }
        
        self.reconnect_attempts += 1;
        
        // Simulate successful connection
        self.connection_state = ConnectionState::Connected;
    }

    /// Simulate connecting state
    fn simulate_connecting(&mut self) {
        // Still connecting
    }

    /// Simulate connected state
    fn simulate_connected(&mut self) {
        self.connection_state = ConnectionState::Enrolling;
    }

    /// Simulate enrolling state
    fn simulate_enrolling(&mut self) {
        // Simulate enrollment completion
        self.connection_state = ConnectionState::Enrolled;
    }

    /// Simulate enrolled state
    fn simulate_enrolled(&mut self) {
        self.heartbeat_counter += 1;
        
        if u64::from(self.heartbeat_counter) >= self.config.heartbeat_interval {
            self.simulate_heartbeat();
            self.heartbeat_counter = 0;
        }
    }

    /// Simulate heartbeat
    fn simulate_heartbeat(&mut self) {
        // Simulate heartbeat sent
    }

    /// Simulate disconnection
    fn simulate_disconnection(&mut self) {
        self.connection_state = ConnectionState::Disconnected;
    }
}
