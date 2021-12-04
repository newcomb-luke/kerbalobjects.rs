pub mod sections;

pub mod symbols;

pub mod instructions;

pub mod errors;

use std::iter::Peekable;
use std::slice::Iter;

use crate::FromBytes;

use crate::errors::{ReadError, ReadResult};

use self::sections::{ReldSection, SectionIndex, SectionKind};
use self::{
    errors::{UpdateError, UpdateResult},
    sections::{DataSection, FuncSection, SectionHeader, StringTable, SymbolTable},
};

use super::ToBytes;

pub use instructions::Instr;

const FILE_VERSION: u8 = 4;
const MAGIC_NUMBER: u32 = 0x666f016b;

pub trait SectionFromBytes {
    fn from_bytes(
        source: &mut Peekable<Iter<u8>>,
        debug: bool,
        size: usize,
        section_index: usize,
    ) -> ReadResult<Self>
    where
        Self: Sized;
}

macro_rules! gen_get_by_name {
    ($func_name: ident, $mut_func_name: ident, $section_name: ident, $section_type: ty) => {
        pub fn $func_name(&self, name: &str) -> Option<&$section_type> {
            self.$section_name.iter().find(|section| {
                match self.sh_name_from_index(section.section_index()) {
                    Some(s) => s == name,
                    None => false,
                }
            })
        }

        pub fn $mut_func_name(&mut self, name: &str) -> Option<&mut $section_type> {
            let mut index_opt = Some(0);

            for (index, section) in self.$section_name.iter().enumerate() {
                match self.sh_name_from_index(section.section_index()) {
                    Some(s) => {
                        if s == name {
                            index_opt = Some(index);
                            break;
                        }
                    }
                    None => continue,
                }
            }

            let index = index_opt?;

            self.$section_name.iter_mut().nth(index)
        }
    };
}

pub struct KOFile {
    header: KOHeader,
    sh_strtab: StringTable,
    section_headers: Vec<SectionHeader>,
    str_tabs: Vec<StringTable>,
    sym_tabs: Vec<SymbolTable>,
    data_sections: Vec<DataSection>,
    func_sections: Vec<FuncSection>,
    reld_sections: Vec<ReldSection>,
}

impl KOFile {
    pub fn new() -> Self {
        let mut kofile = KOFile {
            header: KOHeader::new(2, 2),
            sh_strtab: StringTable::new(256, 1),
            section_headers: Vec::with_capacity(4),
            str_tabs: Vec::with_capacity(2),
            sym_tabs: Vec::with_capacity(1),
            data_sections: Vec::with_capacity(1),
            func_sections: Vec::with_capacity(1),
            reld_sections: Vec::with_capacity(1),
        };

        kofile.add_header(SectionHeader::new(0, sections::SectionKind::Null));

        kofile.add_header(SectionHeader::new(1, sections::SectionKind::StrTab));

        kofile.sh_strtab.add(".shstrtab");

        kofile
    }

    pub fn add_shstr(&mut self, name: &str) -> usize {
        self.sh_strtab.add(name)
    }

    pub fn get_shstr(&self, index: usize) -> Option<&String> {
        self.sh_strtab.get(index)
    }

    pub fn add_header(&mut self, header: SectionHeader) -> usize {
        let index = self.section_headers.len();
        self.section_headers.push(header);
        index
    }

    pub fn get_header(&self, index: usize) -> Option<&SectionHeader> {
        self.section_headers.get(index)
    }

    pub fn get_header_name(&self, header: &SectionHeader) -> Option<&String> {
        self.sh_strtab.get(header.name_idx())
    }

    pub fn sh_name_from_index(&self, index: usize) -> Option<&String> {
        let header = self.section_headers.get(index)?;
        self.sh_strtab.get(header.name_idx())
    }

    pub fn add_str_tab(&mut self, str_tab: StringTable) {
        self.str_tabs.push(str_tab);
    }

