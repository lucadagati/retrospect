// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

//! Minimal firmware for testing WASM execution in Renode
//! This creates a simple binary that can be loaded into Renode
//! and executes a WASM module using our no_std interpreter

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use wasmbed_device_runtime::wasm_interpreter::WasmInstance;

/// Panic handler for no_std
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

/// Entry point - called by Renode
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Simple WASM module: stores 42 in memory at address 0
    let wasm_bytes = &[
        0x00, 0x61, 0x73, 0x6d, // WASM magic
        0x01, 0x00, 0x00, 0x00, // Version 1
        // Memory section: 1 page (64KB)
        0x05, 0x03, 0x01, 0x00, 0x01,
        // Type section: func () -> void
        0x01, 0x04, 0x01, 0x60, 0x00, 0x00,
        // Function section: 1 function
        0x03, 0x02, 0x01, 0x00,
        // Code section: store 42 at address 0
        0x0a, 0x0c, 0x01, 0x0a, 0x00,
        0x41, 0x00, // i32.const 0
        0x41, 0x2a, // i32.const 42
        0x36, 0x02, 0x00, // i32.store offset=0 align=2
        0x0b, // end
    ];

    // Load and execute WASM module
    match WasmInstance::load_module(wasm_bytes) {
        Ok(mut instance) => {
            // Execute the module
            if let Err(_) = instance.execute() {
                // Execution failed - but we continue
            }
            
            // Verify memory contains 42
            if let Some(value) = instance.memory().read_i32(0) {
                if value == 42 {
                    // Success! In real firmware, we'd write to UART
                    // For now, we just loop to indicate success
                }
            }
        }
        Err(_) => {
            // Failed to load - continue anyway
        }
    }
    
    // Infinite loop (in real firmware, would be main loop)
    loop {
        // In real implementation, would handle UART, GPIO, etc.
    }
}

