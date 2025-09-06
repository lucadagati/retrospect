// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

extern crate alloc;

use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;

/// Simple global allocator for no_std environment
pub struct SimpleAllocator;

unsafe impl GlobalAlloc for SimpleAllocator {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        // Simple implementation - just return null for now
        // In a real implementation, this would manage a memory pool
        null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Simple implementation - do nothing for now
    }
}

#[global_allocator]
static ALLOCATOR: SimpleAllocator = SimpleAllocator;
