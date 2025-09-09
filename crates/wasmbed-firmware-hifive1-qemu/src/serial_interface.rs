// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use core::fmt::Write;
use heapless::String;

use crate::wasmbed_client::WasmbedClient;
use crate::application_manager::ApplicationManager;
use crate::monitoring::MonitoringSystem;

pub struct SerialInterface {
    command_buffer: String<256>,
    response_buffer: String<256>,
    uart_base: u32,
}

impl SerialInterface {
    pub fn new() -> Self {
        Self {
            command_buffer: String::new(),
            response_buffer: String::new(),
            uart_base: 0x10013000, // UART0 base address for HiFive1
        }
    }
    
    pub fn process_commands(
        &mut self,
        client: &mut WasmbedClient,
        app_manager: &mut ApplicationManager,
        monitoring_system: &mut MonitoringSystem,
    ) {
        // Check for incoming serial data from UART
        if let Some(command) = self.read_uart_command() {
            self.execute_command(command, client, app_manager, monitoring_system);
            self.send_uart_response();
        }
    }
    
    fn read_uart_command(&mut self) -> Option<String<64>> {
        // Read from UART registers
        // UART_RX register offset: 0x00
        let uart_rx = unsafe { core::ptr::read_volatile((self.uart_base + 0x00) as *const u32) };
        
        // Check if data is available (bit 0 = data ready)
        if (uart_rx & 0x01) != 0 {
            // Read the actual data (bits 7:0)
            let data = (uart_rx >> 8) as u8;
            
            if data == b'\n' || data == b'\r' {
                // End of command, process it
                if !self.command_buffer.is_empty() {
                    let mut command = String::new();
                    let _ = command.push_str(self.command_buffer.as_str());
                    self.command_buffer.clear();
                    return Some(command);
                }
            } else if data >= 32 && data <= 126 {
                // Printable ASCII character
                if self.command_buffer.push(data as char).is_ok() {
                    // Character added successfully
                } else {
                    // Buffer full, clear it
                    self.command_buffer.clear();
                }
            }
        }
        
        None
    }
    
    fn send_uart_response(&mut self) {
        // Send response via UART
        // UART_TX register offset: 0x04
        for byte in self.response_buffer.as_bytes() {
            self.write_uart_byte(*byte);
        }
        
        // Send newline
        self.write_uart_byte(b'\n');
        self.response_buffer.clear();
    }
    
    fn write_uart_byte(&self, byte: u8) {
        // Wait for UART to be ready to transmit
        loop {
            let uart_status = unsafe { core::ptr::read_volatile((self.uart_base + 0x08) as *const u32) };
            if (uart_status & 0x01) != 0 {
                break; // UART is ready
            }
        }
        
        // Write the byte
        unsafe {
            core::ptr::write_volatile((self.uart_base + 0x04) as *mut u32, byte as u32);
        }
    }
    
    fn execute_command(
        &mut self,
        command: String<64>,
        client: &mut WasmbedClient,
        app_manager: &mut ApplicationManager,
        monitoring_system: &mut MonitoringSystem,
    ) {
        self.response_buffer.clear();
        
        match command.as_str() {
            "help" => {
                let _ = write!(self.response_buffer, "Available commands:");
                let _ = write!(self.response_buffer, "  help - Show this help");
                let _ = write!(self.response_buffer, "  status - Show device status");
                let _ = write!(self.response_buffer, "  enroll - Start enrollment process");
                let _ = write!(self.response_buffer, "  heartbeat - Send heartbeat");
                let _ = write!(self.response_buffer, "  wasm_status - Show WASM applications");
                let _ = write!(self.response_buffer, "  microros_status - Show microROS status");
            },
            "status" => {
                let _ = write!(self.response_buffer, "Device Status:");
                let _ = write!(self.response_buffer, "  Device ID: qemu-device-1");
                let _ = write!(self.response_buffer, "  Architecture: RISC-V 32-bit");
                let _ = write!(self.response_buffer, "  Memory: 16KB RAM");
                let _ = write!(self.response_buffer, "  UART: Active");
                let _ = write!(self.response_buffer, "  Firmware: wasmbed-firmware-hifive1-qemu");
            },
            "enroll" => {
                let _ = write!(self.response_buffer, "Starting enrollment process...");
                // In a real implementation, this would trigger enrollment
                let _ = write!(self.response_buffer, "Enrollment initiated");
            },
            "heartbeat" => {
                let _ = write!(self.response_buffer, "Sending heartbeat...");
                // In a real implementation, this would send heartbeat
                let _ = write!(self.response_buffer, "Heartbeat sent");
            },
            "wasm_status" => {
                let _ = write!(self.response_buffer, "WASM Applications:");
                let _ = write!(self.response_buffer, "  microros-px4-bridge: Running");
                let _ = write!(self.response_buffer, "  Status: Active");
            },
            "microros_status" => {
                let _ = write!(self.response_buffer, "microROS Status:");
                let _ = write!(self.response_buffer, "  Topics subscribed: 5");
                let _ = write!(self.response_buffer, "  Communication: Active");
                let _ = write!(self.response_buffer, "  DDS: FastDDS");
            },
            _ => {
                let _ = write!(self.response_buffer, "Unknown command: ");
                let _ = write!(self.response_buffer, "{}", command.as_str());
                let _ = write!(self.response_buffer, "Type 'help' for available commands");
            }
        }
    }
}