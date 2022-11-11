//! # Kerbal Object files
//!
//! The module for describing, reading, and writing Kerbal Object files
//!
//! There **must** be at least the null section, and the .shstrtab, which are automatically
//! added for you using KOFile::new().
//!
//! Each section added **must** have an entry in the section header table, or you will get an error
//! attempting to validate the file. Also all section header table entries must correspond to a section.
//! The series of `new_<section name>` functions create a section header automatically for you, then you
//! later add the section to the KOFile using `.add_<section name>`.
//!
//! ```
//! use kerbalobjects::ko::symbols::{KOSymbol, SymBind, SymType};
//! use kerbalobjects::ko::{Instr, KOFile};
//! use kerbalobjects::{KOSValue, Opcode};
//! use kerbalobjects::ko::SectionIdx;
//! use kerbalobjects::ko::sections::DataIdx;
//! use std::io::Write;
//! use std::path::PathBuf;
//!
//! let mut ko = KOFile::new();
//!
//! let mut data_section = ko.new_data_section(".data");
//! let mut start = ko.new_func_section("_start");
//! let mut symtab = ko.new_symtab(".symtab");
//! let mut symstrtab = ko.new_strtab(".symstrtab");
//!
//! // Set up the main code function section
//! let one = data_section.add_checked(KOSValue::Int16(1));
//!
//! start.add(Instr::TwoOp(
//!     Opcode::Bscp,
//!     one,
//!     data_section.add_checked(KOSValue::Int16(0)),
//! ));
//! start.add(Instr::ZeroOp(Opcode::Argb));
//! start.add(Instr::OneOp(
//!     Opcode::Push,
//!     data_section.add_checked(KOSValue::ArgMarker),
//! ));
//! start.add(Instr::OneOp(
//!     Opcode::Push,
//!     data_section.add_checked(KOSValue::StringValue("Hello, world!".into())),
//! ));
//! start.add(Instr::TwoOp(
//!     Opcode::Call,
//!     data_section.add_checked(KOSValue::String("".into())),
//!     data_section.add_checked(KOSValue::String("print()".into())),
//! ));
//! start.add(Instr::ZeroOp(Opcode::Pop));
//! start.add(Instr::OneOp(Opcode::Escp, one));
//!
//! // Set up our symbols
//! let file_symbol = KOSymbol::new(
//!     symstrtab.add("test.kasm"),
//!     DataIdx::PLACEHOLDER,
//!     0,
//!     SymBind::Global,
//!     SymType::File,
//!     SectionIdx::NULL,
//! );
//! let start_symbol = KOSymbol::new(
//!     symstrtab.add("_start"),
//!     DataIdx::PLACEHOLDER,
//!     start.size() as u16,
//!     SymBind::Global,
//!     SymType::Func,
//!     start.section_index(),
//! );
//!
//! symtab.add(file_symbol);
//! symtab.add(start_symbol);
//!
//! ko.add_data_section(data_section);
//! ko.add_func_section(start);
//! ko.add_str_tab(symstrtab);
//! ko.add_sym_tab(symtab);
//!
//! // Write the file out to disk
//! let mut file_buffer = Vec::with_capacity(2048);
//!
//! let ko = ko.validate().expect("Could not update KO headers properly");
//! ko.write(&mut file_buffer);
//!
//! let file_path = PathBuf::from("test.ko");
//! let mut file =
//!     std::fs::File::create(file_path).expect("Output file could not be created: test.ko");
//!
//! file.write_all(file_buffer.as_slice())
//!     .expect("File test.ko could not be written to.");
//! ```
//!

use std::slice::Iter;

use crate::{BufferIterator, FromBytes, ToBytes, WritableBuffer};

use self::sections::{DataSection, FuncSection, SectionHeader, StringTable, SymbolTable};
use self::sections::{ReldSection, SectionKind};

pub mod errors;
pub mod instructions;
pub mod sections;
pub mod symbols;

use crate::ko::errors::{HeaderParseError, KOParseError, ValidationError};
use crate::ko::sections::StringIdx;
pub use instructions::Instr;

const FILE_VERSION: u8 = 4;
/// `k` 1 `o` `f`
const MAGIC_NUMBER: u32 = 0x666f016b;

/// A wrapper type that represents an index into a section header table of a KO file.
///
/// This type implements From<u16> and u16 implements From<SectionIdx>, but this is provided
/// so that it takes 1 extra step to convert raw integers into SectionIdx which could stop potential
/// logical bugs.
///
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SectionIdx(u16);

