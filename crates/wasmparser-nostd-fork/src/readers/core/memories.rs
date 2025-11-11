use alloc::rc::Rc;
use alloc::{vec::Vec, string::String, boxed::Box};

use crate::{BinaryReader, FromReader, MemoryType, Result, SectionLimited};

/// A reader for the memory section of a WebAssembly module.
pub type MemorySectionReader<'a> = SectionLimited<'a, MemoryType>;

impl<'a> FromReader<'a> for MemoryType {
    fn from_reader(reader: &mut BinaryReader<'a>) -> Result<Self> {
        let pos = reader.original_position();
        let flags = reader.read_u8()?;
        if (flags & !0b111) != 0 {
            bail!(pos, "invalid memory limits flags");
        }

        let memory64 = flags & 0b100 != 0;
        let shared = flags & 0b010 != 0;
        let has_max = flags & 0b001 != 0;
        Ok(MemoryType {
            memory64,
            shared,
            // FIXME(WebAssembly/memory64#21) as currently specified if the
            // `shared` flag is set we should be reading a 32-bit limits field
            // here. That seems a bit odd to me at the time of this writing so
            // I've taken the liberty of reading a 64-bit limits field in those
            // situations. I suspect that this is a typo in the spec, but if not
            // we'll need to update this to read a 32-bit limits field when the
            // shared flag is set.
            initial: if memory64 {
                reader.read_var_u64()?
            } else {
                reader.read_var_u32()?.into()
            },
            maximum: if !has_max {
                None
            } else if memory64 {
                Some(reader.read_var_u64()?)
            } else {
                Some(reader.read_var_u32()?.into())
            },
        })
    }
}
