pub mod sections;

pub mod symbols;

pub mod instructions;

pub mod errors;

use std::iter::Peekable;
use std::slice::Iter;

use crate::FromBytes;

use crate::errors::{ReadError, ReadResult};

use self::sections::{SectionIndex, SectionKind};
use self::{
    errors::{UpdateError, UpdateResult},
    sections::{DataSection, RelSection, SectionHeader, StringTable, SymbolTable},
};

use super::ToBytes;

pub use instructions::Instr;

const FILE_VERSION: u8 = 3;
const MAGIC_NUMBER: u32 = 0x666f016b;

pub struct KOFile {
    header: KOHeader,
    sh_strtab: StringTable,
    section_headers: Vec<SectionHeader>,
    str_tabs: Vec<StringTable>,
    sym_tabs: Vec<SymbolTable>,
    data_sections: Vec<DataSection>,
    rel_sections: Vec<RelSection>,
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
            rel_sections: Vec::with_capacity(1),
        };

        kofile.add_header(SectionHeader::new(0, sections::SectionKind::Null));

        kofile.add_header(SectionHeader::new(1, sections::SectionKind::StrTab));

        kofile.sh_strtab.add(".shstrtab");

        kofile
    }

    pub fn add_shstr(&mut self, name: &str) -> usize {
        self.sh_strtab.add(name)
    }

    pub fn get_shstr(&self, index: usize) -> Option<&str> {
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

    pub fn get_header_name(&self, header: &SectionHeader) -> Option<&str> {
        self.sh_strtab.get(header.name_idx())
    }

    pub fn get_header_name_index(&self, index: usize) -> Option<&str> {
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

    pub fn add_rel_section(&mut self, rel_section: RelSection) {
        self.rel_sections.push(rel_section);
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

    pub fn rel_sections(&self) -> Iter<RelSection> {
        self.rel_sections.iter()
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

    pub fn new_relsection(&mut self, name: &str) -> RelSection {
        let sh_index = self.new_sh(name, SectionKind::Rel);
        RelSection::new(4, sh_index)
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

    fn iter_index_by_name(&self, name: &str, iter: Iter<&dyn SectionIndex>) -> Option<usize> {
        for (index, section) in iter.enumerate() {
            let section_name = match self.get_header_name_index(section.section_index()) {
                Some(s) => s,
                None => {
                    continue;
                }
            };

            if section_name == name {
                return Some(index);
            }
        }

        None
    }

    fn str_tab_index_by_name(&self, name: &str) -> Option<usize> {
        self.iter_index_by_name(
            name,
            self.str_tabs
                .iter()
                .map(|s| s as &dyn SectionIndex)
                .collect::<Vec<&dyn SectionIndex>>()
                .iter(),
        )
    }

    fn sym_tab_index_by_name(&self, name: &str) -> Option<usize> {
        self.iter_index_by_name(
            name,
            self.sym_tabs
                .iter()
                .map(|s| s as &dyn SectionIndex)
                .collect::<Vec<&dyn SectionIndex>>()
                .iter(),
        )
    }

    fn data_section_index_by_name(&self, name: &str) -> Option<usize> {
        self.iter_index_by_name(
            name,
            self.data_sections
                .iter()
                .map(|s| s as &dyn SectionIndex)
                .collect::<Vec<&dyn SectionIndex>>()
                .iter(),
        )
    }

    fn rel_section_index_by_name(&self, name: &str) -> Option<usize> {
        self.iter_index_by_name(
            name,
            self.rel_sections
                .iter()
                .map(|s| s as &dyn SectionIndex)
                .collect::<Vec<&dyn SectionIndex>>()
                .iter(),
        )
    }

    pub fn str_tab_by_name(&self, name: &str) -> Option<&StringTable> {
        self.str_tabs.iter().nth(self.str_tab_index_by_name(name)?)
    }

    pub fn str_tab_by_name_mut(&mut self, name: &str) -> Option<&mut StringTable> {
        let index = self.str_tab_index_by_name(name)?;
        self.str_tabs.iter_mut().nth(index)
    }

    pub fn sym_tab_by_name(&self, name: &str) -> Option<&SymbolTable> {
        self.sym_tabs.iter().nth(self.sym_tab_index_by_name(name)?)
    }

    pub fn sym_tab_by_name_mut(&mut self, name: &str) -> Option<&mut SymbolTable> {
        let index = self.sym_tab_index_by_name(name)?;
        self.sym_tabs.iter_mut().nth(index)
    }

    pub fn data_section_by_name(&self, name: &str) -> Option<&DataSection> {
        self.data_sections
            .iter()
            .nth(self.data_section_index_by_name(name)?)
    }

    pub fn data_section_by_name_mut(&mut self, name: &str) -> Option<&mut DataSection> {
        let index = self.data_section_index_by_name(name)?;
        self.data_sections.iter_mut().nth(index)
    }

    pub fn rel_section_by_name(&self, name: &str) -> Option<&RelSection> {
        self.rel_sections
            .iter()
            .nth(self.rel_section_index_by_name(name)?)
    }

    pub fn rel_section_by_name_mut(&mut self, name: &str) -> Option<&mut RelSection> {
        let index = self.rel_section_index_by_name(name)?;
        self.rel_sections.iter_mut().nth(index)
    }

    pub fn section_headers(&self) -> Iter<SectionHeader> {
        self.section_headers.iter()
    }

    pub fn section_count(&self) -> usize {
        self.section_headers.len()
    }

    pub fn version(&self) -> u8 {
        self.header.version
    }

    pub fn strtab_index(&self) -> usize {
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

        for i in 0..self.rel_sections.len() {
            let rel_section = self.rel_sections.get(i).unwrap();
            let section_index = rel_section.section_index();
            let size = rel_section.size();

            self.update_section_header("RelSection", section_index, size)?;
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

                for rel_section in self.rel_sections.iter() {
                    if rel_section.section_index() == i {
                        rel_section.to_bytes(&mut section_buffer);
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
        let mut str_tabs = Vec::new();
        let mut sym_tabs = Vec::new();
        let mut data_sections = Vec::new();
        let mut rel_sections = Vec::new();

        for _ in 0..header.num_headers {
            let header = SectionHeader::from_bytes(source, debug)?;
            section_headers.push(header);
        }

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
                sections::SectionKind::Rel => {
                    let rel_section = RelSection::from_bytes(source, debug, size, i)?;
                    rel_sections.push(rel_section);
                }
                sections::SectionKind::Debug => {
                    return Err(ReadError::DebugSectionUnsupportedError);
                }
                _ => unreachable!(),
            }
        }

        Ok(KOFile {
            header,
            sh_strtab,
            section_headers,
            str_tabs,
            sym_tabs,
            data_sections,
            rel_sections,
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