    pub fn add_sym_tab(&mut self, sym_tab: SymbolTable) {
        self.sym_tabs.push(sym_tab);
    }

    pub fn add_data_section(&mut self, data_section: DataSection) {
        self.data_sections.push(data_section);
    }

    pub fn add_func_section(&mut self, func_section: FuncSection) {
        self.func_sections.push(func_section);
    }

    pub fn add_reld_section(&mut self, reld_section: ReldSection) {
        self.reld_sections.push(reld_section);
    }

    pub fn str_tabs(&self) -> Iter<StringTable> {
        self.str_tabs.iter()
    }

    pub fn sym_tabs(&self) -> Iter<SymbolTable> {
        self.sym_tabs.iter()
    }

    pub fn data_sections(&self) -> Iter<DataSection> {
        self.data_sections.iter()
    }

    pub fn func_sections(&self) -> Iter<FuncSection> {
        self.func_sections.iter()
    }

    pub fn reld_sections(&self) -> Iter<ReldSection> {
        self.reld_sections.iter()
    }

    pub fn new_sh(&mut self, name: &str, kind: SectionKind) -> usize {
        let name_idx = self.add_shstr(name);
        let header = SectionHeader::new(name_idx, kind);
        self.add_header(header)
    }

    pub fn new_strtab(&mut self, name: &str) -> StringTable {
        let sh_index = self.new_sh(name, SectionKind::StrTab);
        StringTable::new(4, sh_index)
    }

    pub fn new_symtab(&mut self, name: &str) -> SymbolTable {
        let sh_index = self.new_sh(name, SectionKind::SymTab);
        SymbolTable::new(4, sh_index)
    }

    pub fn new_datasection(&mut self, name: &str) -> DataSection {
        let sh_index = self.new_sh(name, SectionKind::Data);
        DataSection::new(4, sh_index)
    }

    pub fn new_funcsection(&mut self, name: &str) -> FuncSection {
        let sh_index = self.new_sh(name, SectionKind::Func);
        FuncSection::new(4, sh_index)
    }

    pub fn new_reldsection(&mut self, name: &str) -> ReldSection {
        let sh_index = self.new_sh(name, SectionKind::Reld);
        ReldSection::new(4, sh_index)
    }

    pub fn sh_index_by_name(&self, name: &str) -> Option<usize> {
        for (index, header) in self.section_headers.iter().enumerate() {
            let name_idx = header.name_idx();
            let sh_name = match self.sh_strtab.get(name_idx) {
                Some(name) => name,
                None => {
                    continue;
                }
            };

            if name == sh_name {
                return Some(index);
            }
        }

        None
    }

    gen_get_by_name!(str_tab_by_name, str_tab_by_name_mut, str_tabs, StringTable);

    gen_get_by_name!(sym_tab_by_name, sym_tab_by_name_mut, sym_tabs, SymbolTable);

    gen_get_by_name!(
        data_section_by_name,
        data_section_by_name_mut,
        data_sections,
        DataSection
    );

    gen_get_by_name!(
        func_section_by_name,
        func_section_by_name_mut,
        func_sections,
        FuncSection
    );

    gen_get_by_name!(
        reld_section_by_name,
        reld_section_by_name_mut,
        reld_sections,
        ReldSection
    );

    pub fn section_headers(&self) -> Iter<SectionHeader> {
        self.section_headers.iter()
    }

    pub fn section_count(&self) -> usize {
        self.section_headers.len()
    }

    pub fn version(&self) -> u8 {
        self.header.version
    }

    pub fn sh_strtab_index(&self) -> usize {
        self.sh_strtab.section_index()
    }

    fn update_section_header(
        &mut self,
        section_name: &'static str,
        section_index: usize,
        section_size: u32,
    ) -> UpdateResult {
        match self.section_headers.get_mut(section_index) {
            Some(header) => {
                header.set_size(section_size);
            }
            None => {
                return Err(UpdateError::InvalidSectionIndexError(
                    section_name,
                    section_index,
                ));
            }
        }

        Ok(())
    }

