// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

//! Build script to ensure linker script is available

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Only run for ARM Cortex-M target
    let target = env::var("TARGET").unwrap();
    if target.starts_with("thumbv") {
        // Select linker script based on target or environment variable
        let linker_script_name = if target.contains("thumbv7em") {
            // Default to STM32F4 for thumbv7em-none-eabihf
            // Can be overridden with LINKER_SCRIPT env var
            env::var("LINKER_SCRIPT")
                .unwrap_or_else(|_| "memory_stm32f4.x".to_string())
        } else {
            // For other ARM targets, try to detect or use default
            env::var("LINKER_SCRIPT")
                .unwrap_or_else(|_| "memory_stm32f4.x".to_string())
        };
        
        // Tell Cargo to rerun this build script if linker script changes
        println!("cargo:rerun-if-changed={}", linker_script_name);
        println!("cargo:rerun-if-changed=memory_stm32f4.x");
        println!("cargo:rerun-if-changed=memory_nrf52840.x");
        
        // Copy linker script to OUT_DIR so it can be found by the linker
        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        let linker_script = out_dir.join("memory.x");
        
        fs::copy(&linker_script_name, &linker_script)
            .expect(&format!("Failed to copy linker script: {}", linker_script_name));
        
        // Tell Cargo where to find the linker script
        println!("cargo:rustc-link-arg=-T{}", linker_script.display());
    }
}

