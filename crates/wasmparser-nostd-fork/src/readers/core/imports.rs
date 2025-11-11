use alloc::rc::Rc;
use alloc::{vec::Vec, string::String, boxed::Box};

use crate::{
    BinaryReader, ExternalKind, FromReader, GlobalType, MemoryType, Result, SectionLimited,
    TableType, TagType,
};

/// Represents a reference to a type definition in a WebAssembly module.
#[derive(Debug, Clone, Copy)]
pub enum TypeRef {
    /// The type is a function.
    ///
    /// The value is an index into the type section.
    Func(u32),
    /// The type is a table.
    Table(TableType),
    /// The type is a memory.
    Memory(MemoryType),
    /// The type is a global.
    Global(GlobalType),
    /// The type is a tag.
    ///
    /// This variant is only used for the exception handling proposal.
    ///
    /// The value is an index in the types index space.
    Tag(TagType),
}

/// Represents an import in a WebAssembly module.
#[derive(Debug, Copy, Clone)]
pub struct Import<'a> {
    /// The module being imported from.
    pub module: &'a str,
    /// The name of the imported item.
    pub name: &'a str,
    /// The type of the imported item.
    pub ty: TypeRef,
}

/// A reader for the import section of a WebAssembly module.
pub type ImportSectionReader<'a> = SectionLimited<'a, Import<'a>>;

impl<'a> FromReader<'a> for Import<'a> {
    fn from_reader(reader: &mut BinaryReader<'a>) -> Result<Self> {
        Ok(Import {
            module: reader.read()?,
            name: reader.read()?,
            ty: reader.read()?,
        })
    }
}

impl<'a> FromReader<'a> for TypeRef {
    fn from_reader(reader: &mut BinaryReader<'a>) -> Result<Self> {
        Ok(match reader.read()? {
            ExternalKind::Func => TypeRef::Func(reader.read_var_u32()?),
            ExternalKind::Table => TypeRef::Table(reader.read()?),
            ExternalKind::Memory => TypeRef::Memory(reader.read()?),
            ExternalKind::Global => TypeRef::Global(reader.read()?),
            ExternalKind::Tag => TypeRef::Tag(reader.read()?),
        })
    }
}
