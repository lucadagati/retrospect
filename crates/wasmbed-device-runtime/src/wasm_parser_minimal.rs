// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

// Minimal WASM parser for no_std environments
// Implements only the essential parts needed by wasmbed-device-runtime

use heapless::{Vec, String};

/// WASM value types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValType {
    I32,
    I64,
    F32,
    F64,
}

/// WASM operator (simplified subset)
#[derive(Debug, Clone, Copy)]
pub enum Operator {
    I32Const { value: i32 },
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
    I32LeS,
    I32GtS,
    I32GeS,
    I32LtU,
    I32LeU,
    I32GtU,
    I32GeU,
    I32Load { align: u32, offset: u32 },
    I32Store { align: u32, offset: u32 },
    I32Load8S { align: u32, offset: u32 },
    I32Load8U { align: u32, offset: u32 },
    I32Store8 { align: u32, offset: u32 },
    I32Load16S { align: u32, offset: u32 },
    I32Load16U { align: u32, offset: u32 },
    I32Store16 { align: u32, offset: u32 },
    // I64 operators
    I64Const { value: i64 },
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
    I64LeS,
    I64GtS,
    I64GeS,
    I64LtU,
    I64LeU,
    I64GtU,
    I64GeU,
    I64Load { align: u32, offset: u32 },
    I64Store { align: u32, offset: u32 },
    // Memory arg struct for compatibility
    // F32 operators
    F32Const { value: f32 },
    F32Add,
    F32Sub,
    F32Mul,
    F32Div,
    F32Eq,
    F32Ne,
    F32Lt,
    F32Le,
    F32Gt,
    F32Ge,
    F32Load { align: u32, offset: u32 },
    F32Store { align: u32, offset: u32 },
    // F64 operators
    F64Const { value: f64 },
    F64Add,
    F64Sub,
    F64Mul,
    F64Div,
    F64Eq,
    F64Ne,
    F64Lt,
    F64Le,
    F64Gt,
    F64Ge,
    F64Load { align: u32, offset: u32 },
    F64Store { align: u32, offset: u32 },
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
    MemorySize,
    MemoryGrow,
    If { blockty: u8 },
    Else,
    Br { relative_depth: u32 },
    BrIf { relative_depth: u32 },
    BrTable { default: u32 },
    Loop { blockty: u8 },
    Block { blockty: u8 },
    End,
    LocalGet { local_index: u32 },
    LocalSet { local_index: u32 },
    LocalTee { local_index: u32 },
    GlobalGet { global_index: u32 },
    GlobalSet { global_index: u32 },
    Call { function_index: u32 },
    Return,
    Drop,
    Select,
    Nop,
    Unreachable,
}

/// WASM import
#[derive(Debug, Clone)]
pub struct Import {
    pub module: String<32>,
    pub name: String<32>,
    pub ty: ImportType,
}

#[derive(Debug, Clone)]
pub enum ImportType {
    Function(u32),
    Memory { min: u32, max: Option<u32> },
}

/// WASM section payload (matches wasmparser API)
#[derive(Debug)]
pub enum Payload {
    ImportSection(ImportSectionReader),
    MemorySection(MemorySectionReader),
    CodeSectionEntry(CodeSectionEntryReader),
    TypeSection(TypeSectionReader),
    FunctionSection(FunctionSectionReader),
    End,
}

/// Import section reader (simplified)
#[derive(Debug)]
pub struct ImportSectionReader {
    imports: Vec<Import, 32>,
    index: usize,
}

impl ImportSectionReader {
    pub fn new(imports: Vec<Import, 32>) -> Self {
        Self { imports, index: 0 }
    }
}

impl Iterator for ImportSectionReader {
    type Item = Result<Import, &'static str>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.imports.len() {
            let import = self.imports[self.index].clone();
            self.index += 1;
            Some(Ok(import))
        } else {
            None
        }
    }
}

/// Memory section reader
#[derive(Debug)]
pub struct MemorySectionReader {
    min: u32,
    max: Option<u32>,
    read: bool,
}

impl MemorySectionReader {
    pub fn new(min: u32, max: Option<u32>) -> Self {
        Self { min, max, read: false }
    }
}

impl Iterator for MemorySectionReader {
    type Item = Result<MemoryType, &'static str>;
    fn next(&mut self) -> Option<Self::Item> {
        if !self.read {
            self.read = true;
            Some(Ok(MemoryType {
                initial: self.min,
                maximum: self.max,
            }))
        } else {
            None
        }
    }
}

// Debug: verify MemorySectionReader values
impl MemorySectionReader {
    pub fn get_min(&self) -> u32 { self.min }
    pub fn get_max(&self) -> Option<u32> { self.max }
}

