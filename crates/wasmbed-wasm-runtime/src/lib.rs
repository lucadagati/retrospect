// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

//! # Wasmbed WASM Runtime
//! 
//! A WebAssembly runtime specifically designed for edge devices with device-specific
//! optimizations and host functions for PX4, microROS, and sensor integration.
//! 
//! ## Features
//! 
//! - **Device-specific configurations**: Optimized for MPU, MCU, and RISC-V architectures
//! - **PX4 integration**: Host functions for MAVLink communication
//! - **microROS integration**: DDS communication host functions
//! - **Sensor access**: Host functions for sensor data reading
//! - **Secure communication**: Encrypted communication host functions
//! - **Memory management**: Efficient memory allocation and limits
//! - **CPU time limits**: Execution time constraints for real-time systems

#![cfg_attr(feature = "no-std", no_std)]

pub mod config;
pub mod context;
pub mod device;
pub mod error;
pub mod host_functions;
pub mod runtime;
pub mod security;
pub mod validation;

// Re-exports for convenience
pub use config::*;
pub use context::*;
pub use device::*;
pub use error::*;
pub use runtime::*;

#[cfg(test)]
mod tests;
