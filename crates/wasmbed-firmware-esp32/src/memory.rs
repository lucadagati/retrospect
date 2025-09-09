// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

// Memory layout symbols for RISC-V HiFive1

// Stack symbols
#[unsafe(no_mangle)]
pub static _stack_start: u32 = 0x80004000; // End of 16KB RAM

// Data section symbols
#[unsafe(no_mangle)]
pub static _sdata: u32 = 0x80000000; // Start of RAM

#[unsafe(no_mangle)]
pub static _edata: u32 = 0x80000000; // Will be set by linker

#[unsafe(no_mangle)]
pub static _sidata: u32 = 0x20000000; // Start of flash

// BSS section symbols
#[unsafe(no_mangle)]
pub static _sbss: u32 = 0x80000000; // Will be set by linker

#[unsafe(no_mangle)]
pub static _ebss: u32 = 0x80000000; // Will be set by linker
