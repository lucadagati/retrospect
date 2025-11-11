// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

//! Global allocator for no_std builds
//! This provides heap allocation required by wasmparser with alloc feature

use linked_list_allocator::Heap;
use spin::Mutex;
use core::alloc::{GlobalAlloc, Layout};

// Heap size: 32KB for WASM runtime
// This is sufficient for parsing WASM modules and executing simple programs
const HEAP_SIZE: usize = 32 * 1024;

struct LockedHeap(Mutex<Heap>);

unsafe impl GlobalAlloc for LockedHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.0.lock().allocate_first_fit(layout).ok().map_or(core::ptr::null_mut(), |allocation| {
            allocation.as_ptr()
        })
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.0.lock().deallocate(core::ptr::NonNull::new_unchecked(ptr), layout);
    }
}

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap(Mutex::new(Heap::empty()));

static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

/// Initialize the global allocator
/// Must be called before any heap allocation
pub unsafe fn init() {
    let heap_start = HEAP.as_mut_ptr();
    let heap_size = HEAP_SIZE;
    ALLOCATOR.0.lock().init(heap_start, heap_size);
}

#[panic_handler]
fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    // In a real embedded system, this would log to UART or LED
    // For now, just loop forever
    loop {}
}

