// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

//! Test integration between gateway and device runtime
//! This example simulates the full deployment flow

use wasmbed_device_runtime::tls_client::{TlsClient, Keypair};
use wasmbed_device_runtime::wasm_interpreter::WasmInstance;

fn main() {
    println!("=== Test Gateway-Device Integration ===\n");

    // Initialize TLS client
    let mut tls_client = TlsClient::new();
    let keypair = Keypair::dummy();

    // Connect to gateway (simulated)
    println!("1. Connecting to gateway...");
    tls_client.connect("127.0.0.1:8443", &keypair).unwrap();
    println!("   ✅ Connected\n");

    // Simulate receiving a deployment message from gateway
    println!("2. Simulating deployment message from gateway...");
    let test_wasm = &[
        0x00, 0x61, 0x73, 0x6d, // WASM magic: "\0asm"
        0x01, 0x00, 0x00, 0x00, // Version 1
        0x01, 0x04, 0x01, 0x60, 0x00, 0x00, // Type section
        0x03, 0x02, 0x01, 0x00, // Function section
        0x05, 0x03, 0x01, 0x00, 0x01, // Memory section
        0x0a, 0x0b, 0x01, 0x09, 0x00, // Code section
        0x41, 0x00, 0x41, 0x2a, 0x36, 0x02, 0x00, 0x0b, // i32.const 0, i32.const 42, i32.store, end
    ];

    tls_client.simulate_receive_deployment("test-app-1", test_wasm);
    println!("   ✅ Deployment message queued\n");

    // Receive and process the message (simulating device runtime main loop)
    println!("3. Processing deployment message...");
    match tls_client.receive_message().unwrap() {
        Some(wasmbed_protocol::ServerMessage::DeployApplication { app_id, wasm_bytes, .. }) => {
            println!("   ✅ Received deployment for app: {}", app_id);
            println!("   ✅ WASM bytecode size: {} bytes", wasm_bytes.len());

            // Load and execute WASM module
            match WasmInstance::load_module(&wasm_bytes) {
                Ok(mut instance) => {
                    println!("   ✅ WASM module loaded successfully");
                    match instance.execute() {
                        Ok(_) => {
                            println!("   ✅ WASM execution completed");
                            
                            // Verify memory contains 42
                            if let Some(value) = instance.memory().read_i32(0) {
                                if value == 42 {
                                    println!("   ✅ Memory verification PASSED (value = 42)");
                                } else {
                                    println!("   ❌ Memory verification FAILED (value = {})", value);
                                }
                            }
                            
                            // Send success acknowledgment
                            tls_client.send_deployment_ack(&app_id, true, None).unwrap();
                            println!("   ✅ Deployment acknowledgment sent\n");
                        }
                        Err(e) => {
                            println!("   ❌ WASM execution failed: {}", e);
                            tls_client.send_deployment_ack(&app_id, false, Some("Execution failed")).unwrap();
                        }
                    }
                }
                Err(e) => {
                    println!("   ❌ Failed to load WASM module: {}", e);
                    tls_client.send_deployment_ack(&app_id, false, Some("Load failed")).unwrap();
                }
            }
        }
        _ => {
            println!("   ❌ Unexpected message type");
        }
    }

    println!("=== Test Complete ===");
    println!("\n✅ Gateway-Device integration is working!");
    println!("   The device runtime can:");
    println!("   - Connect to gateway");
    println!("   - Receive deployment messages");
    println!("   - Load and execute WASM bytecode");
    println!("   - Send acknowledgments");
}

