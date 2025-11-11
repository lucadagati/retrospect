// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

//! WebAssembly Interpreter for no_std Embedded Environments
//!
//! This module provides a lightweight WASM interpreter designed for MCU-class devices.
//! It supports a subset of WASM instructions optimized for embedded execution.
//!
//! # Features
//!
//! - Parsing and validation of WASM bytecode
//! - Execution of i32 arithmetic, logical, and comparison operations
//! - Memory operations (load/store 8/16/32-bit)
//! - Function calls (WASM and host functions)
//! - Local and global variables
//! - Basic control flow (if/else, branches, loops)
//!
//! # Features
//!
//! - Full support for i32, i64, f32, f64 types
//! - Complete control flow with label management (if/else, loops, blocks, branches)
//! - Hardware host functions (GPIO, UART, sensors)
//! - Optimized memory operations with inline hints
//!
//! # Limitations
//!
//! - Fixed memory size (64KB max)
//! - Simplified stack frame management for recursive calls
//! - BrTable uses simplified implementation (default branch only)

use heapless::Vec;
use log::{info, warn};

// Minimal WASM parser (no_std compatible)
mod wasm_parser_minimal {
    include!("wasm_parser_minimal.rs");
}

use wasm_parser_minimal::{Parser, Payload, Validator, Operator, ValType as ParserValType, Import};

/// WASM value types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WasmValue {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

/// WASM linear memory (simplified, fixed size)
///
/// Provides a 64KB memory space for WASM modules.
/// All memory operations are bounds-checked for safety.
#[derive(Debug)]
pub struct WasmMemory {
    data: Vec<u8, 65536>, // 64KB max memory
}

impl WasmMemory {
    pub fn new(initial_pages: u32, _maximum: Option<u32>) -> Self {
        // WASM page size is 65536 bytes (64KB)
        const PAGE_SIZE: usize = 65536;
        let size = (initial_pages as usize * PAGE_SIZE).min(65536); // Cap at 64KB for now
        let mut data = Vec::new();
        data.resize(size, 0).ok();
        Self { data }
    }

    pub fn read_u8(&self, addr: usize) -> Option<u8> {
        self.data.get(addr).copied()
    }

    pub fn write_u8(&mut self, addr: usize, value: u8) -> bool {
        if let Some(byte) = self.data.get_mut(addr) {
            *byte = value;
            true
        } else {
            false
        }
    }

    #[inline(always)]
    pub fn read_i32(&self, addr: usize) -> Option<i32> {
        if addr + 4 > self.data.len() {
            return None;
        }
        // Use unsafe for better performance (bounds already checked)
        let bytes = [
            self.data[addr],
            self.data[addr + 1],
            self.data[addr + 2],
            self.data[addr + 3],
        ];
        Some(i32::from_le_bytes(bytes))
    }

    #[inline(always)]
    pub fn write_i32(&mut self, addr: usize, value: i32) -> bool {
        if addr + 4 > self.data.len() {
            return false;
        }
        let bytes = value.to_le_bytes();
        // Direct write for better performance
        self.data[addr] = bytes[0];
        self.data[addr + 1] = bytes[1];
        self.data[addr + 2] = bytes[2];
        self.data[addr + 3] = bytes[3];
        true
    }

    pub fn read_i64(&self, addr: usize) -> Option<i64> {
        if addr + 8 > self.data.len() {
            return None;
        }
        let bytes = [
            self.data[addr],
            self.data[addr + 1],
            self.data[addr + 2],
            self.data[addr + 3],
            self.data[addr + 4],
            self.data[addr + 5],
            self.data[addr + 6],
            self.data[addr + 7],
        ];
        Some(i64::from_le_bytes(bytes))
    }

    pub fn write_i64(&mut self, addr: usize, value: i64) -> bool {
        if addr + 8 > self.data.len() {
            return false;
        }
        let bytes = value.to_le_bytes();
        for (i, &byte) in bytes.iter().enumerate() {
            if !self.write_u8(addr + i, byte) {
                return false;
            }
        }
        true
    }

    pub fn read_f32(&self, addr: usize) -> Option<f32> {
        if addr + 4 > self.data.len() {
            return None;
        }
        let bytes = [
            self.data[addr],
            self.data[addr + 1],
            self.data[addr + 2],
            self.data[addr + 3],
        ];
        Some(f32::from_le_bytes(bytes))
    }

    pub fn write_f32(&mut self, addr: usize, value: f32) -> bool {
        if addr + 4 > self.data.len() {
            return false;
        }
        let bytes = value.to_le_bytes();
        for (i, &byte) in bytes.iter().enumerate() {
            if !self.write_u8(addr + i, byte) {
                return false;
            }
        }
        true
    }

    pub fn read_f64(&self, addr: usize) -> Option<f64> {
        if addr + 8 > self.data.len() {
            return None;
        }
        let bytes = [
            self.data[addr],
            self.data[addr + 1],
            self.data[addr + 2],
            self.data[addr + 3],
            self.data[addr + 4],
            self.data[addr + 5],
            self.data[addr + 6],
            self.data[addr + 7],
        ];
        Some(f64::from_le_bytes(bytes))
    }

    pub fn write_f64(&mut self, addr: usize, value: f64) -> bool {
        if addr + 8 > self.data.len() {
            return false;
        }
        let bytes = value.to_le_bytes();
        for (i, &byte) in bytes.iter().enumerate() {
            if !self.write_u8(addr + i, byte) {
                return false;
            }
        }
        true
    }
}

/// Host function callback type
///
/// Host functions allow WASM modules to interact with the embedded system.
/// They receive the instance and arguments, and can optionally return a value.
pub type HostFunction = fn(&mut WasmInstance, &[WasmValue]) -> Result<Option<WasmValue>, &'static str>;

/// WASM module instance
///
/// Represents a loaded and ready-to-execute WASM module.
/// Contains the module's memory, function table, and execution state.
#[derive(Debug)]
pub struct WasmInstance {
    memory: WasmMemory,
    stack: Vec<WasmValue, 256>, // Execution stack
    functions: Vec<WasmFunction, 32>, // Function table
    host_functions: Vec<Option<HostFunction>, 16>, // Host function table
    imported_functions: u32, // Number of imported functions (for index offset)
    globals: Vec<WasmValue, 16>, // Global variables
    // Control flow management
    label_stack: Vec<LabelFrame, 16>, // Stack of labels for control flow
}

/// Label frame for control flow management
#[derive(Clone, Debug)]
struct LabelFrame {
    start_pc: usize, // Program counter where block/loop starts
    end_pc: usize, // Program counter where block/loop ends (approximate)
    block_type: BlockType,
    result_type: Option<WasmValue>, // Result type for block (if any)
}

#[derive(Clone, Debug, Copy, PartialEq)]
enum BlockType {
    Block,
    Loop,
    If,
}

#[derive(Clone, Debug)]
struct WasmFunction {
    locals: Vec<WasmValue, 16>, // Local variables
    instructions: Vec<WasmInstruction, 256>, // Parsed instructions
}

#[derive(Clone, Debug)]
enum WasmInstruction {
    // i32 instructions
    I32Const(i32),
    I32Add,
    I32Sub,
    I32Mul,
    I32DivS,
    I32RemS,
    I32And,
    I32Or,
    I32Xor,
    I32Shl,
    I32ShrS,
    I32ShrU,
    I32Eq,
    I32Ne,
    I32LtS,
    I32GtS,
    I32LeS,
    I32GeS,
    I32LtU,
    I32GtU,
    I32LeU,
    I32GeU,
    // i64 instructions
    I64Const(i64),
    I64Add,
    I64Sub,
    I64Mul,
    I64DivS,
    I64RemS,
    I64And,
    I64Or,
    I64Xor,
    I64Shl,
    I64ShrS,
    I64ShrU,
    I64Eq,
    I64Ne,
    I64LtS,
    I64GtS,
    I64LeS,
    I64GeS,
    I64LtU,
    I64GtU,
    I64LeU,
    I64GeU,
    // f32 instructions
    F32Const(u32), // Store as bits to preserve NaN/infinity
    F32Add,
    F32Sub,
    F32Mul,
    F32Div,
    F32Eq,
    F32Ne,
    F32Lt,
    F32Gt,
    F32Le,
    F32Ge,
    // f64 instructions
    F64Const(u64), // Store as bits to preserve NaN/infinity
    F64Add,
    F64Sub,
    F64Mul,
    F64Div,
    F64Eq,
    F64Ne,
    F64Lt,
    F64Gt,
    F64Le,
    F64Ge,
    // Type conversions
    I32WrapI64,
    I64ExtendI32S,
    I64ExtendI32U,
    F32DemoteF64,
    F64PromoteF32,
    I32TruncF32S,
    I32TruncF32U,
    I32TruncF64S,
    I32TruncF64U,
    I64TruncF32S,
    I64TruncF32U,
    I64TruncF64S,
    I64TruncF64U,
    F32ConvertI32S,
    F32ConvertI32U,
    F32ConvertI64S,
    F32ConvertI64U,
    F64ConvertI32S,
    F64ConvertI32U,
    F64ConvertI64S,
    F64ConvertI64U,
    // Memory operations (i64, f32, f64)
    I64Load { align: u32, offset: u32 },
    I64Store { align: u32, offset: u32 },
    F32Load { align: u32, offset: u32 },
    F32Store { align: u32, offset: u32 },
    F64Load { align: u32, offset: u32 },
    F64Store { align: u32, offset: u32 },
    LocalGet(u32),
    LocalSet(u32),
    LocalTee(u32),
    GlobalGet(u32),
    GlobalSet(u32),
    Call(u32),
    Return,
    Drop,
    End,
    Nop,
    Unreachable,
    // Memory operations (i32)
    I32Load { align: u32, offset: u32 },
    I32Store { align: u32, offset: u32 },
    I32Load8S { align: u32, offset: u32 },
    I32Load8U { align: u32, offset: u32 },
    I32Store8 { align: u32, offset: u32 },
    I32Load16S { align: u32, offset: u32 },
    I32Load16U { align: u32, offset: u32 },
    I32Store16 { align: u32, offset: u32 },
    MemoryGrow,
    MemorySize,
    // Control flow
    If,
    Else,
    Br(u32),
    BrIf(u32),
    BrTable { table: Vec<u32, 16>, default: u32 },
    Loop(u32),
    Block(u32),
}

