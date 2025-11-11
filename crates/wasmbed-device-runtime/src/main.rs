// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), no_main)]
// No unstable features needed with panic = "abort"

#[cfg(not(feature = "std"))]
mod getrandom_custom;

#[cfg(not(feature = "std"))]
mod allocator;

#[cfg(not(feature = "std"))]
mod startup;

mod wasm_interpreter;
mod tls_client;
mod tls_io;

use wasm_interpreter::WasmInstance;
use tls_client::{TlsClient, Keypair};
use log::{error, info, warn};
use wasmbed_protocol::ServerMessage;
use core::fmt::Write;

/// Gateway endpoint memory address (written by Renode)
const GATEWAY_ENDPOINT_ADDR: usize = 0x20001000;

/// Read gateway endpoint from memory
/// Format: [length: u32 at 0x20001000][bytes at 0x20001004...]
fn read_gateway_endpoint_from_memory() -> heapless::String<128> {
    unsafe {
        // Read length (first 4 bytes)
        let length_ptr = GATEWAY_ENDPOINT_ADDR as *const u32;
        let length = *length_ptr as usize;
        
        if length == 0 || length > 127 {
            return heapless::String::new();
        }
        
        // Read endpoint string bytes
        let bytes_ptr = (GATEWAY_ENDPOINT_ADDR + 4) as *const u8;
        let mut endpoint = heapless::String::<128>::new();
        
        for i in 0..length {
            let byte = *bytes_ptr.add(i);
            if byte == 0 {
                break; // Null terminator
            }
            if endpoint.push(char::from(byte)).is_err() {
                break; // String full
            }
        }
        
        endpoint
    }
}

