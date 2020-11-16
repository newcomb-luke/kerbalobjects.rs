use std::{error::Error};

use crate::{SectionType, KOFileWriter, KOFileReader};

pub struct SectionHeaderTable {
    headers: Vec<SectionHeader>,
}

impl SectionHeaderTable {

    pub fn from_parts(headers: Vec<SectionHeader>) -> SectionHeaderTable {
        SectionHeaderTable { headers }
    }

    pub fn new() -> SectionHeaderTable {
        SectionHeaderTable { headers: vec![SectionHeader::new(0, String::from(""), SectionType::NULL, 0)] }
    }

    pub fn add(&mut self, sh: SectionHeader) {
        self.headers.push(sh);
    }

    pub fn get(&self, index: u16) -> Result<&SectionHeader, Box<dyn Error>> {
        match self.headers.get(index as usize) {
            Some(header) => Ok(header),
            None => Err(format!("Tried to index section {} of the KO file. Does not exist.", index).into())
        }
    }

    pub fn get_mut(&mut self, index: u16) -> Result<&mut SectionHeader, Box<dyn Error>> {
        match self.headers.get_mut(index as usize) {
            Some(header) => Ok(header),
            None => Err(format!("Tried to index section {} of the KO file. Does not exist.", index).into())
        }
    }

    pub fn get_headers(&mut self) -> &mut Vec<SectionHeader> {
        &mut self.headers
    }

    pub fn size(&self) -> u32 {
        7 * self.headers.len() as u32
    }

    pub fn write(&self, writer: &mut KOFileWriter) -> Result<(), Box<dyn Error>> {

        for header in self.headers.iter() {
            header.write(writer)?;
        }

        Ok(())
    }

    pub fn read(reader: &mut KOFileReader, number_of_sections: u16, file_length: u32) -> Result<SectionHeaderTable, Box<dyn Error>> {

        let mut headers: Vec<SectionHeader> = Vec::with_capacity(number_of_sections as usize);

        // Read all the sections in the section header table
        for _ in 0..number_of_sections {
            headers.push(SectionHeader::read(reader)?);
        }

        // Now time to populate the sizes
        for i in 1..number_of_sections as usize {
            
            let section_size;

            let section_offset = headers.get(i).unwrap().get_offset();

            // If this isn't the last header, then the size of this section is the next section's offset minus this one's
            if i < (number_of_sections as usize - 1) {
                let next_header = headers.get(i + 1).unwrap();

                section_size = next_header.get_offset() - section_offset;
            }
            // If not, then actually the size is just the length of the file minus the offset
            else {
                section_size = file_length - section_offset;
            }

            println!("\tSection {}'s Header:", i);

            println!("\t\tOffset: {}", section_offset);
            println!("\t\tSize: {}", section_size);
            println!("\t\tName index: {}", headers.get(i).unwrap().get_name_index());
            println!("\t\tType: {:?}", headers.get(i).unwrap().get_type());

            headers.get_mut(i).unwrap().set_size(section_size);
        }

        Ok(SectionHeaderTable::from_parts(headers))

    }

}

pub struct SectionHeader {
    name_index: u16,
    name: String,
    section_type: SectionType,
    offset: u32,
    size: u32
}

impl SectionHeader {

    pub fn new(name_index: u16, name: String, section_type: SectionType, offset: u32) -> SectionHeader {

        SectionHeader {
            name_index,
            name,
            section_type,
            offset,
            size: 0
        }

    }

    pub fn write(&self, writer: &mut KOFileWriter) -> Result<(), Box<dyn Error>> {

        writer.write_uint16(self.name_index)?;

        writer.write(self.section_type.to_byte())?;

        writer.write_uint32(self.offset)?;
        
        Ok(())
    }

    pub fn read(reader: &mut KOFileReader) -> Result<SectionHeader, Box<dyn Error>> {

        let name_index = reader.read_int16()? as u16;

        let name = String::new();

        let section_type = SectionType::from(reader.next()?)?;

        let offset = reader.read_int32()? as u32;

        Ok(SectionHeader::new(name_index, name, section_type, offset))

    }
 
    pub fn get_type(&self) -> SectionType {
        self.section_type
    }

    pub fn set_offset(&mut self, offset: u32) {
        self.offset = offset;
    }

    pub fn get_offset(&self) -> u32 {
        self.offset
    }

    pub fn set_size(&mut self, size: u32) {
        self.size = size;
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn get_name_index(&self) -> u16 {
        self.name_index
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn set_name(&mut self, name: &String) {
        self.name = String::from(name);
    }

}