/// Memory type (matches wasmparser API)
#[derive(Debug)]
pub struct MemoryType {
    pub initial: u32,
    pub maximum: Option<u32>,
}

/// Code section entry reader
#[derive(Debug)]
pub struct CodeSectionEntryReader {
    locals: Vec<(u32, ValType), 16>,
    operators: Vec<Operator, 256>,
    locals_read: bool,
}

impl CodeSectionEntryReader {
    pub fn new(locals: Vec<(u32, ValType), 16>, operators: Vec<Operator, 256>) -> Self {
        Self { locals, operators, locals_read: false }
    }

    pub fn get_locals_reader(&mut self) -> Result<LocalsReader, &'static str> {
        self.locals_read = true;
        Ok(LocalsReader { locals: &self.locals, index: 0 })
    }

    pub fn get_operators_reader(&self) -> Result<OperatorsReader, &'static str> {
        Ok(OperatorsReader { operators: &self.operators, index: 0 })
    }
}

/// Locals reader for code section
#[derive(Debug)]
pub struct LocalsReader<'a> {
    locals: &'a [(u32, ValType)],
    index: usize,
}

impl<'a> Iterator for LocalsReader<'a> {
    type Item = Result<(u32, ValType), &'static str>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.locals.len() {
            let (count, ty) = self.locals[self.index];
            self.index += 1;
            Some(Ok((count, ty)))
        } else {
            None
        }
    }
}

/// Operators reader for code section
#[derive(Debug)]
pub struct OperatorsReader<'a> {
    operators: &'a [Operator],
    index: usize,
}

impl<'a> OperatorsReader<'a> {
    pub fn eof(&self) -> bool {
        self.index >= self.operators.len()
    }

    pub fn read(&mut self) -> Result<Operator, &'static str> {
        if self.index < self.operators.len() {
            let op = self.operators[self.index];
            self.index += 1;
            Ok(op)
        } else {
            Err("End of operators")
        }
    }
}

/// Type section reader
#[derive(Debug)]
pub struct TypeSectionReader {
    types: Vec<FunctionType, 32>,
}

impl TypeSectionReader {
    pub fn new(types: Vec<FunctionType, 32>) -> Self {
        Self { types }
    }
}

/// Function section reader
#[derive(Debug)]
pub struct FunctionSectionReader {
    indices: Vec<u32, 32>,
}

impl FunctionSectionReader {
    pub fn new(indices: Vec<u32, 32>) -> Self {
        Self { indices }
    }
}

#[derive(Debug, Clone)]
pub struct FunctionType {
    pub params: Vec<ValType, 8>,
    pub results: Vec<ValType, 8>,
}

/// Minimal WASM parser
pub struct Parser {
    offset: usize,
}

impl Parser {
    pub fn new(_features: u64) -> Self {
        Self { offset: 0 }
    }

    pub fn parse_all<'a>(&'a mut self, bytecode: &'a [u8]) -> ParseIterator<'a> {
        ParseIterator {
            parser: self,
            bytecode,
            done: false,
        }
    }
}

pub struct ParseIterator<'a> {
    parser: &'a mut Parser,
    bytecode: &'a [u8],
    done: bool,
}

