use std::{error::Error};

use crate::{KOFileReader, KOFileWriter};

pub struct HeaderTable {
    headers: Vec<SectionHeader>
}

impl HeaderTable {

    pub fn new(headers: Vec<SectionHeader>) -> HeaderTable {
        HeaderTable { headers }
    }

    pub fn get_headers(&self) -> &Vec<SectionHeader> {
        &self.headers
    }

    pub fn get_header(&self, index: usize) -> Result<&SectionHeader, Box<dyn Error>> {
        match self.headers.get(index) {
            Some(h) => Ok(h),
            None => Err(format!("Tried to index section {} in the section header, which does not exist", index).into()),
        }
    }

    pub fn add(&mut self, header: SectionHeader) {
        self.headers.push(header);
    }

    pub fn read(reader: &mut KOFileReader, number_sections: u16) -> Result<HeaderTable, Box<dyn Error>> {

        let mut headers: Vec<SectionHeader> = Vec::with_capacity(number_sections as usize);

        for _ in 0..number_sections {
            headers.push(SectionHeader::read(reader)?);
        }
        
        Ok(HeaderTable::new(headers))
    }

    pub fn write(&self, writer: &mut KOFileWriter) -> Result<(), Box<dyn Error>> {

        for header in self.headers.iter() {
            header.write(writer)?;
        }

        Ok(())
    }

    pub fn validate_conventions(&self) -> Result<(), Box<dyn Error>> {

        if self.headers.len() < 4 {
            return Err("At least the null section, symbol string table, symbol table, and data section are required. One or more are missing.".into());
        }

        if self.get_header(0)?.get_type() != SectionType::NULL {
            return Err("The first entry into the section header table should be the null section.".into());
        }

        if self.get_header(1)?.get_type() != SectionType::STRTAB
            || self.get_header(1)?.name() != ".symstrtab" {
            return Err("Expected the symbol string table at index 1".into());
        }

        if self.get_header(2)?.get_type() != SectionType::DATA
            || self.get_header(2)?.name() != ".data" {
            return Err("Expected the symbol data section at index 2".into());
        }

        if self.get_header(3)?.get_type() != SectionType::SYMTAB
            || self.get_header(3)?.name() != ".symtab" {
            return Err("Expected the symbol table at index 3".into());
        }

        Ok(())
    }

    pub fn debug_print(&self) {
        println!("Section header table:");

        for header in self.headers.iter() {
            header.debug_print();
        }
    }

}

pub struct SectionHeader {
    section_type: SectionType,
    section_offset: u32,
    section_size: u32,
    section_name: String,
}

impl SectionHeader {

    pub fn new(section_type: SectionType, section_offset: u32, section_size: u32, section_name: &str) -> SectionHeader {
        SectionHeader {
            section_type,
            section_offset,
            section_size,
            section_name: section_name.to_owned(),
        }
    }

    pub fn read(reader: &mut KOFileReader) -> Result<SectionHeader, Box<dyn Error>> {

        let section_type = SectionType::from(reader.next()?)?;

        let section_offset = reader.read_uint32()?;

        let section_size = reader.read_uint32()?;

        let section_name = reader.read_string()?;

        Ok(SectionHeader::new(section_type, section_offset, section_size, &section_name))

    }

    pub fn write(&self, writer: &mut KOFileWriter) -> Result<(), Box<dyn Error>> {

        writer.write(self.section_type.to_byte())?;

        writer.write_uint32(self.section_offset)?;

        writer.write_uint32(self.section_size)?;

        writer.write_string(&self.section_name)?;

        Ok(())
    }

    pub fn get_type(&self) -> SectionType {
        self.section_type
    }

    pub fn offset(&self) -> u32 {
        self.section_offset
    }

    pub fn set_offset(&mut self, offset: u32) {
        self.section_offset = offset;
    }

    pub fn size(&self) -> u32 {
        9 + self.section_name.len() as u32 + 1
    }

    pub fn section_size(&self) -> u32 {
        self.section_size
    }

    pub fn name(&self) -> &String {
        &self.section_name
    }

    pub fn debug_print(&self) {
        println!("\tType: {:?}", self.get_type());
        println!("\tOffset: {}", self.offset());
        println!("\tHeader size: {}", self.size());
        println!("\tSection size: {}", self.section_size());
        println!("\tName: {}", self.name());
    }

}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionType {
    NULL,
    SYMTAB,
    STRTAB,
    REL,
    DATA,
    SUBRT
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