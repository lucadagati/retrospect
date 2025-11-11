use alloc::rc::Rc;
use alloc::{vec::Vec, string::String, boxed::Box};

use crate::{BinaryReader, FromReader, Result, SectionLimited};

/// A reader for the export section of a WebAssembly module.
pub type ExportSectionReader<'a> = SectionLimited<'a, Export<'a>>;

/// External types as defined [here].
///
/// [here]: https://webassembly.github.io/spec/core/syntax/types.html#external-types
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ExternalKind {
    /// The external kind is a function.
    Func,
    /// The external kind if a table.
    Table,
    /// The external kind is a memory.
    Memory,
    /// The external kind is a global.
    Global,
    /// The external kind is a tag.
    Tag,
}

/// Represents an export in a WebAssembly module.
#[derive(Debug, Copy, Clone)]
pub struct Export<'a> {
    /// The name of the exported item.
    pub name: &'a str,
    /// The kind of the export.
    pub kind: ExternalKind,
    /// The index of the exported item.
    pub index: u32,
}

impl<'a> FromReader<'a> for Export<'a> {
    fn from_reader(reader: &mut BinaryReader<'a>) -> Result<Self> {
        Ok(Export {
            name: reader.read_string()?,
            kind: reader.read()?,
            index: reader.read_var_u32()?,
        })
    }
}

impl<'a> FromReader<'a> for ExternalKind {
    fn from_reader(reader: &mut BinaryReader<'a>) -> Result<Self> {
        let offset = reader.original_position();
        let byte = reader.read_u8()?;
        BinaryReader::external_kind_from_byte(byte, offset)
    }
}