impl From<u8> for SectionIdx {
    fn from(i: u8) -> Self {
        Self(i as u16)
    }
}

impl From<u16> for SectionIdx {
    fn from(i: u16) -> Self {
        Self(i)
    }
}

impl From<SectionIdx> for u16 {
    fn from(idx: SectionIdx) -> Self {
        idx.0
    }
}

impl From<SectionIdx> for usize {
    fn from(idx: SectionIdx) -> Self {
        idx.0 as usize
    }
}

impl SectionIdx {
    /// A constant for the null section header index, which is at the section header index of 0
    pub const NULL: SectionIdx = SectionIdx(0u16);
}

/// An in-memory representation of a KO file
///
/// Can be modified, written, or read.
#[derive(Debug)]
pub struct KOFile {
    header: KOHeader,
    shstrtab: StringTable,
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
        // When we create this file, the section header string table will just always be the first section
        let shstrtab_idx = SectionIdx::from(1u16);

        let mut kofile = Self {
            header: KOHeader::new(2, shstrtab_idx),
            shstrtab: StringTable::with_capacity(256, shstrtab_idx),
            section_headers: Vec::with_capacity(4),
            str_tabs: Vec::with_capacity(2),
            sym_tabs: Vec::with_capacity(1),
            data_sections: Vec::with_capacity(1),
            func_sections: Vec::with_capacity(1),
            reld_sections: Vec::with_capacity(1),
        };

        // Add the null entry
        kofile.add_section_header(SectionHeader::null());

        let name_idx = kofile.shstrtab.add(".shstrtab");

        // The first section will always be the section header string table
        kofile.add_section_header(SectionHeader::new(name_idx, SectionKind::StrTab));

