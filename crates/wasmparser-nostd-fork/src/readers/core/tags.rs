use alloc::rc::Rc;
use alloc::{vec::Vec, string::String, boxed::Box};

use crate::{BinaryReader, FromReader, Result, SectionLimited, TagKind, TagType};

/// A reader for the tags section of a WebAssembly module.
pub type TagSectionReader<'a> = SectionLimited<'a, TagType>;

impl<'a> FromReader<'a> for TagType {
    fn from_reader(reader: &mut BinaryReader<'a>) -> Result<Self> {
        let attribute = reader.read_u8()?;
        if attribute != 0 {
            bail!(reader.original_position() - 1, "invalid tag attributes");
        }
        Ok(TagType {
            kind: TagKind::Exception,
            func_type_idx: reader.read_var_u32()?,
        })
    }
}
