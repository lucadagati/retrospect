// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

//! Custom getrandom implementation for no_std embedded targets
//! This provides a simple deterministic RNG for UUID generation

use getrandom::{Error, register_custom_getrandom};

fn custom_getrandom(buf: &mut [u8]) -> Result<(), Error> {
    // Simple deterministic RNG for embedded systems
    // In production, this should use hardware RNG if available
    static mut COUNTER: u64 = 0;
    
    unsafe {
        for byte in buf.iter_mut() {
            COUNTER = COUNTER.wrapping_mul(1103515245).wrapping_add(12345);
            *byte = (COUNTER >> 24) as u8;
        }
    }
    
    Ok(())
}

register_custom_getrandom!(custom_getrandom);

