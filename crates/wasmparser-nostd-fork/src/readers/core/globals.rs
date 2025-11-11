use alloc::rc::Rc;
use alloc::{vec::Vec, string::String, boxed::Box};

use crate::{BinaryReader, ConstExpr, FromReader, GlobalType, Result, SectionLimited};

/// Represents a core WebAssembly global.
#[derive(Debug, Copy, Clone)]
pub struct Global<'a> {
    /// The global's type.
    pub ty: GlobalType,
    /// The global's initialization expression.
    pub init_expr: ConstExpr<'a>,
}

/// A reader for the global section of a WebAssembly module.
pub type GlobalSectionReader<'a> = SectionLimited<'a, Global<'a>>;

impl<'a> FromReader<'a> for Global<'a> {
    fn from_reader(reader: &mut BinaryReader<'a>) -> Result<Self> {
        let ty = reader.read()?;
        let init_expr = reader.read()?;
        Ok(Global { ty, init_expr })
    }
}

impl<'a> FromReader<'a> for GlobalType {
    fn from_reader(reader: &mut BinaryReader<'a>) -> Result<Self> {
        Ok(GlobalType {
            content_type: reader.read()?,
            mutable: match reader.read_u8()? {
                0x00 => false,
                0x01 => true,
                _ => bail!(reader.original_position() - 1, "malformed mutability",),
            },
        })
    }
}