        kofile
    }

    /// Adds a new string to the section header string table
    pub fn add_section_name(&mut self, name: impl Into<String>) -> StringIdx {
        self.shstrtab.add(name)
    }

    /// Gets a section name from the section header string table at the specified index, or
    /// returns None if none exists at that index
    pub fn get_section_name(&self, index: StringIdx) -> Option<&String> {
        self.shstrtab.get(index)
    }

    /// Returns the name of the section referred to by the index into the
    /// Kerbal Object file's section header table, or None if a section header
    /// at that index doesn't exist
    pub fn get_section_name_by_index(&self, index: SectionIdx) -> Option<&String> {
        let header = self.section_headers.get(usize::from(index))?;
        self.shstrtab.get(header.name_idx)
    }

    /// Adds a new section header to the section header table, and returns the index of the
    /// section header
    ///
    /// This will panic if you insert more than u16::MAX section headers. Don't do that.
    ///
    pub fn add_section_header(&mut self, header: SectionHeader) -> SectionIdx {
        let index = self.section_headers.len();

        if index > u16::MAX as usize {
            panic!(
                "Attempted to insert {} section headers. Section indices have a maximum of {}",
                u16::MAX as usize + 1,
                u16::MAX
            );
        }

        self.section_headers.push(header);
        SectionIdx::from(index as u16)
    }

    /// Gets the section header at the provided index into the section header table,
    /// or None if none exists at that index
    pub fn get_section_header(&self, index: SectionIdx) -> Option<&SectionHeader> {
        self.section_headers.get(usize::from(index))
    }

    /// Gets the name of the section referred to by the provided section header,
    /// or None if no section with that header is found in this KO file
    pub fn get_header_name(&self, header: &SectionHeader) -> Option<&String> {
        self.shstrtab.get(header.name_idx)
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
    pub fn new_section_header(&mut self, name: impl Into<String>, kind: SectionKind) -> SectionIdx {
        let name_idx = self.add_section_name(name);
        let header = SectionHeader::new(name_idx, kind);
        self.add_section_header(header)
    }

    /// Returns the index into the section header table of the section with the provided name,
    /// or None if there is no such section
    pub fn get_section_index_by_name(&self, name: impl AsRef<str>) -> Option<SectionIdx> {
        for (index, header) in self.section_headers.iter().enumerate() {
            let name_idx = header.name_idx;
            let sh_name = match self.shstrtab.get(name_idx) {
                Some(name) => name,
                None => {
                    continue;
                }
            };

            if name.as_ref() == sh_name {
                return Some(SectionIdx::from(index as u16));
            }
        }

        None
    }

    /// The Kerbal Object file header
    pub fn header(&self) -> KOHeader {
        self.header
    }

    /// Returns an iterator over all section headers in the Kerbal Object file's section
    /// header table
    pub fn section_headers(&self) -> Iter<SectionHeader> {
        self.section_headers.iter()
    }

    /// Returns the number of sections registered in the section header table. This will always
    /// be at least 1, because of the Null section present in all valid section header tables.
    pub fn section_header_count(&self) -> usize {
        self.section_headers.len()
    }

    /// The index into the section header table of the section header string table
    pub fn shstrtab_index(&self) -> SectionIdx {
        self.shstrtab.section_index()
    }

    // Updates a single section header with the section's actual reported size
    // This can fail if the section header kind doesn't match with the section's kind,
    // or if the section's header index doesn't exist
    fn update_section_header(
        &mut self,
        section_kind: SectionKind,
        section_index: SectionIdx,
        section_size: u32,
    ) -> Result<(), ValidationError> {
        match self
            .section_headers
            .get_mut(u16::from(section_index) as usize)
        {
            Some(header) => {
                if header.section_kind != section_kind {
                    Err(ValidationError::SectionKindMismatchError(
                        section_kind,
                        u16::from(section_index),
                        header.section_kind,
                    ))
                } else {
                    header.size = section_size;
                    Ok(())
                }
            }
            None => Err(ValidationError::InvalidSectionIndexError(
                section_kind,
                u16::from(section_index),
            )),
        }
    }

    // Updates a all section headers with the section's actual reported size
    // This can fail if a section header kind doesn't match with a section's kind,
    // or if a section's header index doesn't exist
    fn update_section_headers(&mut self) -> Result<(), ValidationError> {
        // We'll keep track of which section header entries were actually updated. All of them
        // should be updated except for the first null section.
        let mut header_set = vec![self.shstrtab.section_index()];

        // Update the .shstrtab
        self.section_headers
            .get_mut(usize::from(self.shstrtab.section_index()))
            .unwrap()
            .size = self.shstrtab.size();

        for i in 0..self.str_tabs.len() {
            let section = self.str_tabs.get(i).unwrap();
            let idx = section.section_index();
            let size = section.size();
            header_set.push(idx);

            self.update_section_header(SectionKind::StrTab, idx, size)?;
        }

        for i in 0..self.sym_tabs.len() {
            let section = self.sym_tabs.get(i).unwrap();
            let idx = section.section_index();
            let size = section.size();
            header_set.push(idx);

            self.update_section_header(SectionKind::SymTab, idx, size)?;
        }

        for i in 0..self.data_sections.len() {
            let section = self.data_sections.get(i).unwrap();
            let idx = section.section_index();
            let size = section.size();

            header_set.push(idx);

            self.update_section_header(SectionKind::Data, idx, size)?;
        }

        for i in 0..self.func_sections.len() {
            let section = self.func_sections.get(i).unwrap();
            let idx = section.section_index();
            let size = section.size();
            header_set.push(idx);

            self.update_section_header(SectionKind::Func, idx, size)?;
        }

        for i in 0..self.reld_sections.len() {
            let section = self.reld_sections.get(i).unwrap();
            let idx = section.section_index();
            let size = section.size();
            header_set.push(idx);

            self.update_section_header(SectionKind::Reld, idx, size)?;
        }

        // Skip the first null section
        for (i, header) in self
            .section_headers
            .iter()
            .enumerate()
            .skip(1)
            .map(|(i, h)| (SectionIdx::from(i as u16), h))
        {
            let name = self
                .shstrtab
                .get(header.name_idx)
                .ok_or_else(|| {
                    ValidationError::InvalidSectionHeaderNameIndexError(
                        u16::from(i),
                        header.section_kind,
                        usize::from(header.name_idx),
                    )
                })?
                .clone();

            if !header_set.contains(&i) {
                return Err(ValidationError::InvalidSectionHeaderError(
                    u16::from(i),
                    header.section_kind,
                    name,
                ));
            }
        }

        Ok(())
    }

    /// Consumes and verifies that a Kerbal Object file's section header data is sound so that it
    /// can be written out as a proper KerbalObject file
    ///
    /// This can fail if a section header kind doesn't match with a section's kind,
    /// or if a section's header index doesn't exist
    ///
    /// A WritableKOFile can be turned back into a KOFile
    ///
    pub fn validate(mut self) -> Result<WritableKOFile, (Self, ValidationError)> {
        self.header = KOHeader::new(
            self.section_headers.len() as u16,
            self.shstrtab.section_index(),
        );

        if let Err(e) = self.update_section_headers() {
            Err((self, e))
        } else {
            Ok(WritableKOFile(self))
        }
    }

    /// Parses an entire KOFile from a byte buffer
    pub fn parse(source: &mut BufferIterator) -> Result<Self, KOParseError> {
        let header = KOHeader::parse(source).map_err(KOParseError::HeaderError)?;
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

        let null_header =
            SectionHeader::parse(source).map_err(KOParseError::SectionHeaderParseError)?;

        // The first section header table entry must be a null section
        if null_header.section_kind != SectionKind::Null {
            return Err(KOParseError::MissingNullSectionHeader(
                null_header.section_kind,
            ));
        }
        section_headers.push(null_header);

        for i in 1..header.num_headers {
            let header =
                SectionHeader::parse(source).map_err(KOParseError::SectionHeaderParseError)?;

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
                SectionKind::Debug => {
                    return Err(KOParseError::DebugSectionUnsupportedError(i));
                }
                SectionKind::Null => {
                    return Err(KOParseError::StrayNullSectionHeader(i));
                }
            }

            section_headers.push(header);
        }

        // The null section is here, and we need a .shstrtab, so this is invalid
        if header.shstrtab_idx == SectionIdx::from(0u16) {
            return Err(KOParseError::NullShStrTabIndexError);
        }

        // Allocate now
        str_tabs = Vec::with_capacity(num_str_tabs);
        sym_tabs = Vec::with_capacity(num_sym_tabs);
        data_sections = Vec::with_capacity(num_data_sections);
        func_sections = Vec::with_capacity(num_func_sections);
        reld_sections = Vec::with_capacity(num_reld_sections);

        let shstrtab_size = section_headers
            .get(usize::from(header.shstrtab_idx))
            .ok_or_else(|| {
                KOParseError::InvalidShStrTabIndexError(
                    header.shstrtab_idx.into(),
                    section_headers.len() as u16,
                )
            })?
            .size;

        let shstrtab = StringTable::parse(source, shstrtab_size, header.shstrtab_idx)
            .map_err(KOParseError::StringTableParseError)?;

        // We skip the first one here since, there is no 0th section
        for section_idx in (1..section_headers.len() as u16).map(SectionIdx::from) {
            // We've already parsed the .shstrtab
            if section_idx == header.shstrtab_idx {
                continue;
            }

            let header = section_headers.get(usize::from(section_idx)).unwrap();

            match header.section_kind {
                SectionKind::StrTab => {
                    str_tabs.push(
                        StringTable::parse(source, header.size, section_idx)
                            .map_err(KOParseError::StringTableParseError)?,
                    );
                }
                SectionKind::SymTab => {
                    sym_tabs.push(
                        SymbolTable::parse(source, header.size, section_idx)
                            .map_err(KOParseError::SymbolTableParseError)?,
                    );
                }
                SectionKind::Data => {
                    data_sections.push(
                        DataSection::parse(source, header.size, section_idx)
                            .map_err(KOParseError::DataSectionParseError)?,
                    );
                }
                SectionKind::Func => {
                    func_sections.push(
                        FuncSection::parse(source, header.size, section_idx)
                            .map_err(KOParseError::FunctionSectionParseError)?,
                    );
                }
                SectionKind::Reld => {
                    reld_sections.push(
                        ReldSection::parse(source, header.size, section_idx)
                            .map_err(KOParseError::ReldSectionParseError)?,
                    );
                }
                SectionKind::Debug => unimplemented!(),
                SectionKind::Null => {
                    panic!("Internal library error. Attempted to parse \"null\" section, this should be unreachable.");
                }
            }
        }

        Ok(Self {
            header,
            shstrtab,
            section_headers,
            str_tabs,
            sym_tabs,
            data_sections,
            func_sections,
            reld_sections,
        })
    }

    // There are extra macro-generated functions at the end of this file that are implemented
    // for KOFile.
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

