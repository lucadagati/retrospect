// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

//! Example: Test WASM execution with real bytecode
//! 
//! This example demonstrates how to:
//! - Load a WASM module
//! - Execute it with the interpreter
//! - Test host functions
//!
//! Run with: cargo run --example test_wasm_execution --features std

use wasmbed_device_runtime::wasm_interpreter::WasmInstance;

fn main() {
    println!("🧪 Testing WASM Interpreter");
    println!("============================\n");

    // Test 1: Simple i32 addition
    println!("Test 1: Simple i32 operations");
    let wasm_bytes = vec![
        0x00, 0x61, 0x73, 0x6d, // WASM magic
        0x01, 0x00, 0x00, 0x00, // Version 1
        // Type section: func (i32, i32) -> i32
        0x01, 0x07, 0x01, 0x60, 0x02, 0x7f, 0x7f, 0x01, 0x7f,
        // Function section: 1 function
        0x03, 0x02, 0x01, 0x00,
        // Code section
        0x0a, 0x09, 0x01, 0x07, 0x00,
        0x20, 0x00, // local.get 0
        0x20, 0x01, // local.get 1
        0x6a, // i32.add
        0x0b, // end
    ];

    match WasmInstance::load_module(&wasm_bytes) {
        Ok(mut instance) => {
            println!("  ✓ Module loaded successfully");
            match instance.execute() {
                Ok(_) => println!("  ✓ Execution completed successfully"),
                Err(e) => println!("  ✗ Execution failed: {}", e),
            }
        }
        Err(e) => println!("  ✗ Failed to load module: {}", e),
    }

    println!("\nTest 2: Memory operations");
    let wasm_bytes = vec![
        0x00, 0x61, 0x73, 0x6d, // WASM magic
        0x01, 0x00, 0x00, 0x00, // Version 1
        // Memory section: 1 page
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

    match WasmInstance::load_module(&wasm_bytes) {
        Ok(mut instance) => {
            println!("  ✓ Module loaded successfully");
            match instance.execute() {
                Ok(_) => {
                    println!("  ✓ Execution completed");
                    let value = instance.memory().read_i32(0);
                    match value {
                        Some(42) => println!("  ✓ Memory contains correct value: 42"),
                        Some(v) => println!("  ✗ Memory contains wrong value: {}", v),
                        None => println!("  ✗ Failed to read from memory"),
                    }
                }
                Err(e) => println!("  ✗ Execution failed: {}", e),
            }
        }
        Err(e) => println!("  ✗ Failed to load module: {}", e),
    }

    println!("\nTest 3: Host function imports");
    let wasm_bytes = vec![
        0x00, 0x61, 0x73, 0x6d, // WASM magic
        0x01, 0x00, 0x00, 0x00, // Version 1
        // Type section: func (i32, i32) -> void
        0x01, 0x05, 0x01, 0x60, 0x02, 0x7f, 0x7f, 0x00,
        // Import section: import env.print
        0x02, 0x0f, 0x01,
        0x03, 0x65, 0x6e, 0x76, // "env"
        0x05, 0x70, 0x72, 0x69, 0x6e, 0x74, // "print"
        0x00, 0x00, // func, type 0
        // Function section: 0 functions
        0x03, 0x01, 0x00,
        // Code section: 0 functions
        0x0a, 0x02, 0x00,
    ];

    match WasmInstance::load_module(&wasm_bytes) {
        Ok(instance) => {
            println!("  ✓ Module loaded successfully");
            println!("  ✓ Imported functions: {}", instance.imported_functions());
        }
        Err(e) => println!("  ✗ Failed to load module: {}", e),
    }

    println!("\n✅ All tests completed!");
}

