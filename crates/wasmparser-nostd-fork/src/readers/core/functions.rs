use alloc::rc::Rc;
use alloc::{vec::Vec, string::String, boxed::Box};

/// A reader for the function section of a WebAssembly module.
pub type FunctionSectionReader<'a> = crate::SectionLimited<'a, u32>;
