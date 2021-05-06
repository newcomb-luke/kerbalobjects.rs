pub mod sections;

pub mod symbols;

pub mod instructions;

pub mod errors;

use std::slice::Iter;

use crate::FromBytes;

use crate::errors::{ReadError, ReadResult};

use self::{
    errors::{UpdateError, UpdateResult},
    sections::{DataSection, RelSection, SectionHeader, StringTable, SymbolTable},
};

use super::ToBytes;

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
    fn from_bytes(source: &mut Iter<u8>) -> ReadResult<Self>
    where
        Self: Sized,
    {
        let header = KOHeader::from_bytes(source)?;
        let mut section_headers = Vec::with_capacity(header.num_headers() as usize);
        let mut str_tabs = Vec::new();
        let mut sym_tabs = Vec::new();
        let mut data_sections = Vec::new();
        let mut rel_sections = Vec::new();

        for _ in 0..header.num_headers {
            let header = SectionHeader::from_bytes(source)?;
            section_headers.push(header);
        }

        let strtab_size = section_headers
            .get(1)
            .ok_or(ReadError::MissingSectionError(".shstrtab"))?
            .size() as usize;

        let sh_strtab = StringTable::from_bytes(source, strtab_size, 1)?;

        for i in 2..section_headers.len() {
            let header = section_headers.get(i).unwrap();
            let size = header.size() as usize;

            match header.kind() {
                sections::SectionKind::Null => {}
                sections::SectionKind::StrTab => {
                    let str_tab = StringTable::from_bytes(source, size, i)?;
                    str_tabs.push(str_tab);
                }
                sections::SectionKind::SymTab => {
                    let sym_tab = SymbolTable::from_bytes(source, size, i)?;
                    sym_tabs.push(sym_tab);
                }
                sections::SectionKind::Data => {
                    let data_section = DataSection::from_bytes(source, size, i)?;
                    data_sections.push(data_section);
                }
                sections::SectionKind::Rel => {
                    let rel_section = RelSection::from_bytes(source, size, i)?;
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
    fn from_bytes(source: &mut Iter<u8>) -> ReadResult<Self>
    where
        Self: Sized,
    {
        let magic =
            u32::from_bytes(source).map_err(|_| ReadError::KOHeaderReadError("file magic"))?;
        let version =
            u8::from_bytes(source).map_err(|_| ReadError::KOHeaderReadError("version"))?;
        let num_headers = u16::from_bytes(source)
            .map_err(|_| ReadError::KOHeaderReadError("number of headers"))?;
        let strtab_idx = u16::from_bytes(source)
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
