// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

//! Minimal startup code for ARM Cortex-M
//! Provides reset handler and interrupt vector table

#[cfg(not(feature = "std"))]
extern "C" {
    static _stack_start: u32;
}

#[cfg(not(feature = "std"))]
#[link_section = ".vectors"]
#[no_mangle]
pub static VECTORS: [extern "C" fn(); 2] = [
    reset_handler,  // Reset handler (will be converted to address by linker)
    default_handler, // NMI handler
];

#[cfg(not(feature = "std"))]
#[no_mangle]
pub extern "C" fn reset_handler() {
    // Initialize .data section (copy from flash to RAM)
    extern "C" {
        static _sdata: u32;
        static _edata: u32;
        static _sidata: u32;
    }
    
    unsafe {
        let mut src = &_sidata as *const u32 as *const u8;
        let mut dst = &_sdata as *const u32 as *mut u8;
        let end = &_edata as *const u32 as *mut u8;
        
        while (dst as *const u8) < (end as *const u8) {
            *dst = *src;
            dst = dst.add(1);
            src = src.add(1);
        }
    }
    
    // Initialize .bss section (zero out)
    extern "C" {
        static _sbss: u32;
        static _ebss: u32;
    }
    
    unsafe {
        let mut dst = &_sbss as *const u32 as *mut u8;
        let end = &_ebss as *const u32 as *mut u8;
        
        while (dst as *const u8) < (end as *const u8) {
            *dst = 0;
            dst = dst.add(1);
        }
    }
    
    // Stack pointer is already set by Renode from vector table
    // Now call main
    extern "C" {
        fn main() -> i32;
    }
    
    unsafe {
        main();
    }
    
    // If main returns, loop forever
    loop {}
}

#[cfg(not(feature = "std"))]
#[no_mangle]
pub extern "C" fn default_handler() {
    loop {}
}

