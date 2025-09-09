// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use heapless::String;

pub struct WifiManager {
    connected: bool,
    ssid: String<32>,
}

impl WifiManager {
    pub fn new(_wifi: esp32_hal::wifi::Wifi, _radio_clk: esp32_hal::clock::RadioClockControl) -> Self {
        Self {
            connected: false,
            ssid: String::new(),
        }
    }
    
    pub fn connect(&mut self, ssid: &str, _password: &str) -> Result<(), &'static str> {
        // Simulate WiFi connection
        self.ssid.clear();
        let _ = self.ssid.push_str(ssid);
        self.connected = true;
        Ok(())
    }
    
    pub fn is_connected(&self) -> bool {
        self.connected
    }
    
    pub fn get_ssid(&self) -> &str {
        self.ssid.as_str()
    }
}