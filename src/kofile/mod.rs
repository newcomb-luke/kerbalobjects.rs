pub mod sections;
use sections::{SectionHeader, StringTable};

pub mod symbols;

pub mod instructions;

use self::sections::{DataSection, SymbolTable};

use super::ToBytes;

const FILE_VERSION: u8 = 3;
const MAGIC_NUMBER: u32 = 0x666f016b;

pub struct KOFile<'a> {
    header: KOHeader,
    sh_strtab: StringTable,
    section_headers: Vec<SectionHeader<'a>>,
    str_tabs: Vec<StringTable>,
    sym_tabs: Vec<SymbolTable<'a>>,
    data_sections: Vec<DataSection<'a>>,
}

pub struct KOHeader {
    magic: u32,
    version: u8,
    num_headers: u16,
    strtab_idx: u16,
}

impl KOHeader {
    pub fn new(num_headers: u16, strtab_idx: u16) -> Self {
        KOHeader {
            magic: MAGIC_NUMBER,
            version: FILE_VERSION,
            num_headers,
            strtab_idx,
        }
    }

    pub fn version(&self) -> u8 {
        self.version
    }

    pub fn num_headers(&self) -> u16 {
        self.num_headers
    }

    pub fn strtab_idx(&self) -> u16 {
        self.strtab_idx
    }
}

impl ToBytes for KOHeader {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        super::push_32!(self.magic => buf);

        buf.push(self.version);

        super::push_16!(self.num_headers => buf);

        super::push_16!(self.strtab_idx => buf);
    }
}