impl<'a> Iterator for ParseIterator<'a> {
    type Item = Result<Payload, &'static str>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done || self.parser.offset >= self.bytecode.len() {
            return None;
        }

        // Skip magic and version (already validated)
        if self.parser.offset < 8 {
            self.parser.offset = 8;
        }

        if self.parser.offset >= self.bytecode.len() {
            self.done = true;
            return Some(Ok(Payload::End));
        }

               // Parse WASM sections
               while self.parser.offset < self.bytecode.len() {
                   let section_id = self.bytecode[self.parser.offset];
                   // Debug: log section parsing
                   // eprintln!("[PARSER] Parsing section {} at offset {}", section_id, self.parser.offset);
                   self.parser.offset += 1;

            // Read section size (LEB128)
            let (size, size_bytes) = match read_leb128_u32(&self.bytecode[self.parser.offset..]) {
                Ok((s, b)) => (s, b),
                Err(_) => {
                    self.done = true;
                    return Some(Ok(Payload::End));
                }
            };
            self.parser.offset += size_bytes;

            match section_id {
                1 => { // Type section
                    // Skip type section data
                    self.parser.offset += size as usize;
                    // Simplified: return empty type section
                    return Some(Ok(Payload::TypeSection(TypeSectionReader::new(Vec::new()))));
                }
                2 => { // Import section
                    // Skip import section data
                    self.parser.offset += size as usize;
                    // Simplified: return empty import section
                    return Some(Ok(Payload::ImportSection(ImportSectionReader::new(Vec::new()))));
                }
                3 => { // Function section
                    // Skip function section data
                    self.parser.offset += size as usize;
                    // Simplified: return empty function section
                    return Some(Ok(Payload::FunctionSection(FunctionSectionReader::new(Vec::new()))));
                }
                5 => { // Memory section
                    // Parse memory limits
                    // Memory section structure: count (LEB128) + for each: flags + min + max?
                    let section_data_start = self.parser.offset;
                    if self.parser.offset < self.bytecode.len() {
                        // First read the count of memories (LEB128)
                        let (mem_count, mem_count_bytes) = read_leb128_u32(&self.bytecode[self.parser.offset..]).unwrap_or((1, 1));
                        self.parser.offset += mem_count_bytes;
                        
                        if mem_count == 0 {
                            // Skip entire section if no memories
                            self.parser.offset = section_data_start + size as usize;
                            return Some(Ok(Payload::MemorySection(MemorySectionReader::new(1, None))));
                        }
                        
                        // Read first memory (we only support one memory for now)
                        let flags = self.bytecode[self.parser.offset];
                        self.parser.offset += 1;
                        let (min, min_bytes) = read_leb128_u32(&self.bytecode[self.parser.offset..]).unwrap_or((1, 1));
                        self.parser.offset += min_bytes;
                        let max = if flags & 0x01 != 0 {
                            let (m, b) = read_leb128_u32(&self.bytecode[self.parser.offset..]).unwrap_or((1, 1));
                            self.parser.offset += b;
                            Some(m)
                        } else {
                            None
                        };
                        // Ensure we've consumed the entire section: section_data_start + size
                        self.parser.offset = section_data_start + size as usize;
                        return Some(Ok(Payload::MemorySection(MemorySectionReader::new(min, max))));
                    }
                }
                10 => { // Code section
                    // eprintln!("[PARSER] Found code section at offset {}", self.parser.offset);
                    // Parse function body
                    let section_data_start = self.parser.offset;
                    if self.parser.offset < self.bytecode.len() {
                        // First read the number of functions (LEB128)
                        let (func_count, func_count_bytes) = read_leb128_u32(&self.bytecode[self.parser.offset..]).unwrap_or((0, 1));
                        self.parser.offset += func_count_bytes;
                        
                        if func_count == 0 {
                            // Skip entire section
                            self.parser.offset = section_data_start + size as usize;
                            continue; // Skip if no functions
                        }
                        
                        // For each function, read body size
                        let (body_size, body_size_bytes) = read_leb128_u32(&self.bytecode[self.parser.offset..]).unwrap_or((0, 1));
                        self.parser.offset += body_size_bytes;
                        
                        // Parse locals
                        let (local_count, local_count_bytes) = read_leb128_u32(&self.bytecode[self.parser.offset..]).unwrap_or((0, 1));
                        self.parser.offset += local_count_bytes;
                        
                        let mut locals = Vec::new();
                        for _ in 0..local_count {
                            if self.parser.offset < self.bytecode.len() {
                                let (count, count_bytes) = read_leb128_u32(&self.bytecode[self.parser.offset..]).unwrap_or((1, 1));
                                self.parser.offset += count_bytes;
                                let val_type = if self.parser.offset < self.bytecode.len() {
                                    match self.bytecode[self.parser.offset] {
                                        0x7F => ValType::I32,
                                        0x7E => ValType::I64,
                                        0x7D => ValType::F32,
                                        0x7C => ValType::F64,
                                        _ => ValType::I32,
                                    }
                                } else {
                                    ValType::I32
                                };
                                self.parser.offset += 1;
                                let _ = locals.push((count, val_type));
                            }
                        }
                        
                        // Parse operators
                        // body_size is the total size of the function body (after body_size_bytes)
                        // It includes: local_count (1 byte) + locals (local_count * 2 bytes) + code
                        // body_data_start is where the body data starts (after func_count and body_size_bytes)
                        let body_data_start = self.parser.offset - func_count_bytes - body_size_bytes - local_count_bytes;
                        // Subtract the locals data we've already read
                        let locals_data_size = 1 + (local_count as usize * 2); // local_count (1) + locals (count+type per local)
                        // Code ends at: body_data_start + body_size
                        let code_end = body_data_start + body_size as usize;
                        
                        let mut operators = Vec::new();
                        // Parse until we reach the end of the body or find 'end'
                        while self.parser.offset < code_end && self.parser.offset < self.bytecode.len() {
                            let opcode = self.bytecode[self.parser.offset];
                            self.parser.offset += 1;
                            
                            match opcode {
                                0x0B => { // end
                                    operators.push(Operator::End).ok();
                                    break;
                                }
                                0x41 => { // i32.const
                                    let (value, bytes) = read_leb128_i32(&self.bytecode[self.parser.offset..]).unwrap_or((0, 1));
                                    self.parser.offset += bytes;
                                    operators.push(Operator::I32Const { value }).ok();
                                }
                                0x20 => { // local.get
                                    let (idx, bytes) = read_leb128_u32(&self.bytecode[self.parser.offset..]).unwrap_or((0, 1));
                                    self.parser.offset += bytes;
                                    operators.push(Operator::LocalGet { local_index: idx }).ok();
                                }
                                0x21 => { // local.set
                                    let (idx, bytes) = read_leb128_u32(&self.bytecode[self.parser.offset..]).unwrap_or((0, 1));
                                    self.parser.offset += bytes;
                                    operators.push(Operator::LocalSet { local_index: idx }).ok();
                                }
                                0x36 => { // i32.store
                                    let (align, align_bytes) = read_leb128_u32(&self.bytecode[self.parser.offset..]).unwrap_or((2, 1));
                                    self.parser.offset += align_bytes;
                                    let (offset, offset_bytes) = read_leb128_u32(&self.bytecode[self.parser.offset..]).unwrap_or((0, 1));
                                    self.parser.offset += offset_bytes;
                                    operators.push(Operator::I32Store { align, offset }).ok();
                                }
                                0x6A => { operators.push(Operator::I32Add).ok(); }
                                0x6B => { operators.push(Operator::I32Sub).ok(); }
                                0x6C => { operators.push(Operator::I32Mul).ok(); }
                                0x6D => { operators.push(Operator::I32DivS).ok(); }
                                0x6F => { operators.push(Operator::I32RemS).ok(); }
                                0x71 => { operators.push(Operator::I32Eq).ok(); }
                                0x46 => { operators.push(Operator::I32Eq).ok(); }
                                0x47 => { operators.push(Operator::I32Ne).ok(); }
                                0x48 => { operators.push(Operator::I32LtS).ok(); }
                                0x49 => { operators.push(Operator::I32LeS).ok(); }
                                0x4A => { operators.push(Operator::I32GtS).ok(); }
                                0x4B => { operators.push(Operator::I32GeS).ok(); }
                                0x4C => { operators.push(Operator::I32LtU).ok(); }
                                0x4D => { operators.push(Operator::I32LeU).ok(); }
                                0x4E => { operators.push(Operator::I32GtU).ok(); }
                                0x4F => { operators.push(Operator::I32GeU).ok(); }
                                0x28 => { // i32.load
                                    let (align, align_bytes) = read_leb128_u32(&self.bytecode[self.parser.offset..]).unwrap_or((2, 1));
                                    self.parser.offset += align_bytes;
                                    let (offset, offset_bytes) = read_leb128_u32(&self.bytecode[self.parser.offset..]).unwrap_or((0, 1));
                                    self.parser.offset += offset_bytes;
                                    operators.push(Operator::I32Load { align, offset }).ok();
                                }
                                _ => {
                                    // Skip unknown opcodes
                                }
                            }
                        }
                        
                        // Ensure we've consumed the entire section
                        // Skip to end of section: section_data_start + size
                        self.parser.offset = section_data_start + size as usize;
                        
                        return Some(Ok(Payload::CodeSectionEntry(CodeSectionEntryReader::new(locals, operators))));
                    }
                }
                _ => {
                    // Skip unknown sections
                    self.parser.offset += size as usize;
                }
            }
        }

        self.done = true;
        Some(Ok(Payload::End))
    }
}