    fn update_section_headers(&mut self) -> UpdateResult {
        self.section_headers
            .get_mut(1)
            .unwrap()
            .set_size(self.sh_strtab.size());

        for i in 0..self.str_tabs.len() {
            let str_tab = self.str_tabs.get(i).unwrap();
            let section_index = str_tab.section_index();
            let size = str_tab.size();

            self.update_section_header("StringTable", section_index, size)?;
        }

        for i in 0..self.sym_tabs.len() {
            let sym_tab = self.sym_tabs.get(i).unwrap();
            let section_index = sym_tab.section_index();
            let size = sym_tab.size();

            self.update_section_header("SymbolTable", section_index, size)?;
        }

        for i in 0..self.data_sections.len() {
            let data_section = self.data_sections.get(i).unwrap();
            let section_index = data_section.section_index();
            let size = data_section.size();

            self.update_section_header("DataSection", section_index, size)?;
        }

        for i in 0..self.func_sections.len() {
            let func_section = self.func_sections.get(i).unwrap();
            let section_index = func_section.section_index();
            let size = func_section.size();

            self.update_section_header("FuncSection", section_index, size)?;
        }

        for i in 0..self.reld_sections.len() {
            let reld_section = self.reld_sections.get(i).unwrap();
            let section_index = reld_section.section_index();
            let size = reld_section.size();

            self.update_section_header("ReldSection", section_index, size)?;
        }

        Ok(())
    }

    pub fn update_headers(&mut self) -> UpdateResult {
        self.header = KOHeader::new(
            self.section_headers.len() as u16,
            self.sh_strtab.section_index() as u16,
        );

        self.update_section_headers()?;

        Ok(())
    }
}

impl ToBytes for KOFile {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        self.header.to_bytes(buf);

        // Arbitrary "large number"
        let mut section_buffer: Vec<u8> = Vec::with_capacity(2048);

        for i in 0..self.section_headers.len() {
            self.section_headers.get(i).unwrap().to_bytes(buf);

            if i == 1 {
                self.sh_strtab.to_bytes(&mut section_buffer);
            } else if i != 0 {
                for str_tab in self.str_tabs.iter() {
                    if str_tab.section_index() == i {
                        str_tab.to_bytes(&mut section_buffer);
                        continue;
                    }
                }

                for sym_tab in self.sym_tabs.iter() {
                    if sym_tab.section_index() == i {
                        sym_tab.to_bytes(&mut section_buffer);
                        continue;
                    }
                }

                for data_section in self.data_sections.iter() {
                    if data_section.section_index() == i {
                        data_section.to_bytes(&mut section_buffer);
                        continue;
                    }
                }

                for func_section in self.func_sections.iter() {
                    if func_section.section_index() == i {
                        func_section.to_bytes(&mut section_buffer);
                        continue;
                    }
                }

                for reld_section in self.reld_sections.iter() {
                    if reld_section.section_index() == i {
                        reld_section.to_bytes(&mut section_buffer);
                        continue;
                    }
                }
            }
        }

        buf.append(&mut section_buffer);
    }
}

