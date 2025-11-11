// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

//! Device Runtime no_std implementation for Wasmbed platform
//!
//! This crate provides a no_std compatible runtime for executing
//! WebAssembly applications on embedded devices.

#![cfg_attr(not(feature = "std"), no_std)]

pub mod wasm_interpreter;
pub mod wasm_runtime;
pub mod tls_client;
pub mod tls_io;

// Re-export main types for easier usage
pub use wasm_interpreter::{WasmInstance, WasmValue, WasmMemory};
pub use wasm_runtime::WasmRuntime;
pub use tls_client::{TlsClient, Keypair};

