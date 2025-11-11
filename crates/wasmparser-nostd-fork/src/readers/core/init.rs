use alloc::rc::Rc;
use alloc::{vec::Vec, string::String, boxed::Box};

use crate::{BinaryReader, FromReader, OperatorsReader, Result};

/// Represents an initialization expression.
#[derive(Debug, Copy, Clone)]
pub struct ConstExpr<'a> {
    offset: usize,
    data: &'a [u8],
}

impl<'a> ConstExpr<'a> {
    /// Constructs a new `ConstExpr` from the given data and offset.
    pub fn new(data: &[u8], offset: usize) -> ConstExpr {
        ConstExpr { offset, data }
    }

    /// Gets a binary reader for the initialization expression.
    pub fn get_binary_reader(&self) -> BinaryReader<'a> {
        BinaryReader::new_with_offset(self.data, self.offset)
    }

    /// Gets an operators reader for the initialization expression.
    pub fn get_operators_reader(&self) -> OperatorsReader<'a> {
        OperatorsReader::new(self.get_binary_reader())
    }
}

impl<'a> FromReader<'a> for ConstExpr<'a> {
    fn from_reader(reader: &mut BinaryReader<'a>) -> Result<Self> {
        // FIXME(#188) ideally shouldn't need to skip here
        let reader = reader.skip(|r| r.skip_const_expr())?;
        Ok(ConstExpr::new(
            reader.remaining_buffer(),
            reader.original_position(),
        ))
    }
}
