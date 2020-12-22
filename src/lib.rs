
mod relinstruction;
pub use relinstruction::*;

mod sections;
pub use sections::*;

mod reader;
pub use reader::*;

mod writer;
pub use writer::*;

mod sectionheaders;
pub use sectionheaders::*;

mod kofile;
pub use kofile::*;

pub static FILE_VERSION: u8 = 3;
pub static MAGIC_NUMBER: u32 = 0x666f016b;