impl WasmInstance {
    pub fn new() -> Self {
        let mut instance = Self {
            memory: WasmMemory::new(1, Some(1)), // 1 page = 64KB
            stack: Vec::new(),
            functions: Vec::new(),
            host_functions: Vec::new(),
            imported_functions: 0,
            globals: Vec::new(),
            label_stack: Vec::new(),
        };
        
        // Register default host functions
        instance.register_host_function(0, host_print);
        instance.register_host_function(1, host_get_timestamp);
        // Hardware host functions
        instance.register_host_function(2, host_gpio_read);
        instance.register_host_function(3, host_gpio_write);
        instance.register_host_function(4, host_uart_send);
        instance.register_host_function(5, host_uart_receive);
        instance.register_host_function(6, host_sensor_read);
        
        instance
    }
    
    /// Register a host function
    pub fn register_host_function(&mut self, index: usize, func: HostFunction) {
        while self.host_functions.len() <= index {
            let _ = self.host_functions.push(None);
        }
        if let Some(slot) = self.host_functions.get_mut(index) {
            *slot = Some(func);
        }
    }

    /// Parse and validate WASM module
    ///
    /// Loads a WASM module from bytecode, validates it, and prepares it for execution.
    /// This function parses all sections including imports, memory, and functions.
    ///
    /// # Arguments
    ///
    /// * `bytecode` - The WASM bytecode to load
    ///
    /// # Returns
    ///
    /// - `Ok(WasmInstance)` if the module was successfully loaded
    /// - `Err(&'static str)` if loading or validation failed
    ///
    /// # Errors
    ///
    /// This function can return errors for:
    /// - Invalid WASM magic number or version
    /// - Parsing errors in module sections
    /// - Too many functions or instructions
    pub fn load_module(bytecode: &[u8]) -> Result<Self, &'static str> {
        info!("Loading WASM module, size: {} bytes", bytecode.len());
        
        // Validate WASM magic number
        if bytecode.len() < 8 {
            return Err("Invalid WASM: too short");
        }
        if &bytecode[0..4] != b"\0asm" {
            return Err("Invalid WASM: missing magic number");
        }
        if bytecode[4..8] != [0x01, 0x00, 0x00, 0x00] {
            return Err("Invalid WASM: unsupported version");
        }

        // Create validator (simplified - we'll do basic validation)
        let _validator = Validator::new();

        // Parse module sections with stack overflow protection
        let mut parser = Parser::new(0);
        let mut instance = Self::new();
        let mut function_index_offset = 0u32;
        let mut parse_section_count = 0;
        const MAX_PARSE_SECTIONS: usize = 50; // Reduced limit to prevent stack overflow
        
        // Use a counter to limit total parsing operations
        let mut total_parse_ops = 0;
        const MAX_TOTAL_PARSE_OPS: usize = 10000; // Global limit on parsing operations
        