impl WritableKOFile {
    /// Writes the binary representation of this KO file to the provided buffer
    pub fn write(&self, buf: &mut impl WritableBuffer) {
        let ko = &self.0;
        // Write the file header
        ko.header.write(buf);

        // First write all of the section headers in the section header table
        for header in &ko.section_headers {
            header.write(buf);
        }

        // Write out all of the sections in order
        for section_index in (0..ko.section_header_count()).map(|i| SectionIdx::from(i as u16)) {
            if self.0.shstrtab.section_index() == section_index {
                self.0.shstrtab.write(buf);
                continue;
            }

            if let Some(section) = ko
                .str_tabs
                .iter()
                .find(|s| s.section_index() == section_index)
            {
                section.write(buf);
                continue;
            }

            if let Some(section) = ko
                .sym_tabs
                .iter()
                .find(|s| s.section_index() == section_index)
            {
                section.write(buf);
                continue;
            }

            if let Some(section) = ko
                .data_sections
                .iter()
                .find(|s| s.section_index() == section_index)
            {
                section.write(buf);
                continue;
            }

            if let Some(section) = ko
                .func_sections
                .iter()
                .find(|s| s.section_index() == section_index)
            {
                section.write(buf);
                continue;
            }

            if let Some(section) = ko
                .reld_sections
                .iter()
                .find(|s| s.section_index() == section_index)
            {
                section.write(buf);
                continue;
            }
        }
    }

