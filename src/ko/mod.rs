//! The module for describing, reading, and writing Kerbal Object files

use std::slice::Iter;

use crate::errors::{ReadError, ReadResult};
use crate::{FileIterator, FromBytes, ToBytes};

use self::sections::{DataSection, FuncSection, SectionHeader, StringTable, SymbolTable};
use self::sections::{ReldSection, SectionIndex, SectionKind};

pub mod errors;
pub mod instructions;
pub mod sections;
pub mod symbols;

use crate::ko::errors::{FinalizeError, FinalizeResult};
pub use instructions::Instr;

const FILE_VERSION: u8 = 4;
const MAGIC_NUMBER: u32 = 0x666f016b;

/// Implemented by Kerbal Object sections to read them from a file iterator, with the
/// size and section index known
pub trait SectionFromBytes {
    /// Reads a Kerbal Object file section from a file iterator using the pre-known
    /// size and section header index
    fn from_bytes(source: &mut FileIterator, size: usize, section_index: usize) -> ReadResult<Self>
    where
        Self: Sized;
}

macro_rules! gen_get_by_name {
    ($(#[$ref_attr:meta])* => $func_name: ident, $(#[$mut_attr:meta])* => $mut_func_name: ident, $section_name: ident, $section_type: ty) => {
        $(#[$ref_attr])*
        pub fn $func_name(&self, name: &str) -> Option<&$section_type> {
            self.$section_name.iter().find(|section| {
                match self.sh_name_from_index(section.section_index()) {
                    Some(s) => s == name,
                    None => false,
                }
            })
        }

        $(#[$mut_attr])*
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

macro_rules! gen_new_section {
    ($(#[$attr:meta])* => $func_name: ident, $section_type: ty, $section_kind: expr) => {
        $(#[$attr])*
        pub fn $func_name(&mut self, name: &str) -> $section_type {
            let sh_index = self.new_sh(name, $section_kind);
            <$section_type>::with_capacity(4, sh_index)
        }
    };
}

/// An in-memory representation of a KO file
///
/// Can be modified, written, or read.
#[derive(Debug)]
pub struct KOFile {
    /// The Kerbal Object file header
    pub header: KOHeader,
    sh_strtab: StringTable,
    section_headers: Vec<SectionHeader>,
    str_tabs: Vec<StringTable>,
    sym_tabs: Vec<SymbolTable>,
    data_sections: Vec<DataSection>,
    func_sections: Vec<FuncSection>,
    reld_sections: Vec<ReldSection>,
}

impl KOFile {
    /// Creates a new, empty, Kerbal Object file.
    ///
    /// Creates a new section header table, initialized with a first element that is a Null section,
    /// and a string table that is the section header string table.
    ///
    pub fn new() -> Self {
        let mut kofile = KOFile {
            header: KOHeader::new(2, 2),
            sh_strtab: StringTable::with_capacity(256, 1),
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

    /// Adds a new string to the section header string table
    pub fn add_shstr(&mut self, name: &str) -> usize {
        self.sh_strtab.add(name)
    }

    /// Gets a section name from the section header string table at the specified index, or
    /// None if none exists at that index
    pub fn get_shstr(&self, index: usize) -> Option<&String> {
        self.sh_strtab.get(index)
    }

    /// Adds a new section header to the section header table, and returns the index of the
    /// section header
    pub fn add_header(&mut self, header: SectionHeader) -> usize {
        let index = self.section_headers.len();
        self.section_headers.push(header);
        index
    }

    /// Gets the section header at the provided index into the section header table,
    /// or None if none exists at that index
    pub fn get_header(&self, index: usize) -> Option<&SectionHeader> {
        self.section_headers.get(index)
    }

    /// Gets the name of the section referred to by the provided section header,
    /// or None if no section with that header is found in this KO file
    pub fn get_header_name(&self, header: &SectionHeader) -> Option<&String> {
        self.sh_strtab.get(header.name_idx)
    }

    /// Returns the name of the section referred to by the index into the
    /// Kerbal Object file's section header table, or None if a section header
    /// at that index doesn't exist
    pub fn sh_name_from_index(&self, index: usize) -> Option<&String> {
        let header = self.section_headers.get(index)?;
        self.sh_strtab.get(header.name_idx)
    }

    /// Adds a new string table to this Kerbal Object file
    pub fn add_str_tab(&mut self, str_tab: StringTable) {
        self.str_tabs.push(str_tab);
    }

    /// Adds a new symbol table to this Kerbal Object file
    pub fn add_sym_tab(&mut self, sym_tab: SymbolTable) {
        self.sym_tabs.push(sym_tab);
    }

    /// Adds a new data section to this Kerbal Object file
    pub fn add_data_section(&mut self, data_section: DataSection) {
        self.data_sections.push(data_section);
    }

    /// Adds a new function section to this Kerbal Object file
    pub fn add_func_section(&mut self, func_section: FuncSection) {
        self.func_sections.push(func_section);
    }

    /// Adds a new relocation data section to this Kerbal Object file
    pub fn add_reld_section(&mut self, reld_section: ReldSection) {
        self.reld_sections.push(reld_section);
    }

    /// Returns an iterator over all of the string tables in this Kerbal Object file
    pub fn str_tabs(&self) -> Iter<StringTable> {
        self.str_tabs.iter()
    }

    /// Returns an iterator over all of the symbol tables in this Kerbal Object file
    pub fn sym_tabs(&self) -> Iter<SymbolTable> {
        self.sym_tabs.iter()
    }

    /// Returns an iterator over all of the data sections in this Kerbal Object file
    pub fn data_sections(&self) -> Iter<DataSection> {
        self.data_sections.iter()
    }

    /// Returns an iterator over all of the function sections in this Kerbal Object file
    pub fn func_sections(&self) -> Iter<FuncSection> {
        self.func_sections.iter()
    }

    /// Returns an iterator over all of the relocation data sections in this Kerbal Object file
    pub fn reld_sections(&self) -> Iter<ReldSection> {
        self.reld_sections.iter()
    }

    /// Adds a new section header of the provided name and section kind to this Kerbal
    /// Object file, and returns the index into the section header table of this new header
    pub fn new_sh(&mut self, name: &str, kind: SectionKind) -> usize {
        let name_idx = self.add_shstr(name);
        let header = SectionHeader::new(name_idx, kind);
        self.add_header(header)
    }

    /// Returns the index into the section header table of the section with the provided name,
    /// or None if there is no such section
    pub fn sh_index_by_name(&self, name: &str) -> Option<usize> {
        for (index, header) in self.section_headers.iter().enumerate() {
            let name_idx = header.name_idx;
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

    gen_new_section! {
        /// Creates a new StringTable, and adds a new entry for it in the section
        /// header table with the provided name.
        ///
        /// Returns the new StringTable
        =>
        new_strtab,
        StringTable,
        SectionKind::StrTab
    }

    gen_new_section! {
        /// Creates a new SymbolTable, and adds a new entry for it in the section
        /// header table with the provided name.
        ///
        /// Returns the new SymbolTable
        =>
        new_symtab,
        SymbolTable,
        SectionKind::SymTab
    }

    gen_new_section! {
        /// Creates a new DataSection, and adds a new entry for it in the section
        /// header table with the provided name.
        ///
        /// Returns the new DataSection
        =>
        new_data_section,
        DataSection,
        SectionKind::Data
    }

    gen_new_section! {
        /// Creates a new FuncSection, and adds a new entry for it in the section
        /// header table with the provided name.
        ///
        /// Returns the new FuncSection
        =>
        new_func_section,
        FuncSection,
        SectionKind::Func
    }

    gen_new_section! {
        /// Creates a new ReldSection, and adds a new entry for it in the section
        /// header table with the provided name.
        ///
        /// Returns the new ReldSection
        =>
        new_reld_section,
        ReldSection,
        SectionKind::Reld
    }

    gen_get_by_name! {
    /// Gets a reference to the StringTable with the provided name,
    /// or None if a StringTable by that name doesn't exist
    =>
    str_tab_by_name,
    /// Gets a mutable reference to the StringTable with the provided name,
    /// or None if a StringTable by that name doesn't exist
    =>
    str_tab_by_name_mut, str_tabs, StringTable}

    gen_get_by_name! {
        /// Gets a reference to the SymbolTable with the provided name,
        /// or None if a SymbolTable by that name doesn't exist
        =>
        sym_tab_by_name,
        /// Gets a mutable reference to the SymbolTable with the provided name,
        /// or None if a SymbolTable by that name doesn't exist
        =>
        sym_tab_by_name_mut,
        sym_tabs,
        SymbolTable
    }

    gen_get_by_name! {
        /// Gets a reference to the DataSection with the provided name,
        /// or None if a DataSection by that name doesn't exist
        =>
        data_section_by_name,
        /// Gets a mutable reference to the DataSection with the provided name,
        /// or None if a DataSection by that name doesn't exist
        =>
        data_section_by_name_mut,
        data_sections,
        DataSection
    }

    gen_get_by_name! {
        /// Gets a reference to the FuncSection with the provided name,
        /// or None if a FuncSection by that name doesn't exist
        =>
        func_section_by_name,
        /// Gets a mutable reference to the FuncSection with the provided name,
        /// or None if a FuncSection by that name doesn't exist
        =>
        func_section_by_name_mut,
        func_sections,
        FuncSection
    }

    gen_get_by_name!(
        /// Gets a reference to the ReldSection with the provided name,
        /// or None if a ReldSection by that name doesn't exist
        =>
        reld_section_by_name,
        /// Gets a mutable reference to the ReldSection with the provided name,
        /// or None if a ReldSection by that name doesn't exist
        =>
        reld_section_by_name_mut,
        reld_sections,
        ReldSection
    );

    /// Returns an iterator over all section headers in the Kerbal Object file's section
    /// header table
    pub fn section_headers(&self) -> Iter<SectionHeader> {
        self.section_headers.iter()
    }

    /// Returns the number of sections registered in the section header table. This will always
    /// be at least 1, because of the Null section present in all valid section header tables.
    pub fn section_count(&self) -> usize {
        self.section_headers.len()
    }

    /// The index into the section header table of the section header string table
    pub fn sh_strtab_index(&self) -> usize {
        self.sh_strtab.section_index()
    }

    fn update_section_header(
        &mut self,
        section_name: &'static str,
        section_index: usize,
        section_size: u32,
    ) -> Result<(), FinalizeError> {
        match self.section_headers.get_mut(section_index) {
            Some(header) => {
                header.size = section_size;
            }
            None => {
                return Err(FinalizeError::InvalidSectionIndexError(
                    section_name,
                    section_index,
                ));
            }
        }

        Ok(())
    }

    fn update_section_headers(&mut self) -> Result<(), FinalizeError> {
        self.section_headers.get_mut(1).unwrap().size = self.sh_strtab.size();

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

    /// Verifies that a Kerbal Object file's section header data is sound so that it
    /// can be written out as a proper KerbalObject file
    pub fn finalize(mut self) -> FinalizeResult {
        self.header = KOHeader::new(
            self.section_headers.len() as u16,
            self.sh_strtab.section_index() as u16,
        );

        self.update_section_headers()?;

        Ok(WritableKOFile(self))
    }
}

impl Default for KOFile {
    fn default() -> Self {
        Self::new()
    }
}

/// A finalized Kerbal Object file that can be written to a byte buffer, or converted
/// into a KOFile
#[derive(Debug)]
pub struct WritableKOFile(KOFile);

impl From<WritableKOFile> for KOFile {
    fn from(w_kofile: WritableKOFile) -> Self {
        w_kofile.0
    }
}

impl ToBytes for WritableKOFile {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        let ko = &self.0;

        ko.header.to_bytes(buf);

        // Arbitrary "large number"
        let mut section_buffer: Vec<u8> = Vec::with_capacity(2048);

        for i in 0..ko.section_headers.len() {
            ko.section_headers.get(i).unwrap().to_bytes(buf);

            if i == 1 {
                ko.sh_strtab.to_bytes(&mut section_buffer);
            } else if i != 0 {
                for str_tab in ko.str_tabs.iter() {
                    if str_tab.section_index() == i {
                        str_tab.to_bytes(&mut section_buffer);
                        continue;
                    }
                }

                for sym_tab in ko.sym_tabs.iter() {
                    if sym_tab.section_index() == i {
                        sym_tab.to_bytes(&mut section_buffer);
                        continue;
                    }
                }

                for data_section in ko.data_sections.iter() {
                    if data_section.section_index() == i {
                        data_section.to_bytes(&mut section_buffer);
                        continue;
                    }
                }

                for func_section in ko.func_sections.iter() {
                    if func_section.section_index() == i {
                        func_section.to_bytes(&mut section_buffer);
                        continue;
                    }
                }

                for reld_section in ko.reld_sections.iter() {
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
    fn from_bytes(source: &mut FileIterator) -> ReadResult<Self>
    where
        Self: Sized,
    {
        let header = KOHeader::from_bytes(source)?;
        let mut section_headers = Vec::with_capacity(header.num_headers as usize);
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
            let header = SectionHeader::from_bytes(source)?;

            match header.section_kind {
                SectionKind::StrTab => {
                    num_str_tabs += 1;
                }
                SectionKind::SymTab => {
                    num_sym_tabs += 1;
                }
                SectionKind::Data => {
                    num_data_sections += 1;
                }
                SectionKind::Func => {
                    num_func_sections += 1;
                }
                SectionKind::Reld => {
                    num_reld_sections += 1;
                }
                SectionKind::Null | SectionKind::Debug => {}
            }

            #[cfg(feature = "print_debug")]
            {
                println!(
                    "Section header: Name index: {}, kind: {:?}, size: {}",
                    header.name_idx, header.section_kind, header.size
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
            .size as usize;

        let sh_strtab = StringTable::from_bytes(source, strtab_size, 1)?;

        for i in 2..section_headers.len() {
            let header = section_headers.get(i).unwrap();
            let size = header.size as usize;

            match header.section_kind {
                SectionKind::Null => {}
                SectionKind::StrTab => {
                    let str_tab = StringTable::from_bytes(source, size, i)?;
                    str_tabs.push(str_tab);
                }
                SectionKind::SymTab => {
                    let sym_tab = SymbolTable::from_bytes(source, size, i)?;
                    sym_tabs.push(sym_tab);
                }
                SectionKind::Data => {
                    let data_section = DataSection::from_bytes(source, size, i)?;
                    data_sections.push(data_section);
                }
                SectionKind::Func => {
                    let func_section = FuncSection::from_bytes(source, size, i)?;
                    func_sections.push(func_section);
                }
                SectionKind::Reld => {
                    let reld_section = ReldSection::from_bytes(source, size, i)?;
                    reld_sections.push(reld_section);
                }
                SectionKind::Debug => {
                    return Err(ReadError::DebugSectionUnsupportedError);
                }
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

/// The header of a Kerbal Object file
#[derive(Debug)]
pub struct KOHeader {
    magic: u32,
    /// The version number of this Kerbal Object file
    pub version: u8,
    /// The number of headers contained within the section header table
    pub num_headers: u16,
    /// The index of the section header string table
    pub sh_strtab_idx: u16,
}

impl KOHeader {
    /// Creates a new Kerbal Object file header
    pub fn new(num_headers: u16, sh_strtab_idx: u16) -> Self {
        Self {
            magic: MAGIC_NUMBER,
            version: FILE_VERSION,
            num_headers,
            sh_strtab_idx,
        }
    }

    /// Returns the size in bytes of a Kerbal Object file header
    pub fn size_bytes() -> usize {
        9
    }
}

impl ToBytes for KOHeader {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        self.magic.to_bytes(buf);
        self.version.to_bytes(buf);
        self.num_headers.to_bytes(buf);
        self.sh_strtab_idx.to_bytes(buf);
    }
}

impl FromBytes for KOHeader {
    fn from_bytes(source: &mut FileIterator) -> ReadResult<Self>
    where
        Self: Sized,
    {
        let magic =
            u32::from_bytes(source).map_err(|_| ReadError::KOHeaderReadError("file magic"))?;
        let version =
            u8::from_bytes(source).map_err(|_| ReadError::KOHeaderReadError("version"))?;
        let num_headers = u16::from_bytes(source)
            .map_err(|_| ReadError::KOHeaderReadError("number of headers"))?;
        let sh_strtab_idx = u16::from_bytes(source)
            .map_err(|_| ReadError::KOHeaderReadError("section header string table index"))?;

        if magic != MAGIC_NUMBER {
            return Err(ReadError::InvalidKOFileMagicError);
        }

        if version != FILE_VERSION {
            return Err(ReadError::VersionMismatchError(version));
        }

        Ok(Self {
            magic,
            version,
            num_headers,
            sh_strtab_idx,
        })
    }
}
