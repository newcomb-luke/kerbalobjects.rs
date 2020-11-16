use std::{error::Error};

use crate::{KOFileWriter, KOFileReader};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionType {
    NULL,
    SYMTAB,
    STRTAB,
    REL,
    DATA,
    SUBRT
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolInfo {
    LOCAL,
    GLOBAL,
    EXTERN
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolType {
    NOTYPE,
    OBJECT,
    FUNC,
    SECTION
}

pub struct SymbolTable {
    entries: Vec<SymbolTableEntry>,
    index: u16,
    name: String,
}

impl SymbolTable {

    pub fn from(entries: Vec<SymbolTableEntry>, index: u16, name: &str) -> SymbolTable {

        SymbolTable {
            entries,
            index,
            name: String::from(name),
        }
    }

    pub fn new(name: &str) -> SymbolTable {
        SymbolTable {
            entries: Vec::new(),
            index: 0,
            name: String::from(name),
        }
    }

    pub fn size(&self) -> u32 {
        self.entries.len() as u32 * 14
    }

    pub fn set_index(&mut self, index: u16) {
        self.index = index;
    }

    pub fn get_index(&self) -> u16 {
        self.index
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = String::from(name);
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn add(&mut self, entry: SymbolTableEntry) {
        self.entries.push(entry);
    }

    pub fn get(&self, index: usize) -> Result<&SymbolTableEntry, Box<dyn Error>> {
        match self.entries.get(index) {
            Some(entry) => Ok(entry),
            None => Err(format!("Tried to get symbol from symbol table that does not exist at index: {}", index).into())
        }
    }

    pub fn write(&mut self, writer: &mut KOFileWriter) -> Result<(), Box<dyn Error>> {
        for entry in self.entries.iter() {
            entry.write(writer)?;
        }
        Ok(())
    }

    pub fn read(&mut self, reader: &mut KOFileReader, size: u32) -> Result<SymbolTable, Box<dyn Error>> {

        let mut entries = Vec::new();

        while self.size() < size {
            entries.push( SymbolTableEntry::read(reader)? );
        }

        Ok(SymbolTable::from(entries, 0, ""))

    }

}

pub struct SymbolTableEntry {
    name_index: u32,
    name: String,
    value_address: u32,
    size: u16,
    info: SymbolInfo,
    sym_type: SymbolType,
    data_section: u16
}

impl SymbolTableEntry {

    pub fn new(name: &str, value_address: u32, size: u16, info: SymbolInfo, sym_type: SymbolType, data_section: u16) -> SymbolTableEntry {
        SymbolTableEntry {
            name_index: 0,
            name: String::from(name),
            value_address,
            size,
            info,
            sym_type,
            data_section
        }
    }

    pub fn set_name(&mut self, name: String) {
        self.name = String::from(name);
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn write(&self, writer: &mut KOFileWriter) -> Result<(), Box<dyn Error>> {

        writer.write_uint32(self.name_index)?;

        writer.write_uint32(self.value_address)?;

        writer.write_uint16(self.size)?;

        writer.write(self.info.to_byte())?;

        writer.write(self.sym_type.to_byte())?;

        writer.write_uint16(self.data_section)?;

        Ok(())
    }

    pub fn read(reader: &mut KOFileReader) -> Result<SymbolTableEntry, Box<dyn Error>> {

        let name_index = reader.read_uint32()?;

        let value_address = reader.read_uint32()?;

        let size = reader.read_uint16()?;

        let info = SymbolInfo::from(reader.next()?)?;

        let sym_type = SymbolType::from(reader.next()?)?;

        let data_section = reader.read_uint16()?;

        Ok(SymbolTableEntry {
            name_index,
            name: String::new(),
            value_address,
            size,
            info,
            sym_type,
            data_section,
        })

    }

}

pub struct StringTable {
    strings: Vec<String>,
    current_size: u16,
    index: u16,
    name: String,
    raw_contents: Vec<u8>,
}

impl StringTable {

    /// Creates a fresh new symbol table
    pub fn new(name: &str) -> StringTable {
        // We always start the size at 1 because the first byte is always \0
        StringTable { strings: Vec::new(), current_size: 1, index: 0, name: String::from(name), raw_contents: vec![0] }
    }

    pub fn get_index(&self) -> u16 {
        self.index
    }

    pub fn set_index(&mut self, index: u16) {
        self.index = index;
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    /// Returns the string at a certain index of this strtab.
    pub fn get_string_at(&self, index: u16) -> Result<String, Box<dyn Error>> {
        let mut internal = String::new();

        for i in (index as usize)..self.raw_contents.len() {

            let c = match self.raw_contents.get(i) {
                Some(c) => *c as char,
                None => return Err( format!("Error getting string at index {} from string table", i).into() )
            };

            if c == '\0' {
                break;
            }

            internal.push(c);
        }

        Ok(internal)
    }

    /// Same as add(), but checks to see if the string is empty, if it is, the string isn't added and the index of 0 is returned instead
    pub fn add_checked(&mut self, string: &str) -> u16 {
        if !string.is_empty() {
            self.add(string)
        } else {
            0
        }
    }

    /// Adds the string to the strings vector, and returns the index into the string table that it will reside
    pub fn add(&mut self, string: &str) -> u16 {

        // Write the bytes to the internal contents
        for b in string.bytes() {
            self.raw_contents.push(b);
        }
        self.raw_contents.push(0);

        self.strings.push(String::from(string));
        self.current_size += 1 + string.len() as u16;

        self.current_size - (1 + string.len() as u16)

    }

    /// Writes the entire symbol table to the writer
    pub fn write(&mut self, writer: &mut KOFileWriter) -> Result<(), Box<dyn Error>> {

        writer.write_multiple(&self.raw_contents)?;

        Ok(())
    }

    pub fn read(reader: &mut KOFileReader, size: u32) -> Result<StringTable, Box<dyn Error>> {

        println!("\t\tReading string table. Size: {}", size);

        let mut strtab = StringTable::new("");

        // That pesky first null character.
        reader.next()?;

        while strtab.size() < size {

            let s = &reader.read_string()?;

            strtab.add( s );
        }

        Ok(strtab)
    }

    pub fn size(&self) -> u32 {
        self.current_size as u32
    }

}

pub struct RelocatableSection {

}

pub struct DataSection {

}

pub struct SubroutineSection {

}

impl SectionType {

    pub fn to_byte(&self) -> u8 {
        match self {
            SectionType::NULL => 0,
            SectionType::SYMTAB => 1,
            SectionType::STRTAB => 2,
            SectionType::REL => 3,
            SectionType::DATA => 4,
            SectionType::SUBRT => 5
        }
    }

    pub fn from(byte: u8) -> Result<SectionType, Box<dyn Error>>{
        match byte {
            0 => Ok(SectionType::NULL),
            1 => Ok(SectionType::SYMTAB),
            2 => Ok(SectionType::STRTAB),
            3 => Ok(SectionType::REL),
            4 => Ok(SectionType::DATA),
            5 => Ok(SectionType::SUBRT),
            b => Err(format!("Section type of {} is not a valid section type.", b).into())
        }
    }

}

impl SymbolInfo {

    pub fn to_byte(&self) -> u8 {
        match self {
            SymbolInfo::LOCAL => 0,
            SymbolInfo::GLOBAL => 1,
            SymbolInfo::EXTERN => 2,
        }
    }

    pub fn from(byte: u8) -> Result<SymbolInfo, Box<dyn Error>>{
        match byte {
            0 => Ok(SymbolInfo::LOCAL),
            1 => Ok(SymbolInfo::GLOBAL),
            2 => Ok(SymbolInfo::EXTERN),
            b => Err(format!("Section type of {} is not a valid section type.", b).into())
        }
    }

}

impl SymbolType {

    pub fn to_byte(&self) -> u8 {
        match self {
            SymbolType::NOTYPE => 0,
            SymbolType::OBJECT => 1,
            SymbolType::FUNC => 2,
            SymbolType::SECTION => 3,
        }
    }

    pub fn from(byte: u8) -> Result<SymbolType, Box<dyn Error>>{
        match byte {
            0 => Ok(SymbolType::NOTYPE),
            1 => Ok(SymbolType::OBJECT),
            2 => Ok(SymbolType::FUNC),
            3 => Ok(SymbolType::SECTION),
            b => Err(format!("Section type of {} is not a valid section type.", b).into())
        }
    }

}