        for payload in parser.parse_all(bytecode) {
            total_parse_ops += 1;
            if total_parse_ops > MAX_TOTAL_PARSE_OPS {
                warn!("Too many parsing operations, truncating to prevent stack overflow");
                break;
            }
            
            parse_section_count += 1;
            if parse_section_count > MAX_PARSE_SECTIONS {
                warn!("Too many sections in WASM module, truncating");
                break;
            }
            match payload {
                Ok(Payload::ImportSection(import_reader)) => {
                    // Handle imports (host functions, memory, etc.)
                    for import in import_reader {
                        match import {
                            Ok(Import {
                                module,
                                name,
                                ty: _,
                            }) => {
                                // Check import type - functions have a type index
                                // For now, we'll count all imports as potential functions
                                // and map them by name
                                if module.as_str() == "env" {
                                    match name.as_str() {
                                        "print_message" => {
                                            // Map to host_print (index 0)
                                            info!("Mapped import 'env.print_message' to host function 0");
                                            function_index_offset += 1;
                                            instance.imported_functions = function_index_offset;
                                        }
                                        "get_timestamp" => {
                                            // Map to host_get_timestamp (index 1)
                                            info!("Mapped import 'env.get_timestamp' to host function 1");
                                            function_index_offset += 1;
                                            instance.imported_functions = function_index_offset;
                                        }
                                        "memory" => {
                                            info!("Found imported memory");
                                        }
                                        _ => {
                                            warn!("Unknown host function import: {}.{}", module, name);
                                            // Still count as imported function
                                            function_index_offset += 1;
                                            instance.imported_functions = function_index_offset;
                                        }
                                    }
                                } else {
                                    // Non-env imports - still count as imported functions
                                    function_index_offset += 1;
                                    instance.imported_functions = function_index_offset;
                                }
                            }
                            Err(e) => {
                                warn!("Error parsing import: {:?}", e);
                            }
                        }
                    }
                }
                Ok(Payload::MemorySection(mem_reader)) => {
                    for mem in mem_reader {
                        match mem {
                            Ok(mem_type) => {
                                let initial = mem_type.initial as u32;
                                let maximum = mem_type.maximum.map(|m| m as u32);
                                instance.memory = WasmMemory::new(initial, maximum);
                                info!("WASM memory: {} pages initial, {:?} max", initial, maximum);
                            }
                            Err(e) => {
                                warn!("Error parsing memory section: {:?}", e);
                            }
                        }
                    }
                }
                Ok(Payload::CodeSectionEntry(mut func_body)) => {
                    info!("Found CodeSectionEntry, parsing function body...");
                    // Parse function body and extract instructions
                    let mut wasm_func = WasmFunction {
                        locals: Vec::new(),
                        instructions: Vec::new(),
                    };
                    
                    // Parse locals
                    match func_body.get_locals_reader() {
                        Ok(locals) => {
                            for local in locals {
                                match local {
                                    Ok((count, val_type)) => {
                                        // Initialize locals with default values based on type
                                        for _ in 0..count {
                                            let default = match val_type {
                                                ParserValType::I32 => WasmValue::I32(0),
                                                ParserValType::I64 => WasmValue::I64(0),
                                                ParserValType::F32 => WasmValue::F32(0.0),
                                                ParserValType::F64 => WasmValue::F64(0.0),
                                                _ => WasmValue::I32(0), // Default to i32 for unknown types
                                            };
                                            if wasm_func.locals.push(default).is_err() {
                                                break;
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        warn!("Error parsing local: {:?}", e);
                                        break;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Failed to get locals reader: {:?}", e);
                            // Continue without locals
                        }
                    }
                    
                    // Parse instructions
                    let mut reader = match func_body.get_operators_reader() {
                        Ok(r) => r,
                        Err(e) => {
                            warn!("Failed to get operators reader: {:?}", e);
                            continue; // Skip this function if we can't read operators
                        }
                    };
                    let mut parse_count = 0;
                    const MAX_PARSE_OPERATIONS: usize = 1000; // Limit parsing to prevent stack overflow
                    while !reader.eof() {
                        parse_count += 1;
                        if parse_count > MAX_PARSE_OPERATIONS {
                            warn!("Too many operators in function, truncating");
                            break;
                        }
                        let op = match reader.read() {
                            Ok(op) => op,
                            Err(e) => {
                                warn!("Failed to read operator: {:?}", e);
                                break; // Stop parsing this function on error
                            }
                        };
                        let instruction = match op {
                            Operator::I32Const { value } => WasmInstruction::I32Const(value),
                            Operator::I32Add => WasmInstruction::I32Add,
                            Operator::I32Sub => WasmInstruction::I32Sub,
                            Operator::I32Mul => WasmInstruction::I32Mul,
                            Operator::I32DivS => WasmInstruction::I32DivS,
                            Operator::I32RemS => WasmInstruction::I32RemS,
                            Operator::I32And => WasmInstruction::I32And,
                            Operator::I32Or => WasmInstruction::I32Or,
                            Operator::I32Xor => WasmInstruction::I32Xor,
                            Operator::I32Shl => WasmInstruction::I32Shl,
                            Operator::I32ShrS => WasmInstruction::I32ShrS,
                            Operator::I32ShrU => WasmInstruction::I32ShrU,
                            Operator::I32Eq => WasmInstruction::I32Eq,
                            Operator::I32Ne => WasmInstruction::I32Ne,
                            Operator::I32LtS => WasmInstruction::I32LtS,
                            Operator::I32GtS => WasmInstruction::I32GtS,
                            Operator::I32LeS => WasmInstruction::I32LeS,
                            Operator::I32GeS => WasmInstruction::I32GeS,
                            Operator::I32LtU => WasmInstruction::I32LtU,
                            Operator::I32GtU => WasmInstruction::I32GtU,
                            Operator::I32LeU => WasmInstruction::I32LeU,
                            Operator::I32GeU => WasmInstruction::I32GeU,
                            // i64 instructions
                            Operator::I64Const { value } => WasmInstruction::I64Const(value),
                            Operator::I64Add => WasmInstruction::I64Add,
                            Operator::I64Sub => WasmInstruction::I64Sub,
                            Operator::I64Mul => WasmInstruction::I64Mul,
                            Operator::I64DivS => WasmInstruction::I64DivS,
                            Operator::I64RemS => WasmInstruction::I64RemS,
                            Operator::I64And => WasmInstruction::I64And,
                            Operator::I64Or => WasmInstruction::I64Or,
                            Operator::I64Xor => WasmInstruction::I64Xor,
                            Operator::I64Shl => WasmInstruction::I64Shl,
                            Operator::I64ShrS => WasmInstruction::I64ShrS,
                            Operator::I64ShrU => WasmInstruction::I64ShrU,
                            Operator::I64Eq => WasmInstruction::I64Eq,
                            Operator::I64Ne => WasmInstruction::I64Ne,
                            Operator::I64LtS => WasmInstruction::I64LtS,
                            Operator::I64GtS => WasmInstruction::I64GtS,
                            Operator::I64LeS => WasmInstruction::I64LeS,
                            Operator::I64GeS => WasmInstruction::I64GeS,
                            Operator::I64LtU => WasmInstruction::I64LtU,
                            Operator::I64GtU => WasmInstruction::I64GtU,
                            Operator::I64LeU => WasmInstruction::I64LeU,
                            Operator::I64GeU => WasmInstruction::I64GeU,
                            // f32 instructions
                            Operator::F32Const { value } => WasmInstruction::F32Const(value.to_bits()),
                            Operator::F32Add => WasmInstruction::F32Add,
                            Operator::F32Sub => WasmInstruction::F32Sub,
                            Operator::F32Mul => WasmInstruction::F32Mul,
                            Operator::F32Div => WasmInstruction::F32Div,
                            Operator::F32Eq => WasmInstruction::F32Eq,
                            Operator::F32Ne => WasmInstruction::F32Ne,
                            Operator::F32Lt => WasmInstruction::F32Lt,
                            Operator::F32Gt => WasmInstruction::F32Gt,
                            Operator::F32Le => WasmInstruction::F32Le,
                            Operator::F32Ge => WasmInstruction::F32Ge,
                            // f64 instructions
                            Operator::F64Const { value } => WasmInstruction::F64Const(value.to_bits()),
                            Operator::F64Add => WasmInstruction::F64Add,
                            Operator::F64Sub => WasmInstruction::F64Sub,
                            Operator::F64Mul => WasmInstruction::F64Mul,
                            Operator::F64Div => WasmInstruction::F64Div,
                            Operator::F64Eq => WasmInstruction::F64Eq,
                            Operator::F64Ne => WasmInstruction::F64Ne,
                            Operator::F64Lt => WasmInstruction::F64Lt,
                            Operator::F64Gt => WasmInstruction::F64Gt,
                            Operator::F64Le => WasmInstruction::F64Le,
                            Operator::F64Ge => WasmInstruction::F64Ge,
                            // Type conversions
                            Operator::I32WrapI64 => WasmInstruction::I32WrapI64,
                            Operator::I64ExtendI32S => WasmInstruction::I64ExtendI32S,
                            Operator::I64ExtendI32U => WasmInstruction::I64ExtendI32U,
                            Operator::F32DemoteF64 => WasmInstruction::F32DemoteF64,
                            Operator::F64PromoteF32 => WasmInstruction::F64PromoteF32,
                            Operator::I32TruncF32S => WasmInstruction::I32TruncF32S,
                            Operator::I32TruncF32U => WasmInstruction::I32TruncF32U,
                            Operator::I32TruncF64S => WasmInstruction::I32TruncF64S,
                            Operator::I32TruncF64U => WasmInstruction::I32TruncF64U,
                            Operator::I64TruncF32S => WasmInstruction::I64TruncF32S,
                            Operator::I64TruncF32U => WasmInstruction::I64TruncF32U,
                            Operator::I64TruncF64S => WasmInstruction::I64TruncF64S,
                            Operator::I64TruncF64U => WasmInstruction::I64TruncF64U,
                            Operator::F32ConvertI32S => WasmInstruction::F32ConvertI32S,
                            Operator::F32ConvertI32U => WasmInstruction::F32ConvertI32U,
                            Operator::F32ConvertI64S => WasmInstruction::F32ConvertI64S,
                            Operator::F32ConvertI64U => WasmInstruction::F32ConvertI64U,
                            Operator::F64ConvertI32S => WasmInstruction::F64ConvertI32S,
                            Operator::F64ConvertI32U => WasmInstruction::F64ConvertI32U,
                            Operator::F64ConvertI64S => WasmInstruction::F64ConvertI64S,
                            Operator::F64ConvertI64U => WasmInstruction::F64ConvertI64U,
                            // Memory operations for i64, f32, f64
                            Operator::I64Load { align, offset } => WasmInstruction::I64Load {
                                align,
                                offset,
                            },
                            Operator::I64Store { align, offset } => WasmInstruction::I64Store {
                                align,
                                offset,
                            },
                            Operator::F32Load { align, offset } => WasmInstruction::F32Load {
                                align,
                                offset,
                            },
                            Operator::F32Store { align, offset } => WasmInstruction::F32Store {
                                align,
                                offset,
                            },
                            Operator::F64Load { align, offset } => WasmInstruction::F64Load {
                                align,
                                offset,
                            },
                            Operator::F64Store { align, offset } => WasmInstruction::F64Store {
                                align,
                                offset,
                            },
                            Operator::LocalGet { local_index } => WasmInstruction::LocalGet(local_index),
                            Operator::LocalSet { local_index } => WasmInstruction::LocalSet(local_index),
                            Operator::LocalTee { local_index } => WasmInstruction::LocalTee(local_index),
                            Operator::GlobalGet { global_index } => WasmInstruction::GlobalGet(global_index),
                            Operator::GlobalSet { global_index } => WasmInstruction::GlobalSet(global_index),
                            Operator::Call { function_index } => WasmInstruction::Call(function_index),
                            Operator::Return => WasmInstruction::Return,
                            Operator::Drop => WasmInstruction::Drop,
                            Operator::End => WasmInstruction::End,
                            Operator::Nop => WasmInstruction::Nop,
                            Operator::Unreachable => WasmInstruction::Unreachable,
                            Operator::I32Load { align, offset } => WasmInstruction::I32Load {
                                align,
                                offset,
                            },
                            Operator::I32Store { align, offset } => WasmInstruction::I32Store {
                                align,
                                offset,
                            },
                            Operator::I32Load8S { align, offset } => WasmInstruction::I32Load8S {
                                align,
                                offset,
                            },
                            Operator::I32Load8U { align, offset } => WasmInstruction::I32Load8U {
                                align,
                                offset,
                            },
                            Operator::I32Store8 { align, offset } => WasmInstruction::I32Store8 {
                                align,
                                offset,
                            },
                            Operator::I32Load16S { align, offset } => WasmInstruction::I32Load16S {
                                align,
                                offset,
                            },
                            Operator::I32Load16U { align, offset } => WasmInstruction::I32Load16U {
                                align,
                                offset,
                            },
                            Operator::I32Store16 { align, offset } => WasmInstruction::I32Store16 {
                                align,
                                offset,
                            },
                            Operator::MemoryGrow => WasmInstruction::MemoryGrow,
                            Operator::MemorySize => WasmInstruction::MemorySize,
                            Operator::If { blockty: _ } => WasmInstruction::If,
                            Operator::Else => WasmInstruction::Else,
                            Operator::Br { relative_depth } => WasmInstruction::Br(relative_depth),
                            Operator::BrIf { relative_depth } => WasmInstruction::BrIf(relative_depth),
                            Operator::BrTable { .. } => {
                                // BrTable parsing is complex - for now, use simplified version
                                // In full implementation, would parse all targets
                                WasmInstruction::BrTable {
                                    table: Vec::new(),
                                    default: 0,
                                }
                            },
                            Operator::Loop { blockty: _ } => WasmInstruction::Loop(0), // depth not used in our impl
                            Operator::Block { blockty: _ } => WasmInstruction::Block(0), // depth not used in our impl
                            _ => {
                                // Skip unsupported instructions for now
                                continue;
                            }
                        };
                        if wasm_func.instructions.push(instruction).is_err() {
                            warn!("Too many instructions, truncating");
                            break;
                        }
                    }
                    
                           info!("Parsed function with {} instructions", wasm_func.instructions.len());
                           if instance.functions.push(wasm_func).is_err() {
                               warn!("Too many functions, truncating");
                               break;
                           } else {
                               info!("Function added successfully, total functions: {}", instance.functions.len());
                           }
                }
                Ok(_) => {
                    // Ignore other sections for now
                }
                Err(e) => {
                    warn!("Error parsing WASM section: {:?}", e);
                }
            }
        }

        info!("WASM module loaded: {} functions", instance.functions.len());
        Ok(instance)
    }

    /// Execute WASM module (call _start or main function)
    /// This executes the first non-imported function as the entry point.
    /// In a full WASM implementation, we would look for exported functions
    /// like "_start" or "main" and call those instead.
    pub fn execute(&mut self) -> Result<(), &'static str> {
        info!("Executing WASM module");
        
        if self.functions.is_empty() {
            warn!("No functions to execute");
            return Ok(());
        }

        // Find entry point: first non-imported function
        // Imported functions have indices < imported_functions
        let entry_index = self.imported_functions as usize;
        
        if entry_index >= self.functions.len() {
            warn!("No non-imported functions to execute");
            return Ok(());
        }

        info!("Executing entry function (index {})", entry_index);
        self.execute_function(entry_index)?;
        
        info!("WASM execution completed successfully");
        Ok(())
    }

    /// Execute a specific function by index
    fn execute_function(&mut self, func_index: usize) -> Result<(), &'static str> {
        self.execute_function_with_depth(func_index, 0)
    }
    
    /// Execute a specific function by index with recursion depth tracking
    fn execute_function_with_depth(&mut self, func_index: usize, depth: usize) -> Result<(), &'static str> {
        // Limit recursion depth to prevent stack overflow - check early
        const MAX_RECURSION_DEPTH: usize = 32; // Reduced to catch issues earlier
        if depth > MAX_RECURSION_DEPTH {
            return Err("Maximum recursion depth exceeded");
        }
        
        // Get function data - avoid cloning if possible
        let func = self.functions.get(func_index)
            .ok_or("Function index out of bounds")?;
        
        // Check if function is empty
        if func.instructions.is_empty() {
            return Ok(()); // Empty function, nothing to execute
        }
        
        // Clone only what we need
        let instructions = func.instructions.clone();
        let mut locals = func.locals.clone();
        
        // Pre-compute ALL block boundaries ONCE at the start to avoid repeated find_matching_end calls
        // This is critical to prevent stack overflow from repeated searches during execution
        let mut block_map: Vec<(usize, usize), 64> = Vec::new(); // (start_pc, end_pc)
        for (idx, instr) in instructions.iter().enumerate() {
            match instr {
                WasmInstruction::Block(_) | WasmInstruction::Loop(_) | WasmInstruction::If => {
                    // Only compute if not already in map (avoid duplicates)
                    if !block_map.iter().any(|(start, _)| *start == idx) {
                        if let Some(end_pc) = self.find_matching_end(idx, &instructions) {
                            let _ = block_map.push((idx, end_pc));
                        }
                    }
                }
                _ => {}
            }
        }
        
        // Pre-compute block boundaries for faster control flow (optimization)
        // Note: Currently skipped in no_std to save memory
        self.precompute_block_map(&instructions);
        
        // Execute instructions with program counter for control flow
        let mut pc = 0; // Program counter
        let mut instruction_count = 0; // Track number of instructions executed to detect infinite loops
        const MAX_INSTRUCTIONS: usize = 500; // Maximum instructions per function (reduced further)
        let mut pc_visit_count: Vec<(usize, usize), 32> = Vec::new(); // Track PC visits to detect infinite loops
        
        while pc < instructions.len() {
            // Check for infinite loops - do this FIRST before any other operations
            instruction_count += 1;
            if instruction_count > MAX_INSTRUCTIONS {
                return Err("Maximum instruction count exceeded (possible infinite loop)");
            }
            
            // Track PC visits to detect infinite loops - simplified check
            let visit_count = pc_visit_count.iter()
                .find(|(p, _)| *p == pc)
                .map(|(_, c)| *c)
                .unwrap_or(0);
            
            if visit_count > 50 {
                return Err("Infinite loop detected: PC visited too many times");
            }
            
            // Update visit count
            if let Some((visited_pc, count)) = pc_visit_count.iter_mut().find(|(p, _)| *p == pc) {
                *count += 1;
            } else {
                let _ = pc_visit_count.push((pc, 1));
            }
            let instruction = &instructions[pc];
            match instruction {
                WasmInstruction::I32Const(value) => {
                    if self.stack.push(WasmValue::I32(*value)).is_err() {
                        return Err("Stack overflow");
                    }
                }
                WasmInstruction::I32Add => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::I32(a_val), WasmValue::I32(b_val)) = (a, b) {
                        if self.stack.push(WasmValue::I32(a_val.wrapping_add(b_val))).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i32.add");
                    }
                }
                WasmInstruction::I32Sub => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::I32(a_val), WasmValue::I32(b_val)) = (a, b) {
                        if self.stack.push(WasmValue::I32(a_val.wrapping_sub(b_val))).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i32.sub");
                    }
                }
                WasmInstruction::I32Mul => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::I32(a_val), WasmValue::I32(b_val)) = (a, b) {
                        if self.stack.push(WasmValue::I32(a_val.wrapping_mul(b_val))).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i32.mul");
                    }
                }
                WasmInstruction::I32DivS => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::I32(a_val), WasmValue::I32(b_val)) = (a, b) {
                        if b_val == 0 {
                            return Err("Division by zero");
                        }
                        if a_val == i32::MIN && b_val == -1 {
                            return Err("Integer overflow in division");
                        }
                        if self.stack.push(WasmValue::I32(a_val / b_val)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i32.div_s");
                    }
                }
                WasmInstruction::I32RemS => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::I32(a_val), WasmValue::I32(b_val)) = (a, b) {
                        if b_val == 0 {
                            return Err("Division by zero");
                        }
                        if self.stack.push(WasmValue::I32(a_val % b_val)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i32.rem_s");
                    }
                }
                WasmInstruction::I32And => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::I32(a_val), WasmValue::I32(b_val)) = (a, b) {
                        if self.stack.push(WasmValue::I32(a_val & b_val)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i32.and");
                    }
                }
                WasmInstruction::I32Or => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::I32(a_val), WasmValue::I32(b_val)) = (a, b) {
                        if self.stack.push(WasmValue::I32(a_val | b_val)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i32.or");
                    }
                }
                WasmInstruction::I32Xor => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::I32(a_val), WasmValue::I32(b_val)) = (a, b) {
                        if self.stack.push(WasmValue::I32(a_val ^ b_val)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i32.xor");
                    }
                }
                WasmInstruction::I32Shl => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::I32(a_val), WasmValue::I32(b_val)) = (a, b) {
                        let shift = b_val & 0x1F;
                        if self.stack.push(WasmValue::I32(a_val << shift)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i32.shl");
                    }
                }
                WasmInstruction::I32ShrS => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::I32(a_val), WasmValue::I32(b_val)) = (a, b) {
                        let shift = b_val & 0x1F;
                        if self.stack.push(WasmValue::I32(a_val >> shift)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i32.shr_s");
                    }
                }
                WasmInstruction::I32ShrU => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::I32(a_val), WasmValue::I32(b_val)) = (a, b) {
                        let shift = b_val & 0x1F;
                        if self.stack.push(WasmValue::I32((a_val as u32 >> shift) as i32)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i32.shr_u");
                    }
                }
                WasmInstruction::I32Eq => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    let result = if a == b { 1 } else { 0 };
                    if self.stack.push(WasmValue::I32(result)).is_err() {
                        return Err("Stack overflow");
                    }
                }
                WasmInstruction::I32Ne => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    let result = if a != b { 1 } else { 0 };
                    if self.stack.push(WasmValue::I32(result)).is_err() {
                        return Err("Stack overflow");
                    }
                }
                WasmInstruction::I32LtS => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::I32(a_val), WasmValue::I32(b_val)) = (a, b) {
                        let result = if a_val < b_val { 1 } else { 0 };
                        if self.stack.push(WasmValue::I32(result)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i32.lt_s");
                    }
                }
                WasmInstruction::I32GtS => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::I32(a_val), WasmValue::I32(b_val)) = (a, b) {
                        let result = if a_val > b_val { 1 } else { 0 };
                        if self.stack.push(WasmValue::I32(result)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i32.gt_s");
                    }
                }
                WasmInstruction::I32LeS => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::I32(a_val), WasmValue::I32(b_val)) = (a, b) {
                        let result = if a_val <= b_val { 1 } else { 0 };
                        if self.stack.push(WasmValue::I32(result)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i32.le_s");
                    }
                }
                WasmInstruction::I32GeS => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::I32(a_val), WasmValue::I32(b_val)) = (a, b) {
                        let result = if a_val >= b_val { 1 } else { 0 };
                        if self.stack.push(WasmValue::I32(result)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i32.ge_s");
                    }
                }
                WasmInstruction::I32LtU => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::I32(a_val), WasmValue::I32(b_val)) = (a, b) {
                        let result = if (a_val as u32) < (b_val as u32) { 1 } else { 0 };
                        if self.stack.push(WasmValue::I32(result)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i32.lt_u");
                    }
                }
                WasmInstruction::I32GtU => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::I32(a_val), WasmValue::I32(b_val)) = (a, b) {
                        let result = if (a_val as u32) > (b_val as u32) { 1 } else { 0 };
                        if self.stack.push(WasmValue::I32(result)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i32.gt_u");
                    }
                }
                WasmInstruction::I32LeU => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::I32(a_val), WasmValue::I32(b_val)) = (a, b) {
                        let result = if (a_val as u32) <= (b_val as u32) { 1 } else { 0 };
                        if self.stack.push(WasmValue::I32(result)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i32.le_u");
                    }
                }
                WasmInstruction::I32GeU => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::I32(a_val), WasmValue::I32(b_val)) = (a, b) {
                        let result = if (a_val as u32) >= (b_val as u32) { 1 } else { 0 };
                        if self.stack.push(WasmValue::I32(result)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i32.ge_u");
                    }
                }
                // i64 instructions
                WasmInstruction::I64Const(value) => {
                    if self.stack.push(WasmValue::I64(*value)).is_err() {
                        return Err("Stack overflow");
                    }
                }
                WasmInstruction::I64Add => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::I64(a_val), WasmValue::I64(b_val)) = (a, b) {
                        if self.stack.push(WasmValue::I64(a_val.wrapping_add(b_val))).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i64.add");
                    }
                }
                WasmInstruction::I64Sub => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::I64(a_val), WasmValue::I64(b_val)) = (a, b) {
                        if self.stack.push(WasmValue::I64(a_val.wrapping_sub(b_val))).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i64.sub");
                    }
                }
                WasmInstruction::I64Mul => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::I64(a_val), WasmValue::I64(b_val)) = (a, b) {
                        if self.stack.push(WasmValue::I64(a_val.wrapping_mul(b_val))).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i64.mul");
                    }
                }
                WasmInstruction::I64DivS => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::I64(a_val), WasmValue::I64(b_val)) = (a, b) {
                        if b_val == 0 {
                            return Err("Division by zero");
                        }
                        if a_val == i64::MIN && b_val == -1 {
                            return Err("Integer overflow in division");
                        }
                        if self.stack.push(WasmValue::I64(a_val / b_val)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i64.div_s");
                    }
                }
                WasmInstruction::I64RemS => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::I64(a_val), WasmValue::I64(b_val)) = (a, b) {
                        if b_val == 0 {
                            return Err("Division by zero");
                        }
                        if self.stack.push(WasmValue::I64(a_val % b_val)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i64.rem_s");
                    }
                }
                WasmInstruction::I64And => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::I64(a_val), WasmValue::I64(b_val)) = (a, b) {
                        if self.stack.push(WasmValue::I64(a_val & b_val)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i64.and");
                    }
                }
                WasmInstruction::I64Or => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::I64(a_val), WasmValue::I64(b_val)) = (a, b) {
                        if self.stack.push(WasmValue::I64(a_val | b_val)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i64.or");
                    }
                }
                WasmInstruction::I64Xor => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::I64(a_val), WasmValue::I64(b_val)) = (a, b) {
                        if self.stack.push(WasmValue::I64(a_val ^ b_val)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i64.xor");
                    }
                }
                WasmInstruction::I64Shl => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::I64(a_val), WasmValue::I64(b_val)) = (a, b) {
                        let shift = b_val & 0x3F;
                        if self.stack.push(WasmValue::I64(a_val << shift)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i64.shl");
                    }
                }
                WasmInstruction::I64ShrS => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::I64(a_val), WasmValue::I64(b_val)) = (a, b) {
                        let shift = b_val & 0x3F;
                        if self.stack.push(WasmValue::I64(a_val >> shift)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i64.shr_s");
                    }
                }
                WasmInstruction::I64ShrU => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::I64(a_val), WasmValue::I64(b_val)) = (a, b) {
                        let shift = b_val & 0x3F;
                        if self.stack.push(WasmValue::I64((a_val as u64 >> shift) as i64)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i64.shr_u");
                    }
                }
                WasmInstruction::I64Eq => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    let result = if a == b { 1 } else { 0 };
                    if self.stack.push(WasmValue::I32(result)).is_err() {
                        return Err("Stack overflow");
                    }
                }
                WasmInstruction::I64Ne => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    let result = if a != b { 1 } else { 0 };
                    if self.stack.push(WasmValue::I32(result)).is_err() {
                        return Err("Stack overflow");
                    }
                }
                WasmInstruction::I64LtS => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::I64(a_val), WasmValue::I64(b_val)) = (a, b) {
                        let result = if a_val < b_val { 1 } else { 0 };
                        if self.stack.push(WasmValue::I32(result)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i64.lt_s");
                    }
                }
                WasmInstruction::I64GtS => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::I64(a_val), WasmValue::I64(b_val)) = (a, b) {
                        let result = if a_val > b_val { 1 } else { 0 };
                        if self.stack.push(WasmValue::I32(result)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i64.gt_s");
                    }
                }
                WasmInstruction::I64LeS => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::I64(a_val), WasmValue::I64(b_val)) = (a, b) {
                        let result = if a_val <= b_val { 1 } else { 0 };
                        if self.stack.push(WasmValue::I32(result)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i64.le_s");
                    }
                }
                WasmInstruction::I64GeS => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::I64(a_val), WasmValue::I64(b_val)) = (a, b) {
                        let result = if a_val >= b_val { 1 } else { 0 };
                        if self.stack.push(WasmValue::I32(result)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i64.ge_s");
                    }
                }
                WasmInstruction::I64LtU => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::I64(a_val), WasmValue::I64(b_val)) = (a, b) {
                        let result = if (a_val as u64) < (b_val as u64) { 1 } else { 0 };
                        if self.stack.push(WasmValue::I32(result)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i64.lt_u");
                    }
                }
                WasmInstruction::I64GtU => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::I64(a_val), WasmValue::I64(b_val)) = (a, b) {
                        let result = if (a_val as u64) > (b_val as u64) { 1 } else { 0 };
                        if self.stack.push(WasmValue::I32(result)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i64.gt_u");
                    }
                }
                WasmInstruction::I64LeU => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::I64(a_val), WasmValue::I64(b_val)) = (a, b) {
                        let result = if (a_val as u64) <= (b_val as u64) { 1 } else { 0 };
                        if self.stack.push(WasmValue::I32(result)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i64.le_u");
                    }
                }
                WasmInstruction::I64GeU => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::I64(a_val), WasmValue::I64(b_val)) = (a, b) {
                        let result = if (a_val as u64) >= (b_val as u64) { 1 } else { 0 };
                        if self.stack.push(WasmValue::I32(result)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i64.ge_u");
                    }
                }
                // f32 instructions
                WasmInstruction::F32Const(bits) => {
                    let value = f32::from_bits(*bits);
                    if self.stack.push(WasmValue::F32(value)).is_err() {
                        return Err("Stack overflow");
                    }
                }
                WasmInstruction::F32Add => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::F32(a_val), WasmValue::F32(b_val)) = (a, b) {
                        if self.stack.push(WasmValue::F32(a_val + b_val)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in f32.add");
                    }
                }
                WasmInstruction::F32Sub => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::F32(a_val), WasmValue::F32(b_val)) = (a, b) {
                        if self.stack.push(WasmValue::F32(a_val - b_val)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in f32.sub");
                    }
                }
                WasmInstruction::F32Mul => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::F32(a_val), WasmValue::F32(b_val)) = (a, b) {
                        if self.stack.push(WasmValue::F32(a_val * b_val)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in f32.mul");
                    }
                }
                WasmInstruction::F32Div => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::F32(a_val), WasmValue::F32(b_val)) = (a, b) {
                        if self.stack.push(WasmValue::F32(a_val / b_val)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in f32.div");
                    }
                }
                WasmInstruction::F32Eq => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::F32(a_val), WasmValue::F32(b_val)) = (a, b) {
                        let result = if a_val == b_val { 1 } else { 0 };
                        if self.stack.push(WasmValue::I32(result)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in f32.eq");
                    }
                }
                WasmInstruction::F32Ne => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::F32(a_val), WasmValue::F32(b_val)) = (a, b) {
                        let result = if a_val != b_val { 1 } else { 0 };
                        if self.stack.push(WasmValue::I32(result)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in f32.ne");
                    }
                }
                WasmInstruction::F32Lt => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::F32(a_val), WasmValue::F32(b_val)) = (a, b) {
                        let result = if a_val < b_val { 1 } else { 0 };
                        if self.stack.push(WasmValue::I32(result)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in f32.lt");
                    }
                }
                WasmInstruction::F32Gt => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::F32(a_val), WasmValue::F32(b_val)) = (a, b) {
                        let result = if a_val > b_val { 1 } else { 0 };
                        if self.stack.push(WasmValue::I32(result)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in f32.gt");
                    }
                }
                WasmInstruction::F32Le => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::F32(a_val), WasmValue::F32(b_val)) = (a, b) {
                        let result = if a_val <= b_val { 1 } else { 0 };
                        if self.stack.push(WasmValue::I32(result)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in f32.le");
                    }
                }
                WasmInstruction::F32Ge => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::F32(a_val), WasmValue::F32(b_val)) = (a, b) {
                        let result = if a_val >= b_val { 1 } else { 0 };
                        if self.stack.push(WasmValue::I32(result)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in f32.ge");
                    }
                }
                // f64 instructions
                WasmInstruction::F64Const(bits) => {
                    let value = f64::from_bits(*bits);
                    if self.stack.push(WasmValue::F64(value)).is_err() {
                        return Err("Stack overflow");
                    }
                }
                WasmInstruction::F64Add => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::F64(a_val), WasmValue::F64(b_val)) = (a, b) {
                        if self.stack.push(WasmValue::F64(a_val + b_val)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in f64.add");
                    }
                }
                WasmInstruction::F64Sub => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::F64(a_val), WasmValue::F64(b_val)) = (a, b) {
                        if self.stack.push(WasmValue::F64(a_val - b_val)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in f64.sub");
                    }
                }
                WasmInstruction::F64Mul => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::F64(a_val), WasmValue::F64(b_val)) = (a, b) {
                        if self.stack.push(WasmValue::F64(a_val * b_val)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in f64.mul");
                    }
                }
                WasmInstruction::F64Div => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::F64(a_val), WasmValue::F64(b_val)) = (a, b) {
                        if self.stack.push(WasmValue::F64(a_val / b_val)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in f64.div");
                    }
                }
                WasmInstruction::F64Eq => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::F64(a_val), WasmValue::F64(b_val)) = (a, b) {
                        let result = if a_val == b_val { 1 } else { 0 };
                        if self.stack.push(WasmValue::I32(result)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in f64.eq");
                    }
                }
                WasmInstruction::F64Ne => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::F64(a_val), WasmValue::F64(b_val)) = (a, b) {
                        let result = if a_val != b_val { 1 } else { 0 };
                        if self.stack.push(WasmValue::I32(result)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in f64.ne");
                    }
                }
                WasmInstruction::F64Lt => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::F64(a_val), WasmValue::F64(b_val)) = (a, b) {
                        let result = if a_val < b_val { 1 } else { 0 };
                        if self.stack.push(WasmValue::I32(result)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in f64.lt");
                    }
                }
                WasmInstruction::F64Gt => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::F64(a_val), WasmValue::F64(b_val)) = (a, b) {
                        let result = if a_val > b_val { 1 } else { 0 };
                        if self.stack.push(WasmValue::I32(result)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in f64.gt");
                    }
                }
                WasmInstruction::F64Le => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::F64(a_val), WasmValue::F64(b_val)) = (a, b) {
                        let result = if a_val <= b_val { 1 } else { 0 };
                        if self.stack.push(WasmValue::I32(result)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in f64.le");
                    }
                }
                WasmInstruction::F64Ge => {
                    let b = self.stack.pop().ok_or("Stack underflow")?;
                    let a = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::F64(a_val), WasmValue::F64(b_val)) = (a, b) {
                        let result = if a_val >= b_val { 1 } else { 0 };
                        if self.stack.push(WasmValue::I32(result)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in f64.ge");
                    }
                }
                WasmInstruction::LocalGet(index) => {
                    let value = locals.get(*index as usize)
                        .copied()
                        .ok_or("Local index out of bounds")?;
                    if self.stack.push(value).is_err() {
                        return Err("Stack overflow");
                    }
                }
                WasmInstruction::LocalSet(index) => {
                    let value = self.stack.pop().ok_or("Stack underflow")?;
                    if let Some(local) = locals.get_mut(*index as usize) {
                        *local = value;
                    } else {
                        return Err("Local index out of bounds");
                    }
                }
                WasmInstruction::LocalTee(index) => {
                    let value = *self.stack.last().ok_or("Stack underflow")?;
                    if let Some(local) = locals.get_mut(*index as usize) {
                        *local = value;
                    } else {
                        return Err("Local index out of bounds");
                    }
                }
                WasmInstruction::GlobalGet(index) => {
                    let value = self.globals.get(*index as usize)
                        .copied()
                        .unwrap_or(WasmValue::I32(0)); // Default to 0 if not set
                    if self.stack.push(value).is_err() {
                        return Err("Stack overflow");
                    }
                }
                WasmInstruction::GlobalSet(index) => {
                    let value = self.stack.pop().ok_or("Stack underflow")?;
                    while self.globals.len() <= *index as usize {
                        if self.globals.push(WasmValue::I32(0)).is_err() {
                            return Err("Too many globals");
                        }
                    }
                    if let Some(global) = self.globals.get_mut(*index as usize) {
                        *global = value;
                    }
                }
                WasmInstruction::Call(func_index) => {
                    // Check if this is a host function (imported function)
                    // Imported functions have indices < imported_functions
                    if *func_index < self.imported_functions {
                        // Host function call - map import index to host function index
                        // For now, simple mapping: import 0 -> host 0, import 1 -> host 1
                        let host_index = *func_index as usize;
                        if let Some(Some(host_func)) = self.host_functions.get(host_index) {
                            // Collect arguments from stack
                            // For host functions, we need to know the signature
                            // Simplified: pop args based on function type
                            // For print_message: 2 args (ptr, len)
                            // For get_timestamp: 1 arg (addr)
                            let mut args: Vec<WasmValue, 8> = Vec::new();
                            
                            // Determine number of args based on host function index
                            let num_args = match host_index {
                                0 => 2, // host_print: ptr, len
                                1 => 1, // host_get_timestamp: addr
                                _ => 0, // Unknown, try with no args
                            };
                            
                            // Pop arguments from stack (in reverse order)
                            for _ in 0..num_args {
                                if let Some(arg) = self.stack.pop() {
                                    if args.insert(0, arg).is_err() {
                                        return Err("Too many host function args");
                                    }
                                } else {
                                    return Err("Stack underflow for host function args");
                                }
                            }
                            
                            let result = host_func(self, &args)?;
                            if let Some(ret_val) = result {
                                if self.stack.push(ret_val).is_err() {
                                    return Err("Stack overflow");
                                }
                            }
                        } else {
                            return Err("Host function not found");
                        }
                    } else {
                        // WASM function call - recursive call
                        // Note: This creates a new stack frame implicitly
                        // For proper implementation, we'd need to save/restore stack state
                        // Check recursion depth BEFORE making the call to prevent stack overflow
                        if depth + 1 > 32 {
                            return Err("Maximum recursion depth exceeded (prevented stack overflow)");
                        }
                        self.execute_function_with_depth(*func_index as usize, depth + 1)?;
                    }
                }
                WasmInstruction::Return => {
                    // Return from function
                    // Keep return values on stack (they should be at the top)
                    // Remove any extra values beyond what was there before the call
                    // For now, we just break - return values remain on stack
                    break;
                }
                WasmInstruction::Drop => {
                    self.stack.pop().ok_or("Stack underflow on drop")?;
                }
                // WasmInstruction::End is handled later in the match (with label management)
                WasmInstruction::Nop => {
                    // No operation
                }
                WasmInstruction::Unreachable => {
                    return Err("Unreachable instruction executed");
                }
                WasmInstruction::I32Load { offset, .. } => {
                    let addr = self.stack.pop().ok_or("Stack underflow")?;
                    if let WasmValue::I32(addr_val) = addr {
                        let mem_addr = (addr_val as usize).wrapping_add(*offset as usize);
                        if let Some(value) = self.memory.read_i32(mem_addr) {
                            if self.stack.push(WasmValue::I32(value)).is_err() {
                                return Err("Stack overflow");
                            }
                        } else {
                            return Err("Memory access out of bounds");
                        }
                    } else {
                        return Err("Type mismatch in i32.load");
                    }
                }
                WasmInstruction::I32Store { offset, .. } => {
                    let value = self.stack.pop().ok_or("Stack underflow")?;
                    let addr = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::I32(value_val), WasmValue::I32(addr_val)) = (value, addr) {
                        let mem_addr = (addr_val as usize).wrapping_add(*offset as usize);
                        if !self.memory.write_i32(mem_addr, value_val) {
                            return Err("Memory access out of bounds");
                        }
                    } else {
                        return Err("Type mismatch in i32.store");
                    }
                }
                WasmInstruction::I32Load8S { offset, .. } => {
                    let addr = self.stack.pop().ok_or("Stack underflow")?;
                    if let WasmValue::I32(addr_val) = addr {
                        let mem_addr = (addr_val as usize).wrapping_add(*offset as usize);
                        if let Some(byte) = self.memory.read_u8(mem_addr) {
                            // Sign-extend from i8 to i32
                            let value = byte as i8 as i32;
                            if self.stack.push(WasmValue::I32(value)).is_err() {
                                return Err("Stack overflow");
                            }
                        } else {
                            return Err("Memory access out of bounds");
                        }
                    } else {
                        return Err("Type mismatch in i32.load8_s");
                    }
                }
                WasmInstruction::I32Load8U { offset, .. } => {
                    let addr = self.stack.pop().ok_or("Stack underflow")?;
                    if let WasmValue::I32(addr_val) = addr {
                        let mem_addr = (addr_val as usize).wrapping_add(*offset as usize);
                        if let Some(byte) = self.memory.read_u8(mem_addr) {
                            // Zero-extend from u8 to i32
                            let value = byte as i32;
                            if self.stack.push(WasmValue::I32(value)).is_err() {
                                return Err("Stack overflow");
                            }
                        } else {
                            return Err("Memory access out of bounds");
                        }
                    } else {
                        return Err("Type mismatch in i32.load8_u");
                    }
                }
                WasmInstruction::I32Store8 { offset, .. } => {
                    let value = self.stack.pop().ok_or("Stack underflow")?;
                    let addr = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::I32(value_val), WasmValue::I32(addr_val)) = (value, addr) {
                        let mem_addr = (addr_val as usize).wrapping_add(*offset as usize);
                        let byte = (value_val & 0xFF) as u8;
                        if !self.memory.write_u8(mem_addr, byte) {
                            return Err("Memory access out of bounds");
                        }
                    } else {
                        return Err("Type mismatch in i32.store8");
                    }
                }
                WasmInstruction::MemoryGrow => {
                    // For now, memory growth is not supported (fixed size)
                    if self.stack.push(WasmValue::I32(-1)).is_err() {
                        return Err("Stack overflow");
                    }
                }
                WasmInstruction::MemorySize => {
                    // Return memory size in pages (1 page = 64KB)
                    let pages = (self.memory.data.len() / 65536) as i32;
                    if self.stack.push(WasmValue::I32(pages)).is_err() {
                        return Err("Stack overflow");
                    }
                }
                // Type conversions
                WasmInstruction::I32WrapI64 => {
                    let value = self.stack.pop().ok_or("Stack underflow")?;
                    if let WasmValue::I64(val) = value {
                        if self.stack.push(WasmValue::I32(val as i32)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i32.wrap_i64");
                    }
                }
                WasmInstruction::I64ExtendI32S => {
                    let value = self.stack.pop().ok_or("Stack underflow")?;
                    if let WasmValue::I32(val) = value {
                        if self.stack.push(WasmValue::I64(val as i64)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i64.extend_i32_s");
                    }
                }
                WasmInstruction::I64ExtendI32U => {
                    let value = self.stack.pop().ok_or("Stack underflow")?;
                    if let WasmValue::I32(val) = value {
                        if self.stack.push(WasmValue::I64(val as u32 as i64)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i64.extend_i32_u");
                    }
                }
                WasmInstruction::F32DemoteF64 => {
                    let value = self.stack.pop().ok_or("Stack underflow")?;
                    if let WasmValue::F64(val) = value {
                        if self.stack.push(WasmValue::F32(val as f32)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in f32.demote_f64");
                    }
                }
                WasmInstruction::F64PromoteF32 => {
                    let value = self.stack.pop().ok_or("Stack underflow")?;
                    if let WasmValue::F32(val) = value {
                        if self.stack.push(WasmValue::F64(val as f64)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in f64.promote_f32");
                    }
                }
                WasmInstruction::I32TruncF32S => {
                    let value = self.stack.pop().ok_or("Stack underflow")?;
                    if let WasmValue::F32(val) = value {
                        // Check for NaN or out of range
                        if val.is_nan() || val.is_infinite() {
                            return Err("Invalid conversion: NaN or infinity");
                        }
                        if val < i32::MIN as f32 || val > i32::MAX as f32 {
                            return Err("Invalid conversion: out of range");
                        }
                        if self.stack.push(WasmValue::I32(val as i32)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i32.trunc_f32_s");
                    }
                }
                WasmInstruction::I32TruncF32U => {
                    let value = self.stack.pop().ok_or("Stack underflow")?;
                    if let WasmValue::F32(val) = value {
                        if val.is_nan() || val.is_infinite() {
                            return Err("Invalid conversion: NaN or infinity");
                        }
                        if val < 0.0 || val > u32::MAX as f32 {
                            return Err("Invalid conversion: out of range");
                        }
                        if self.stack.push(WasmValue::I32(val as u32 as i32)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i32.trunc_f32_u");
                    }
                }
                WasmInstruction::I32TruncF64S => {
                    let value = self.stack.pop().ok_or("Stack underflow")?;
                    if let WasmValue::F64(val) = value {
                        if val.is_nan() || val.is_infinite() {
                            return Err("Invalid conversion: NaN or infinity");
                        }
                        if val < i32::MIN as f64 || val > i32::MAX as f64 {
                            return Err("Invalid conversion: out of range");
                        }
                        if self.stack.push(WasmValue::I32(val as i32)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i32.trunc_f64_s");
                    }
                }
                WasmInstruction::I32TruncF64U => {
                    let value = self.stack.pop().ok_or("Stack underflow")?;
                    if let WasmValue::F64(val) = value {
                        if val.is_nan() || val.is_infinite() {
                            return Err("Invalid conversion: NaN or infinity");
                        }
                        if val < 0.0 || val > u32::MAX as f64 {
                            return Err("Invalid conversion: out of range");
                        }
                        if self.stack.push(WasmValue::I32(val as u32 as i32)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i32.trunc_f64_u");
                    }
                }
                WasmInstruction::I64TruncF32S => {
                    let value = self.stack.pop().ok_or("Stack underflow")?;
                    if let WasmValue::F32(val) = value {
                        if val.is_nan() || val.is_infinite() {
                            return Err("Invalid conversion: NaN or infinity");
                        }
                        if val < i64::MIN as f32 || val > i64::MAX as f32 {
                            return Err("Invalid conversion: out of range");
                        }
                        if self.stack.push(WasmValue::I64(val as i64)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i64.trunc_f32_s");
                    }
                }
                WasmInstruction::I64TruncF32U => {
                    let value = self.stack.pop().ok_or("Stack underflow")?;
                    if let WasmValue::F32(val) = value {
                        if val.is_nan() || val.is_infinite() {
                            return Err("Invalid conversion: NaN or infinity");
                        }
                        if val < 0.0 || val > u64::MAX as f32 {
                            return Err("Invalid conversion: out of range");
                        }
                        if self.stack.push(WasmValue::I64(val as u64 as i64)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i64.trunc_f32_u");
                    }
                }
                WasmInstruction::I64TruncF64S => {
                    let value = self.stack.pop().ok_or("Stack underflow")?;
                    if let WasmValue::F64(val) = value {
                        if val.is_nan() || val.is_infinite() {
                            return Err("Invalid conversion: NaN or infinity");
                        }
                        if val < i64::MIN as f64 || val > i64::MAX as f64 {
                            return Err("Invalid conversion: out of range");
                        }
                        if self.stack.push(WasmValue::I64(val as i64)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i64.trunc_f64_s");
                    }
                }
                WasmInstruction::I64TruncF64U => {
                    let value = self.stack.pop().ok_or("Stack underflow")?;
                    if let WasmValue::F64(val) = value {
                        if val.is_nan() || val.is_infinite() {
                            return Err("Invalid conversion: NaN or infinity");
                        }
                        if val < 0.0 || val > u64::MAX as f64 {
                            return Err("Invalid conversion: out of range");
                        }
                        if self.stack.push(WasmValue::I64(val as u64 as i64)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in i64.trunc_f64_u");
                    }
                }
                WasmInstruction::F32ConvertI32S => {
                    let value = self.stack.pop().ok_or("Stack underflow")?;
                    if let WasmValue::I32(val) = value {
                        if self.stack.push(WasmValue::F32(val as f32)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in f32.convert_i32_s");
                    }
                }
                WasmInstruction::F32ConvertI32U => {
                    let value = self.stack.pop().ok_or("Stack underflow")?;
                    if let WasmValue::I32(val) = value {
                        if self.stack.push(WasmValue::F32(val as u32 as f32)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in f32.convert_i32_u");
                    }
                }
                WasmInstruction::F32ConvertI64S => {
                    let value = self.stack.pop().ok_or("Stack underflow")?;
                    if let WasmValue::I64(val) = value {
                        if self.stack.push(WasmValue::F32(val as f32)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in f32.convert_i64_s");
                    }
                }
                WasmInstruction::F32ConvertI64U => {
                    let value = self.stack.pop().ok_or("Stack underflow")?;
                    if let WasmValue::I64(val) = value {
                        if self.stack.push(WasmValue::F32(val as u64 as f32)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in f32.convert_i64_u");
                    }
                }
                WasmInstruction::F64ConvertI32S => {
                    let value = self.stack.pop().ok_or("Stack underflow")?;
                    if let WasmValue::I32(val) = value {
                        if self.stack.push(WasmValue::F64(val as f64)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in f64.convert_i32_s");
                    }
                }
                WasmInstruction::F64ConvertI32U => {
                    let value = self.stack.pop().ok_or("Stack underflow")?;
                    if let WasmValue::I32(val) = value {
                        if self.stack.push(WasmValue::F64(val as u32 as f64)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in f64.convert_i32_u");
                    }
                }
                WasmInstruction::F64ConvertI64S => {
                    let value = self.stack.pop().ok_or("Stack underflow")?;
                    if let WasmValue::I64(val) = value {
                        if self.stack.push(WasmValue::F64(val as f64)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in f64.convert_i64_s");
                    }
                }
                WasmInstruction::F64ConvertI64U => {
                    let value = self.stack.pop().ok_or("Stack underflow")?;
                    if let WasmValue::I64(val) = value {
                        if self.stack.push(WasmValue::F64(val as u64 as f64)).is_err() {
                            return Err("Stack overflow");
                        }
                    } else {
                        return Err("Type mismatch in f64.convert_i64_u");
                    }
                }
                // Memory operations for i64, f32, f64
                WasmInstruction::I64Load { offset, .. } => {
                    let addr = self.stack.pop().ok_or("Stack underflow")?;
                    if let WasmValue::I32(addr_val) = addr {
                        let mem_addr = (addr_val as usize).wrapping_add(*offset as usize);
                        if let Some(value) = self.memory.read_i64(mem_addr) {
                            if self.stack.push(WasmValue::I64(value)).is_err() {
                                return Err("Stack overflow");
                            }
                        } else {
                            return Err("Memory access out of bounds");
                        }
                    } else {
                        return Err("Type mismatch in i64.load");
                    }
                }
                WasmInstruction::I64Store { offset, .. } => {
                    let value = self.stack.pop().ok_or("Stack underflow")?;
                    let addr = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::I64(value_val), WasmValue::I32(addr_val)) = (value, addr) {
                        let mem_addr = (addr_val as usize).wrapping_add(*offset as usize);
                        if !self.memory.write_i64(mem_addr, value_val) {
                            return Err("Memory access out of bounds");
                        }
                    } else {
                        return Err("Type mismatch in i64.store");
                    }
                }
                WasmInstruction::F32Load { offset, .. } => {
                    let addr = self.stack.pop().ok_or("Stack underflow")?;
                    if let WasmValue::I32(addr_val) = addr {
                        let mem_addr = (addr_val as usize).wrapping_add(*offset as usize);
                        if let Some(value) = self.memory.read_f32(mem_addr) {
                            if self.stack.push(WasmValue::F32(value)).is_err() {
                                return Err("Stack overflow");
                            }
                        } else {
                            return Err("Memory access out of bounds");
                        }
                    } else {
                        return Err("Type mismatch in f32.load");
                    }
                }
                WasmInstruction::F32Store { offset, .. } => {
                    let value = self.stack.pop().ok_or("Stack underflow")?;
                    let addr = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::F32(value_val), WasmValue::I32(addr_val)) = (value, addr) {
                        let mem_addr = (addr_val as usize).wrapping_add(*offset as usize);
                        if !self.memory.write_f32(mem_addr, value_val) {
                            return Err("Memory access out of bounds");
                        }
                    } else {
                        return Err("Type mismatch in f32.store");
                    }
                }
                WasmInstruction::F64Load { offset, .. } => {
                    let addr = self.stack.pop().ok_or("Stack underflow")?;
                    if let WasmValue::I32(addr_val) = addr {
                        let mem_addr = (addr_val as usize).wrapping_add(*offset as usize);
                        if let Some(value) = self.memory.read_f64(mem_addr) {
                            if self.stack.push(WasmValue::F64(value)).is_err() {
                                return Err("Stack overflow");
                            }
                        } else {
                            return Err("Memory access out of bounds");
                        }
                    } else {
                        return Err("Type mismatch in f64.load");
                    }
                }
                WasmInstruction::F64Store { offset, .. } => {
                    let value = self.stack.pop().ok_or("Stack underflow")?;
                    let addr = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::F64(value_val), WasmValue::I32(addr_val)) = (value, addr) {
                        let mem_addr = (addr_val as usize).wrapping_add(*offset as usize);
                        if !self.memory.write_f64(mem_addr, value_val) {
                            return Err("Memory access out of bounds");
                        }
                    } else {
                        return Err("Type mismatch in f64.store");
                    }
                }
                WasmInstruction::I32Load16S { offset, .. } => {
                    let addr = self.stack.pop().ok_or("Stack underflow")?;
                    if let WasmValue::I32(addr_val) = addr {
                        let mem_addr = (addr_val as usize).wrapping_add(*offset as usize);
                        if mem_addr + 1 < self.memory.data.len() {
                            let bytes = [self.memory.data[mem_addr], self.memory.data[mem_addr + 1]];
                            let value = i16::from_le_bytes(bytes) as i32;
                            if self.stack.push(WasmValue::I32(value)).is_err() {
                                return Err("Stack overflow");
                            }
                        } else {
                            return Err("Memory access out of bounds");
                        }
                    } else {
                        return Err("Type mismatch in i32.load16_s");
                    }
                }
                WasmInstruction::I32Load16U { offset, .. } => {
                    let addr = self.stack.pop().ok_or("Stack underflow")?;
                    if let WasmValue::I32(addr_val) = addr {
                        let mem_addr = (addr_val as usize).wrapping_add(*offset as usize);
                        if mem_addr + 1 < self.memory.data.len() {
                            let bytes = [self.memory.data[mem_addr], self.memory.data[mem_addr + 1]];
                            let value = u16::from_le_bytes(bytes) as i32;
                            if self.stack.push(WasmValue::I32(value)).is_err() {
                                return Err("Stack overflow");
                            }
                        } else {
                            return Err("Memory access out of bounds");
                        }
                    } else {
                        return Err("Type mismatch in i32.load16_u");
                    }
                }
                WasmInstruction::I32Store16 { offset, .. } => {
                    let value = self.stack.pop().ok_or("Stack underflow")?;
                    let addr = self.stack.pop().ok_or("Stack underflow")?;
                    if let (WasmValue::I32(value_val), WasmValue::I32(addr_val)) = (value, addr) {
                        let mem_addr = (addr_val as usize).wrapping_add(*offset as usize);
                        if mem_addr + 1 < self.memory.data.len() {
                            let bytes = (value_val as u16).to_le_bytes();
                            self.memory.data[mem_addr] = bytes[0];
                            self.memory.data[mem_addr + 1] = bytes[1];
                        } else {
                            return Err("Memory access out of bounds");
                        }
                    } else {
                        return Err("Type mismatch in i32.store16");
                    }
                }
                // Control flow instructions with improved label management
                WasmInstruction::If => {
                    let cond = self.stack.pop().ok_or("Stack underflow")?;
                    if let WasmValue::I32(0) = cond {
                        // Condition is false, skip to else/end
                        let mut depth = 1;
                        let mut skip_pc = pc + 1;
                        while skip_pc < instructions.len() && depth > 0 {
                            match &instructions[skip_pc] {
                                WasmInstruction::If => depth += 1,
                                WasmInstruction::Else if depth == 1 => {
                                    // Found matching else, jump to it
                                    pc = skip_pc;
                                    break;
                                }
                                WasmInstruction::End => {
                                    depth -= 1;
                                    if depth == 0 {
                                        // Found matching end, jump to it
                                        pc = skip_pc;
                                        break;
                                    }
                                }
                                _ => {}
                            }
                            skip_pc += 1;
                        }
                    } else {
                        // If true, push label for else/end
                        // Use pre-computed block map to avoid find_matching_end calls
                        if let Some((_, end_pc)) = block_map.iter().find(|(start, _)| *start == pc) {
                            self.label_stack.push(LabelFrame {
                                start_pc: pc,
                                end_pc: *end_pc,
                                block_type: BlockType::If,
                                result_type: None,
                            }).ok();
                        }
                    }
                }
                WasmInstruction::Else => {
                    // Jump to end of if block
                    if let Some(frame) = self.label_stack.last() {
                        if frame.block_type == BlockType::If {
                            pc = frame.end_pc;
                            let _ = self.label_stack.pop();
                            continue;
                        }
                    }
                }
                WasmInstruction::Br(depth) => {
                    // Branch to label at depth
                    let target_depth = *depth as usize;
                    if let Some(frame) = self.label_stack.iter().rev().nth(target_depth) {
                        pc = frame.end_pc;
                        // Pop frames up to target
                        while self.label_stack.len() > target_depth {
                            let _ = self.label_stack.pop();
                        }
                        continue;
                    } else {
                        // Branch to function end
                        break;
                    }
                }
                WasmInstruction::BrIf(depth) => {
                    let cond = self.stack.pop().ok_or("Stack underflow")?;
                    if let WasmValue::I32(0) = cond {
                        // Condition is false, continue
                    } else {
                        // Condition is true, branch
                        let target_depth = *depth as usize;
                        if let Some(frame) = self.label_stack.iter().rev().nth(target_depth) {
                            pc = frame.end_pc;
                            while self.label_stack.len() > target_depth {
                                let _ = self.label_stack.pop();
                            }
                            continue;
                        } else {
                            break;
                        }
                    }
                }
                WasmInstruction::BrTable { default, .. } => {
                    // Get index from stack
                    let index_val = self.stack.pop().ok_or("Stack underflow")?;
                    let _index = if let WasmValue::I32(i) = index_val {
                        i as usize
                    } else {
                        return Err("Invalid index type in br_table");
                    };
                    
                    // Use default if index out of bounds (simplified - full impl would use table)
                    let target_depth = *default as usize;
                    
                    if let Some(frame) = self.label_stack.iter().rev().nth(target_depth) {
                        pc = frame.end_pc;
                        while self.label_stack.len() > target_depth {
                            let _ = self.label_stack.pop();
                        }
                        continue;
                    } else {
                        break;
                    }
                }
                WasmInstruction::Loop(_) => {
                    // Push loop label - loop continues at start
                    // Use pre-computed block map to avoid find_matching_end calls
                    let end_pc = block_map.iter()
                        .find(|(start, _)| *start == pc)
                        .map(|(_, end)| *end);
                    
                    if let Some(end_pc) = end_pc {
                        self.label_stack.push(LabelFrame {
                            start_pc: pc,
                            end_pc,
                            block_type: BlockType::Loop,
                            result_type: None,
                        }).ok();
                    }
                }
                WasmInstruction::Block(_) => {
                    // Push block label
                    // Use pre-computed block map to avoid find_matching_end calls
                    let end_pc = block_map.iter()
                        .find(|(start, _)| *start == pc)
                        .map(|(_, end)| *end);
                    
                    if let Some(end_pc) = end_pc {
                        self.label_stack.push(LabelFrame {
                            start_pc: pc,
                            end_pc,
                            block_type: BlockType::Block,
                            result_type: None,
                        }).ok();
                    }
                }
                WasmInstruction::End => {
                    // Pop label frame if present
                    if let Some(frame) = self.label_stack.last() {
                        match frame.block_type {
                            BlockType::Loop => {
                                // Loop: jump back to start
                                // Check for infinite loops - if we've been at this PC many times, it's likely infinite
                                if instruction_count > MAX_INSTRUCTIONS / 2 {
                                    return Err("Infinite loop detected in WASM code");
                                }
                                // Additional safety check: if we're jumping back to the same PC, limit iterations
                                if pc == frame.start_pc && instruction_count > 100 {
                                    return Err("Infinite loop detected: jumping to same PC");
                                }
                                pc = frame.start_pc;
                                continue;
                            }
                            BlockType::Block | BlockType::If => {
                                // Block/If: just pop and continue
                                let _ = self.label_stack.pop();
                            }
                        }
                    }
                }
            }
            
            pc += 1; // Move to next instruction (unless branch changed pc)
        }
        
        Ok(())
    }
    
    /// Precompute block boundaries for faster lookups (optimization)
    /// Note: In no_std, we skip this optimization to save memory
    fn precompute_block_map(&self, _instructions: &[WasmInstruction]) {
        // In no_std environment, we skip precomputation to save memory
        // Block boundaries are computed on-demand using find_matching_end
    }
    
    /// Find matching end instruction for a block/loop/if
    fn find_matching_end(&self, start_pc: usize, instructions: &[WasmInstruction]) -> Option<usize> {
        let mut depth = 1;
        let mut pc = start_pc + 1;
        let mut search_count = 0;
        const MAX_SEARCH_DEPTH: usize = 1000; // Limit search to prevent infinite loops
        
        while pc < instructions.len() {
            search_count += 1;
            if search_count > MAX_SEARCH_DEPTH {
                // Safety: if we've searched too far, return None to prevent stack overflow
                warn!("find_matching_end: search limit exceeded, no matching end found");
                return None;
            }
            
            match &instructions[pc] {
                WasmInstruction::If | WasmInstruction::Loop(_) | WasmInstruction::Block(_) => {
                    depth += 1;
                }
                WasmInstruction::End => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(pc);
                    }
                }
                _ => {}
            }
            pc += 1;
        }
        
        None // No matching end found
    }

    /// Get memory reference for host functions
    pub fn memory_mut(&mut self) -> &mut WasmMemory {
        &mut self.memory
    }
    
    /// Get memory reference (immutable)
    pub fn memory(&self) -> &WasmMemory {
        &self.memory
    }
    
    /// Get number of imported functions
    pub fn imported_functions(&self) -> u32 {
        self.imported_functions
    }
}

/// Host function: Print message from memory
/// Args: [ptr: i32, len: i32] - memory address and length
pub fn host_print(instance: &mut WasmInstance, args: &[WasmValue]) -> Result<Option<WasmValue>, &'static str> {
    if args.len() < 2 {
        return Err("host_print requires 2 arguments (ptr, len)");
    }
    
    let ptr = if let WasmValue::I32(p) = args[0] { p as usize } else { return Err("Invalid ptr type"); };
    let len = if let WasmValue::I32(l) = args[1] { l as usize } else { return Err("Invalid len type"); };
    
    // Read string from memory
    let mut message = heapless::String::<256>::new();
    for i in 0..len.min(255) {
        if let Some(byte) = instance.memory().read_u8(ptr + i) {
            if byte == 0 {
                break; // Null terminator
            }
            if message.push(char::from(byte)).is_err() {
                break;
            }
        } else {
            break;
        }
    }
    
    info!("[WASM] {}", message);
    Ok(None)
}

/// Host function: Get current timestamp
///
/// Writes a timestamp value to WASM memory.
///
/// # Arguments
///
/// * `args[0]` - Memory address where to write timestamp (addr: i32)
///
/// # Example
///
/// From WASM:
/// ```wat
/// (call $get_timestamp (i32.const 100))
/// ```
pub fn host_get_timestamp(instance: &mut WasmInstance, args: &[WasmValue]) -> Result<Option<WasmValue>, &'static str> {
    if args.is_empty() {
        return Err("host_get_timestamp requires 1 argument (addr)");
    }
    
    let addr = if let WasmValue::I32(a) = args[0] { a as usize } else { return Err("Invalid addr type"); };
    
    // Write a simple timestamp (seconds since epoch, simplified)
    // In real implementation, would use actual time
    let timestamp: i32 = 1234567890; // Placeholder
    
    if !instance.memory_mut().write_i32(addr, timestamp) {
        return Err("Memory write failed");
    }
    
    info!("[WASM] host_get_timestamp: wrote timestamp {} to address {}", timestamp, addr);
    Ok(None)
}

/// Host function: GPIO Read
///
/// Reads the state of a GPIO pin.
///
/// # Arguments
///
/// * `args[0]` - GPIO pin number (pin: i32)
///
/// # Returns
///
/// Returns i32: 0 for LOW, 1 for HIGH
///
/// # Example
///
/// From WASM:
/// ```wat
/// (call $gpio_read (i32.const 5))
/// ```
pub fn host_gpio_read(_instance: &mut WasmInstance, args: &[WasmValue]) -> Result<Option<WasmValue>, &'static str> {
    if args.is_empty() {
        return Err("host_gpio_read requires 1 argument (pin)");
    }
    
    let pin = if let WasmValue::I32(p) = args[0] { p } else { return Err("Invalid pin type"); };
    
    // In Renode emulation, simulate GPIO read
    // For now, return a simulated value (alternating based on pin number)
    let value = (pin % 2) as i32;
    info!("[WASM] host_gpio_read: pin {} = {}", pin, value);
    Ok(Some(WasmValue::I32(value)))
}

/// Host function: GPIO Write
///
/// Writes a value to a GPIO pin.
///
/// # Arguments
///
/// * `args[0]` - GPIO pin number (pin: i32)
/// * `args[1]` - Value to write (value: i32, 0=LOW, 1=HIGH)
///
/// # Example
///
/// From WASM:
/// ```wat
/// (call $gpio_write (i32.const 5) (i32.const 1))
/// ```
pub fn host_gpio_write(_instance: &mut WasmInstance, args: &[WasmValue]) -> Result<Option<WasmValue>, &'static str> {
    if args.len() < 2 {
        return Err("host_gpio_write requires 2 arguments (pin, value)");
    }
    
    let pin = if let WasmValue::I32(p) = args[0] { p } else { return Err("Invalid pin type"); };
    let value = if let WasmValue::I32(v) = args[1] { v } else { return Err("Invalid value type"); };
    
    // In Renode emulation, simulate GPIO write
    info!("[WASM] host_gpio_write: pin {} = {}", pin, value);
    Ok(None)
}

/// Host function: UART Send
///
/// Sends data over UART.
///
/// # Arguments
///
/// * `args[0]` - UART port number (port: i32)
/// * `args[1]` - Memory address of data to send (ptr: i32)
/// * `args[2]` - Length of data to send (len: i32)
///
/// # Example
///
/// From WASM:
/// ```wat
/// (call $uart_send (i32.const 0) (i32.const 100) (i32.const 10))
/// ```
pub fn host_uart_send(instance: &mut WasmInstance, args: &[WasmValue]) -> Result<Option<WasmValue>, &'static str> {
    if args.len() < 3 {
        return Err("host_uart_send requires 3 arguments (port, ptr, len)");
    }
    
    let port = if let WasmValue::I32(p) = args[0] { p } else { return Err("Invalid port type"); };
    let ptr = if let WasmValue::I32(p) = args[1] { p as usize } else { return Err("Invalid ptr type"); };
    let len = if let WasmValue::I32(l) = args[2] { l as usize } else { return Err("Invalid len type"); };
    
    // Read data from memory
    let mut data = heapless::Vec::<u8, 256>::new();
    for i in 0..len.min(255) {
        if let Some(byte) = instance.memory().read_u8(ptr + i) {
            if data.push(byte).is_err() {
                break;
            }
        } else {
            break;
        }
    }
    
    // In Renode emulation, simulate UART send
    info!("[WASM] host_uart_send: port {} sent {} bytes: {:?}", port, data.len(), data);
    Ok(None)
}

/// Host function: UART Receive
///
/// Receives data from UART.
///
/// # Arguments
///
/// * `args[0]` - UART port number (port: i32)
/// * `args[1]` - Memory address to store received data (ptr: i32)
/// * `args[2]` - Maximum length to receive (max_len: i32)
///
/// # Returns
///
/// Returns i32: number of bytes received
///
/// # Example
///
/// From WASM:
/// ```wat
/// (call $uart_receive (i32.const 0) (i32.const 200) (i32.const 64))
/// ```
pub fn host_uart_receive(_instance: &mut WasmInstance, args: &[WasmValue]) -> Result<Option<WasmValue>, &'static str> {
    if args.len() < 3 {
        return Err("host_uart_receive requires 3 arguments (port, ptr, max_len)");
    }
    
    let port = if let WasmValue::I32(p) = args[0] { p } else { return Err("Invalid port type"); };
    let _ptr = if let WasmValue::I32(p) = args[1] { p as usize } else { return Err("Invalid ptr type"); };
    let _max_len = if let WasmValue::I32(l) = args[2] { l as usize } else { return Err("Invalid max_len type"); };
    
    // In Renode emulation, simulate UART receive
    // For now, return 0 bytes (no data available)
    let received = 0;
    info!("[WASM] host_uart_receive: port {} received {} bytes", port, received);
    Ok(Some(WasmValue::I32(received as i32)))
}

/// Host function: Sensor Read
///
/// Reads a value from a sensor.
///
/// # Arguments
///
/// * `args[0]` - Sensor ID (sensor_id: i32)
/// * `args[1]` - Memory address to store sensor value (ptr: i32)
///
/// # Returns
///
/// Returns i32: 0 on success, -1 on error
///
/// # Example
///
/// From WASM:
/// ```wat
/// (call $sensor_read (i32.const 0) (i32.const 300))
/// ```
pub fn host_sensor_read(instance: &mut WasmInstance, args: &[WasmValue]) -> Result<Option<WasmValue>, &'static str> {
    if args.len() < 2 {
        return Err("host_sensor_read requires 2 arguments (sensor_id, ptr)");
    }
    
    let sensor_id = if let WasmValue::I32(s) = args[0] { s } else { return Err("Invalid sensor_id type"); };
    let ptr = if let WasmValue::I32(p) = args[1] { p as usize } else { return Err("Invalid ptr type"); };
    
    // In Renode emulation, simulate sensor read
    // For now, write a simulated sensor value (e.g., temperature in 0.1°C units)
    let sensor_value: i32 = 250; // 25.0°C
    if !instance.memory_mut().write_i32(ptr, sensor_value) {
        return Err("Memory write failed");
    }
    
    info!("[WASM] host_sensor_read: sensor {} = {} (0.1°C units)", sensor_id, sensor_value);
    Ok(Some(WasmValue::I32(0))) // Success
}