impl FromBytes for KOFile {
    fn from_bytes(source: &mut Peekable<Iter<u8>>, debug: bool) -> ReadResult<Self>
    where
        Self: Sized,
    {
        let header = KOHeader::from_bytes(source, debug)?;
        let mut section_headers = Vec::with_capacity(header.num_headers() as usize);
        let mut str_tabs;
        let mut sym_tabs;
        let mut data_sections;
        let mut func_sections;
        let mut reld_sections;
        let mut num_str_tabs = 0;
        let mut num_sym_tabs = 0;
        let mut num_data_sections = 0;
        let mut num_func_sections = 0;
        let mut num_reld_sections = 0;

        for _ in 0..header.num_headers {
            let header = SectionHeader::from_bytes(source, debug)?;

            match header.kind() {
                sections::SectionKind::StrTab => {
                    num_str_tabs += 1;
                }
                sections::SectionKind::SymTab => {
                    num_sym_tabs += 1;
                }
                sections::SectionKind::Data => {
                    num_data_sections += 1;
                }
                sections::SectionKind::Func => {
                    num_func_sections += 1;
                }
                sections::SectionKind::Reld => {
                    num_reld_sections += 1;
                }
                sections::SectionKind::Null
                | sections::SectionKind::Unknown
                | sections::SectionKind::Debug => {}
            }

            if debug {
                println!(
                    "Section header: Name index: {}, kind: {:?}, size: {}",
                    header.name_idx(),
                    header.kind(),
                    header.size()
                );
            }

            section_headers.push(header);
        }

        // Allocate now
        str_tabs = Vec::with_capacity(num_str_tabs);
        sym_tabs = Vec::with_capacity(num_sym_tabs);
        data_sections = Vec::with_capacity(num_data_sections);
        func_sections = Vec::with_capacity(num_func_sections);
        reld_sections = Vec::with_capacity(num_reld_sections);

        let strtab_size = section_headers
            .get(1)
            .ok_or(ReadError::MissingSectionError(".shstrtab"))?
            .size() as usize;

        let sh_strtab = StringTable::from_bytes(source, debug, strtab_size, 1)?;

        for i in 2..section_headers.len() {
            let header = section_headers.get(i).unwrap();
            let size = header.size() as usize;

            match header.kind() {
                sections::SectionKind::Null => {}
                sections::SectionKind::StrTab => {
                    let str_tab = StringTable::from_bytes(source, debug, size, i)?;
                    str_tabs.push(str_tab);
                }
                sections::SectionKind::SymTab => {
                    let sym_tab = SymbolTable::from_bytes(source, debug, size, i)?;
                    sym_tabs.push(sym_tab);
                }
                sections::SectionKind::Data => {
                    let data_section = DataSection::from_bytes(source, debug, size, i)?;
                    data_sections.push(data_section);
                }
                sections::SectionKind::Func => {
                    let func_section = FuncSection::from_bytes(source, debug, size, i)?;
                    func_sections.push(func_section);
                }
                sections::SectionKind::Reld => {
                    let reld_section = ReldSection::from_bytes(source, debug, size, i)?;
                    reld_sections.push(reld_section);
                }
                sections::SectionKind::Debug => {
                    return Err(ReadError::DebugSectionUnsupportedError);
                }
                sections::SectionKind::Unknown => {}
            }
        }

        Ok(KOFile {
            header,
            sh_strtab,
            section_headers,
            str_tabs,
            sym_tabs,
            data_sections,
            func_sections,
            reld_sections,
        })
    }
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

    pub fn size_bytes() -> usize {
        9
    }
}

impl ToBytes for KOHeader {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        self.magic.to_bytes(buf);
        self.version.to_bytes(buf);
        self.num_headers.to_bytes(buf);
        self.strtab_idx.to_bytes(buf);
    }
}

impl FromBytes for KOHeader {
    fn from_bytes(source: &mut Peekable<Iter<u8>>, debug: bool) -> ReadResult<Self>
    where
        Self: Sized,
    {
        let magic = u32::from_bytes(source, debug)
            .map_err(|_| ReadError::KOHeaderReadError("file magic"))?;
        let version =
            u8::from_bytes(source, debug).map_err(|_| ReadError::KOHeaderReadError("version"))?;
        let num_headers = u16::from_bytes(source, debug)
            .map_err(|_| ReadError::KOHeaderReadError("number of headers"))?;
        let strtab_idx = u16::from_bytes(source, debug)
            .map_err(|_| ReadError::KOHeaderReadError("string table index"))?;

        if magic != MAGIC_NUMBER {
            return Err(ReadError::InvalidKOFileMagicError);
        }

        if version != FILE_VERSION {
            return Err(ReadError::VersionMismatchError(version));
        }

        Ok(KOHeader {
            magic,
            version,
            num_headers,
            strtab_idx,
        })
    }
}
