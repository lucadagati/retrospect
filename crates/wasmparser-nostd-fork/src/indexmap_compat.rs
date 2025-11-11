// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright © 2025 Wasmbed contributors

//! Compatibility layer for indexmap-nostd to match indexmap API

pub use indexmap_nostd::{IndexMap, IndexSet};

// Equivalent type for compatibility (indexmap-nostd may not have this)
// Using a simple type alias - may need adjustment based on actual indexmap-nostd API
pub type Equivalent<K> = K;

// Re-export map module for Entry
pub mod map {
    pub use indexmap_nostd::map::Entry;
}