    /// Gets the KOFile back out of this
    pub fn get(self) -> KOFile {
        self.0
    }
}

impl From<WritableKOFile> for KOFile {
    fn from(w_kofile: WritableKOFile) -> Self {
        w_kofile.0
    }
}

/// The header of a Kerbal Object file
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct KOHeader {
    /// The "magic numbers" that identify a KO file
    pub magic: u32,
    /// The specification version number of this Kerbal Object file
    pub version: u8,
    /// The number of headers contained within the section header table
    pub num_headers: u16,
    /// The index of the section header string table
    pub shstrtab_idx: SectionIdx,
}

impl KOHeader {
    /// The size in bytes of a KO file header
    const HEADER_SIZE: usize = 9;

    /// Creates a new Kerbal Object file header
    pub const fn new(num_headers: u16, shstrtab_idx: SectionIdx) -> Self {
        Self {
            magic: MAGIC_NUMBER,
            version: FILE_VERSION,
            num_headers,
            shstrtab_idx,
        }
    }

    /// Returns the size in bytes of a Kerbal Object file header
    pub const fn size() -> usize {
        Self::HEADER_SIZE
    }

    /// Converts the header to its binary representation and writes it to the provided buffer
    fn write(&self, buf: &mut impl WritableBuffer) {
        self.magic.to_bytes(buf);
        self.version.to_bytes(buf);
        self.num_headers.to_bytes(buf);
        u16::from(self.shstrtab_idx).to_bytes(buf);
    }

    /// Parses a KOHeader from the provided byte buffer
    pub fn parse(source: &mut BufferIterator) -> Result<Self, HeaderParseError> {
        let magic = u32::from_bytes(source).map_err(|_| HeaderParseError::MissingMagicError)?;
        let version = u8::from_bytes(source).map_err(|_| HeaderParseError::MissingVersionError)?;
        let num_headers =
            u16::from_bytes(source).map_err(|_| HeaderParseError::MissingNumHeaders)?;
        let shstrtab_idx = SectionIdx(
            u16::from_bytes(source).map_err(|_| HeaderParseError::MissingShStrTabIndex)?,
        );

        if magic != MAGIC_NUMBER {
            return Err(HeaderParseError::InvalidMagicError(magic, MAGIC_NUMBER));
        }

        if version != FILE_VERSION {
            return Err(HeaderParseError::UnsupportedVersionError(
                version,
                FILE_VERSION,
            ));
        }

        Ok(Self {
            magic,
            version,
            num_headers,
            shstrtab_idx,
        })
    }
}

// All of this down here to not clutter up the more important code

macro_rules! gen_get_by_name {
    ($(#[$ref_attr:meta])* => $func_name: ident, $(#[$mut_attr:meta])* => $mut_func_name: ident, $section_name: ident, $section_type: ty) => {
        $(#[$ref_attr])*
        pub fn $func_name(&self, name: impl AsRef<str>) -> Option<&$section_type> {
            self.$section_name.iter().find(|section| {
                match self.get_section_name_by_index(section.section_index()) {
                    Some(s) => s == name.as_ref(),
                    None => false,
                }
            })
        }

        $(#[$mut_attr])*
        pub fn $mut_func_name(&mut self, name: impl AsRef<str>) -> Option<&mut $section_type> {
            let mut index_opt = Some(0);

            for (index, section) in self.$section_name.iter().enumerate() {
                match self.get_section_name_by_index(section.section_index()) {
                    Some(s) => {
                        if s == name.as_ref() {
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
        pub fn $func_name(&mut self, name: impl Into<String>) -> $section_type {
            let sh_index = self.new_section_header(name, $section_kind);
            <$section_type>::with_capacity(4, sh_index)
        }
    };
}

impl KOFile {
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
}