#[cfg(not(feature = "std"))]
#[no_mangle]
pub extern "C" fn main() -> i32 {
    // Initialize UART first (before allocator) so we can see logs
    unsafe {
        uart_init();
    }
    
    // Initialize global allocator (required for wasmparser with alloc feature)
    unsafe {
        allocator::init();
    }
    
    // Initialize logging (in no_std, logs go to UART in Renode)
    let _ = log::set_logger(&SimpleLogger);
    log::set_max_level(log::LevelFilter::Info);

    info!("Device runtime starting...");

    // Read gateway endpoint from memory (written by Renode at 0x20001000)
    // Format: [length: u32][bytes...]
    let gateway_endpoint = read_gateway_endpoint_from_memory();
    if gateway_endpoint.is_empty() {
        warn!("Gateway endpoint from memory is empty! Using default 127.0.0.1:8443");
    } else {
        info!("Gateway endpoint from memory: {} (length: {})", gateway_endpoint, gateway_endpoint.len());
    }

    // Initialize TLS client with real TLS enabled
    // Use new() which creates a client with use_real_tls=false by default
    // For no_std, we'll use simulated connection (MemoryIo)
    let mut tls_client = TlsClient::new();
    // Create a dummy keypair for TLS connection
    // In production, this would be generated or loaded from secure storage
    let mut public_key = heapless::Vec::<u8, 256>::new();
    let mut private_key = heapless::Vec::<u8, 256>::new();
    // Initialize with zeros (dummy keys)
    for _ in 0..32 {
        let _ = public_key.push(0);
        let _ = private_key.push(0);
    }
    let keypair = Keypair {
        public_key,
        private_key,
    };

    // Connect to TCP bridge endpoint
    // For testing, use hardcoded bridge endpoint
    let endpoint_str = if gateway_endpoint.is_empty() {
        "127.0.0.1:40483"  // Default bridge port
    } else {
        gateway_endpoint.as_str()
    };
    
    info!("Connecting to TCP bridge at: {}", endpoint_str);
    info!("TLS client initialized, attempting connection...");
    let mut retry_count = 0;
    let max_retries = 5;
    loop {
        info!("Connection attempt #{} to {}", retry_count + 1, endpoint_str);
        match tls_client.connect(endpoint_str, &keypair) {
            Ok(_) => {
                info!("✅ Successfully connected to gateway at {}", endpoint_str);
                info!("Entering main loop for message processing...");
                break;
            }
            Err(e) => {
                retry_count += 1;
                error!("Failed to connect to gateway: {} (attempt {}/{})", e, retry_count, max_retries);
                
                if retry_count >= max_retries {
                    error!("❌ Max connection retries reached. Exiting.");
                    return 1;
                }
                
                // Wait before retrying
                info!("Waiting 2 seconds before retry...");
                for _ in 0..2000000 {
                    core::hint::spin_loop();
                }
            }
        }
    }
    
    // Main loop: receive and process messages from gateway
    let mut heartbeat_counter: u32 = 0;
    loop {
        // Check for incoming messages
        match tls_client.receive_message() {
            Ok(Some(message)) => {
                match message {
                    ServerMessage::DeployApplication { app_id, wasm_bytes, .. } => {
                        info!("Received deployment request for app: {}", app_id);
                        
                        // Load and execute WASM module
                        match WasmInstance::load_module(&wasm_bytes) {
                            Ok(mut instance) => {
                                info!("WASM module loaded successfully");
                                match instance.execute() {
                                    Ok(_) => {
                                        info!("WASM execution completed successfully");
                                        // Send success acknowledgment
                                        let _ = tls_client.send_deployment_ack(&app_id, true, None);
                                    }
                                    Err(e) => {
                                        error!("WASM execution failed: {}", e);
                                        // Send failure acknowledgment (error message truncated for no_std)
                                        let _ = tls_client.send_deployment_ack(&app_id, false, Some("Execution failed"));
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to load WASM module: {}", e);
                                // Send failure acknowledgment (error message truncated for no_std)
                                let _ = tls_client.send_deployment_ack(&app_id, false, Some("Load failed"));
                }
                        }
                    }
                    ServerMessage::StopApplication { app_id } => {
                    info!("Received stop request for app: {}", app_id);
                        // TODO: Implement application stopping
                        let _ = tls_client.send_stop_ack(&app_id, true, None);
                }
                    ServerMessage::HeartbeatAck => {
                    // Heartbeat acknowledged
                }
                _ => {
                        warn!("Received unhandled message type");
                }
            }
        }
            Ok(None) => {
                // No messages available, continue
            }
            Err(e) => {
                error!("Error receiving message: {}", e);
            }
        }
        
        // Send periodic heartbeat (every ~1000 iterations)
        heartbeat_counter = heartbeat_counter.wrapping_add(1);
        if heartbeat_counter % 1000 == 0 {
            let _ = tls_client.send_heartbeat();
        }
        
        // Small delay to prevent busy-waiting
        // In real implementation, would use proper sleep or interrupt-driven I/O
        for _ in 0..1000 {
            core::hint::spin_loop();
        }
    }
}

#[cfg(feature = "std")]
fn main() {
    // Initialize logging for std builds
    log::set_logger(&StdLogger).unwrap();
    log::set_max_level(log::LevelFilter::Info);

    info!("Device runtime starting with TLS support...");

    // Initialize TLS client
    let mut tls_client = TlsClient::new();
    let mut public_key = heapless::Vec::<u8, 256>::new();
    let mut private_key = heapless::Vec::<u8, 256>::new();
    for _ in 0..32 {
        let _ = public_key.push(0);
        let _ = private_key.push(0);
    }
    let keypair = Keypair {
        public_key,
        private_key,
    };

    // Connect directly to gateway TLS (bypassing TCP bridge for testing)
    let endpoint = "127.0.0.1:8443";
    
    info!("Connecting to TCP bridge at: {}", endpoint);
    match tls_client.connect(endpoint, &keypair) {
        Ok(_) => {
            info!("✅ Successfully connected to gateway via TCP bridge!");
            
            // === ENROLLMENT PROTOCOL ===
            info!("Starting enrollment process...");
            
            // Step 1: Send enrollment request
            if let Err(e) = tls_client.send_enrollment_request() {
                error!("Failed to send enrollment request: {}", e);
                std::process::exit(1);
            }
            
            // Step 2: Wait for enrollment accepted
            info!("Waiting for enrollment acceptance...");
            let mut enrollment_accepted = false;
            let mut device_uuid: Option<String> = None;
            
            for _ in 0..50 {  // Wait up to 5 seconds
                match tls_client.receive_message() {
                    Ok(Some(ServerMessage::EnrollmentAccepted)) => {
                        info!("✅ Enrollment accepted!");
                        enrollment_accepted = true;
                        break;
                    }
                    Ok(Some(ServerMessage::EnrollmentRejected { reason })) => {
                        let reason_str = String::from_utf8_lossy(&reason);
                        error!("❌ Enrollment rejected: {}", reason_str);
                        std::process::exit(1);
                    }
                    Ok(Some(msg)) => {
                        warn!("Unexpected message during enrollment: {:?}", msg);
                    }
                    Ok(None) => {
                        // No message yet, continue waiting
                    }
                    Err(e) => {
                        error!("Error receiving enrollment response: {}", e);
                        std::process::exit(1);
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            
            if !enrollment_accepted {
                error!("❌ Enrollment timeout - did not receive EnrollmentAccepted");
                std::process::exit(1);
            }
            
            // Step 3: Send public key (using dummy key for now)
            let public_key_bytes = keypair.public_key.as_slice();
            if let Err(e) = tls_client.send_public_key(public_key_bytes) {
                error!("Failed to send public key: {}", e);
                std::process::exit(1);
            }
            
            // Step 4: Wait for device UUID
            info!("Waiting for device UUID...");
            for _ in 0..50 {  // Wait up to 5 seconds
                match tls_client.receive_message() {
                    Ok(Some(ServerMessage::DeviceUuid { uuid })) => {
                        // Format UUID as standard format: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
                        let mut uuid_str = String::new();
                        for (i, &byte) in uuid.bytes.iter().enumerate() {
                            if i == 4 || i == 6 || i == 8 || i == 10 {
                                uuid_str.push('-');
                            }
                            uuid_str.push_str(&format!("{:02x}", byte));
                        }
                        // UUID format: 8-4-4-4-12 hex chars = bytes at positions: 0-3, 4-5, 6-7, 8-9, 10-15
                        // So dashes go after bytes 3, 5, 7, 9 (indices 4, 6, 8, 10)
                        info!("✅ Received device UUID: {}", uuid_str);
                        device_uuid = Some(uuid_str);
                        break;
                    }
                    Ok(Some(msg)) => {
                        warn!("Unexpected message while waiting for UUID: {:?}", msg);
                    }
                    Ok(None) => {
                        // Continue waiting
                    }
                    Err(e) => {
                        error!("Error receiving device UUID: {}", e);
                        std::process::exit(1);
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            
            if device_uuid.is_none() {
                error!("❌ Timeout waiting for device UUID");
                std::process::exit(1);
            }
            
            // Step 5: Send enrollment acknowledgment
            if let Err(e) = tls_client.send_enrollment_ack() {
                error!("Failed to send enrollment ack: {}", e);
                std::process::exit(1);
            }
            
            // Step 6: Wait for enrollment completed
            info!("Waiting for enrollment completion...");
            for _ in 0..50 {
                match tls_client.receive_message() {
                    Ok(Some(ServerMessage::EnrollmentCompleted)) => {
                        info!("🎉 Enrollment completed successfully!");
                        break;
                    }
                    Ok(Some(msg)) => {
                        warn!("Unexpected message: {:?}", msg);
                    }
                    Ok(None) => {
                        // Continue waiting
                    }
                    Err(e) => {
                        error!("Error receiving enrollment completion: {}", e);
                        break;
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            
            info!("✅ Device enrolled! Device UUID: {:?}", device_uuid);
            info!("Starting normal operation mode...");
            
            // Main loop: receive and process messages
            info!("Entering main loop, waiting for messages...");
            let mut message_count = 0;
            let max_iterations = 1000;  // Run for 100 seconds
            
            for _ in 0..max_iterations {
                match tls_client.receive_message() {
                    Ok(Some(message)) => {
                        message_count += 1;
                        info!("📨 Received message #{}: {:?}", message_count, message);
                        
                        match message {
                            ServerMessage::DeployApplication { app_id, wasm_bytes, .. } => {
                                info!("🚀 Deploying application: {} ({} bytes)", app_id, wasm_bytes.len());
                                
                                // Load and execute WASM
                                match WasmInstance::load_module(&wasm_bytes) {
                                    Ok(mut instance) => {
                                        info!("WASM module loaded successfully");
                                        match instance.execute() {
                                            Ok(_) => {
                                                info!("✅ WASM execution completed successfully");
                                                let _ = tls_client.send_deployment_ack(&app_id, true, None);
                                            }
                                            Err(e) => {
                                                error!("❌ WASM execution failed: {}", e);
                                                let _ = tls_client.send_deployment_ack(&app_id, false, Some("Execution failed"));
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        error!("❌ Failed to load WASM module: {}", e);
                                        let _ = tls_client.send_deployment_ack(&app_id, false, Some("Load failed"));
                                    }
                                }
                            }
                            ServerMessage::StopApplication { app_id } => {
                                info!("🛑 Stopping application: {}", app_id);
                                let _ = tls_client.send_stop_ack(&app_id, true, None);
                            }
                            ServerMessage::HeartbeatAck => {
                                info!("💓 Heartbeat acknowledged");
                            }
                            _ => {
                                warn!("❓ Received unhandled message type");
                            }
                        }
                    }
                    Ok(None) => {
                        // No messages, continue
                    }
                    Err(e) => {
                        error!("Error receiving message: {}", e);
                        break;
                    }
                }
                
                // Send periodic heartbeat
                if let Err(e) = tls_client.send_heartbeat() {
                    error!("Failed to send heartbeat: {}", e);
                }
                
                // Small delay
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            
            info!("Main loop completed after {} iterations, {} messages received", max_iterations, message_count);
        }
        Err(e) => {
            error!("❌ Failed to connect to gateway: {}", e);
            std::process::exit(1);
        }
    }
    
    info!("Device runtime test completed");
}

/// UART base address for nRF52840 UART0
const UART0_BASE: usize = 0x40002000;

/// UART register offsets
const UART_ENABLE: usize = 0x500;
const UART_TXD: usize = 0x51C;
const UART_TXDRDY: usize = 0x10C;
const UART_PSELTXD: usize = 0x50C;
const UART_BAUDRATE: usize = 0x524;
const UART_TASKS_STARTTX: usize = 0x008;

/// Static flag to track UART initialization
static mut UART_INITIALIZED: bool = false;

/// Initialize UART0 (call once at startup)
unsafe fn uart_init() {
    if UART_INITIALIZED {
        return;
    }
    
    // Enable UART
    let enable_reg = (UART0_BASE + UART_ENABLE) as *mut u32;
    *enable_reg = 1;
    
    // Configure TXD pin (pin 6 for Arduino Nano 33 BLE)
    let pseltxd_reg = (UART0_BASE + UART_PSELTXD) as *mut u32;
    *pseltxd_reg = 6;
    
    // Set baudrate to 115200 (0x01D7E000)
    let baudrate_reg = (UART0_BASE + UART_BAUDRATE) as *mut u32;
    *baudrate_reg = 0x01D7E000;
    
    // Start TX task
    let starttx_reg = (UART0_BASE + UART_TASKS_STARTTX) as *mut u32;
    *starttx_reg = 1;
    
    UART_INITIALIZED = true;
}

/// Write a byte to UART0
/// This is a simple implementation that writes directly to UART registers
/// In Renode, this will appear in the UART analyzer
unsafe fn uart_write_byte(byte: u8) {
    // Initialize UART on first use
    uart_init();
    
    // Write byte to TXD register
    let txd_reg = (UART0_BASE + UART_TXD) as *mut u8;
    *txd_reg = byte;
    
    // Wait for TXDRDY (transmission ready)
    let txdrdy_reg = (UART0_BASE + UART_TXDRDY) as *mut u32;
    // Simple busy wait - in real hardware this would use interrupts
    for _ in 0..1000 {
        if *txdrdy_reg != 0 {
            *txdrdy_reg = 0; // Clear event
            break;
        }
    }
}

/// Write a string to UART0
unsafe fn uart_write_str(s: &str) {
    for byte in s.bytes() {
        uart_write_byte(byte);
    }
}

/// UART writer for formatting (implements core::fmt::Write)
struct UartWriter;

impl Write for UartWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        unsafe {
            uart_write_str(s);
        }
        Ok(())
    }
}

/// Simple logger implementation for no_std
/// Writes to UART0 in Renode (visible in UART analyzer)
struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        // Format log message without allocation using heapless::String
        let level = match record.level() {
            log::Level::Error => "ERROR",
            log::Level::Warn => "WARN",
            log::Level::Info => "INFO",
            log::Level::Debug => "DEBUG",
            log::Level::Trace => "TRACE",
        };
        
        // Use heapless::String to format message (max 256 bytes)
        let mut buffer = heapless::String::<256>::new();
        let _ = write!(buffer, "[{}] ", level);
        let _ = write!(buffer, "{}", record.args());
        let _ = write!(buffer, "\n");
        
        // Write formatted message to UART
        unsafe {
            uart_write_str(buffer.as_str());
        }
    }

    fn flush(&self) {
        // UART is unbuffered, so flush is a no-op
    }
}

#[cfg(feature = "std")]
struct StdLogger;

#[cfg(feature = "std")]
impl log::Log for StdLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        use std::io::{self, Write};
        let _ = writeln!(io::stderr(), "[{}] {}", record.level(), record.args());
    }

    fn flush(&self) {}
}