/// Read LEB128 unsigned 32-bit integer
fn read_leb128_u32(data: &[u8]) -> Result<(u32, usize), &'static str> {
    let mut result = 0u32;
    let mut shift = 0;
    let mut bytes_read = 0;
    
    for &byte in data.iter().take(5) {
        bytes_read += 1;
        result |= ((byte & 0x7F) as u32) << shift;
        if (byte & 0x80) == 0 {
            return Ok((result, bytes_read));
        }
        shift += 7;
        if shift >= 32 {
            return Err("LEB128 overflow");
        }
    }
    
    Err("LEB128 incomplete")
}

/// Read LEB128 signed 32-bit integer
fn read_leb128_i32(data: &[u8]) -> Result<(i32, usize), &'static str> {
    let mut result = 0i32;
    let mut shift = 0;
    let mut bytes_read = 0;
    
    for &byte in data.iter().take(5) {
        bytes_read += 1;
        result |= ((byte & 0x7F) as i32) << shift;
        if (byte & 0x80) == 0 {
            // Sign extend
            if shift < 32 && (byte & 0x40) != 0 {
                result |= !0 << (shift + 7);
            }
            return Ok((result, bytes_read));
        }
        shift += 7;
        if shift >= 32 {
            return Err("LEB128 overflow");
        }
    }
    
    Err("LEB128 incomplete")
}

/// Validator (simplified - no-op for now)
pub struct Validator;

impl Validator {
    pub fn new() -> Self {
        Self
    }
}
