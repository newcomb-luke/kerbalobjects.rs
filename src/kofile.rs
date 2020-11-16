use std::{error::Error};

use crate::{StringTable, SectionType, KOFileWriter, FILE_VERSION, KOFileReader, SectionHeaderTable, SectionHeader};

pub struct KOFile {
    header: KOHeader,
    section_headers: SectionHeaderTable,
    strtabs: Vec<StringTable>,
    shstrtab_vec_index: usize,
    symbol_strtab_vec_index: usize,
}

impl KOFile {

    pub fn from_parts(header: KOHeader, section_headers: SectionHeaderTable) -> KOFile {
        KOFile {
            header,
            section_headers,
            strtabs: Vec::new(),
            shstrtab_vec_index: 0,
            symbol_strtab_vec_index: 0,
        }
    }

    pub fn new() -> KOFile {

        let mut header = KOHeader::new();

        let mut section_headers = SectionHeaderTable::new();

        let mut shstrtab = StringTable::new(".shstrtab");
        shstrtab.set_index(1);

        section_headers.add( SectionHeader::new(shstrtab.add(".shstrtab"), String::from(".shstrtab"), SectionType::STRTAB, header.get_file_length()) );

        header.inc_number_sections();

        let strtabs = vec![shstrtab];

        KOFile {
            header,
            section_headers,
            strtabs,
            shstrtab_vec_index: 0,
            symbol_strtab_vec_index: 0,
        }
    }

    pub fn add_strtab(&mut self, strtab: StringTable) -> u16 {

        // The index into the section header table will be this
        let index = self.header.get_number_sections();

        // Actually add it to the list of string 
        self.strtabs.push(strtab);

        // The index into the vector of string tables is this
        let vec_index = self.strtabs.len() - 1;

        // Get a mutable reference to the string table
        let table_ref = self.strtabs.get_mut(vec_index).unwrap();

        // Set the index of the string table now that we know it
        table_ref.set_index(index);

        // Get the name of the string table as a string
        let table_name = table_ref.get_name().to_owned();

        // NOTE: We must do anything that we want to do with the string table itself above, because below we borrow the vector mutably

        // Get the index of the string table in the section header string table
        let name_index = self.strtabs.get_mut(0).unwrap().add(&table_name);

        // Add a header for the string table
        self.section_headers.add( SectionHeader::new(name_index, table_name, SectionType::STRTAB, 0 ) );

        // Add 1 to the number of sections
        self.header.inc_number_sections();

        index
    }

    pub fn get_shstrtab(&mut self) -> Result<&mut StringTable, Box<dyn Error>> {
        match self.strtabs.get_mut(self.shstrtab_vec_index) {
            Some(r) => Ok(r),
            None => Err("Tried to get the section header string table from a KO file with none.".into()),
        }
    }

    pub fn get_sym_strtab(&mut self) -> Result<&mut StringTable, Box<dyn Error>> {
        match self.strtabs.get_mut(self.symbol_strtab_vec_index) {
            Some(r) => Ok(r),
            None => Err("Tried to get the symbol string table from a KO file with none.".into()),
        }
    }

    pub fn update_offsets_sizes(&mut self) -> Result<(), Box<dyn Error>>{

        self.header.add_to_length(self.section_headers.size());

        for i in 1..self.header.get_number_sections() + 1 {

            // TODO: Support all the section types
            for strtab in self.strtabs.iter() {
                if strtab.get_index() == i {
                    self.section_headers.get_mut(i)?.set_offset(self.header.get_file_length());

                    self.header.add_to_length( strtab.size() );
                }
            }

        }

        Ok(())
    }

    pub fn write(&mut self, writer: &mut KOFileWriter) -> Result<(), Box<dyn Error>> {

        self.update_offsets_sizes()?;

        self.header.write(writer)?;

        self.section_headers.write(writer)?;

        for i in 1..self.header.get_number_sections() + 1 {

            println!("\tWriting section {}", i);

            for strtab in self.strtabs.iter_mut() {

                if strtab.get_index() == i {
                    strtab.write(writer)?;
                }
            }

        }

        // Finally, actually write the contents of the file
        writer.write_to_file()?;

        Ok(())
    }

    pub fn read(reader: &mut KOFileReader) -> Result<KOFile, Box<dyn Error>> {

        let header: KOHeader;
        let mut section_headers: SectionHeaderTable;
        let mut strtabs: Vec<StringTable> = Vec::new();
        let mut shstrtab_vec_index = 0;
        let mut symbol_strtab_vec_index = 0;

        header = KOHeader::read(reader)?;

        section_headers = SectionHeaderTable::read(reader, header.number_sections, header.file_length)?;

        for i in 0..header.get_number_sections() {

            println!("\tReading section {}", i+1);

            let header_ref = section_headers.get(i)?;

            if header_ref.get_type() == SectionType::STRTAB {

                let strtab = StringTable::read(reader, header_ref.size())?;

                if i == header.get_shstrtab_index() as u16 {
                    shstrtab_vec_index = strtabs.len();
                }

                strtabs.push( strtab );
            }

        }

        for header in section_headers.get_headers().iter_mut() {

            let shstrtab = match strtabs.get(shstrtab_vec_index) {
                Some(r) => r,
                None => return Err("Tried to get the section header string table from a KO file with none.".into())
            };

            header.set_name( &shstrtab.get_string_at(header.get_name_index())? );
        }

        for (index, strtab) in strtabs.iter().enumerate() {
            if strtab.get_name() == ".strtab" {
                symbol_strtab_vec_index = index;
            }
        }

        Ok(KOFile {
            header,
            section_headers,
            strtabs,
            shstrtab_vec_index,
            symbol_strtab_vec_index
        })

    }

}

pub struct KOHeader {
    file_length: u32,
    version: u8,
    number_sections: u16,
    shstrtab_index: u8
}

impl KOHeader {

    pub fn from_parts(file_length: u32, version: u8, number_sections: u16, shstrtab_index: u8) -> KOHeader {
        KOHeader {
            file_length,
            version,
            number_sections,
            shstrtab_index
        }
    }

    pub fn new() -> KOHeader {
        KOHeader {
            file_length: 12, // The KO file header is 12 bytes long in total
            version: FILE_VERSION,
            number_sections: 1,
            shstrtab_index: 1 // Default to 1. This way there is room for the null section if there is nothing else.
        }
    }

    pub fn read(reader: &mut KOFileReader) -> Result<KOHeader, Box<dyn Error>> {

        println!("\tHeader:");

        // We need to make sure that this file is actually a KOFile
        let magic_number = reader.read_int32()? as u32;
        // The magic number is 'k' 1 'o' 'f' or as one u32: 0x6b016f66
        // Because we are writing using little endian though, it needs to be: 0x666f016b

        if magic_number != 0x666f016b {
            return Err("Input files must all be KerbalObject files (.ko)".into());
        }

        // The files length is recorded as one u32
        let file_length = reader.read_int32()? as u32;

        println!("\t\tLength: {}", file_length);

        // The file's version (should only really be v1 at this point)
        let version = reader.next()?;

        println!("\t\tVersion: {}", version);

        let number_sections = reader.read_int16()? as u16;

        println!("\t\t{} sections", number_sections);

        let strtab_index = reader.next()?;

        println!("\t\tString table index: {}", strtab_index);

        Ok(KOHeader::from_parts(file_length, version, number_sections, strtab_index))

    }

    pub fn set_file_length(&mut self, file_length: u32) {
        self.file_length = file_length;
    }

    pub fn get_file_length(&self) -> u32 {
        self.file_length
    }

    pub fn add_to_length(&mut self, length: u32) {
        self.file_length += length;
    }

    pub fn set_version(&mut self, version: u8) {
        self.version = version;
    }

    pub fn set_number_sections(&mut self, number_sections: u16) {
        self.number_sections = number_sections;
    }

    pub fn inc_number_sections(&mut self) {
        self.number_sections += 1;
    }

    pub fn set_shstrtab_index(&mut self, shstrtab_index: u8) {
        self.shstrtab_index = shstrtab_index;
    }

    pub fn get_version(&self) -> u8 {
        self.version
    }

    pub fn get_number_sections(&self) -> u16 {
        self.number_sections
    }

    pub fn get_shstrtab_index(&self) -> u8 {
        self.shstrtab_index
    }

    pub fn write(&self, writer: &mut KOFileWriter) -> Result<(), Box<dyn Error>> {

        // The magic number is 'k' 1 'o' 'f' or as one u32: 0x6b016f66
        // Because we are writing using little endian though, it needs to be: 0x666f016b
        let magic_number: u32 = 0x666f016b;

        writer.write_uint32(magic_number)?;

        println!("File length is {} bytes", self.file_length);

        writer.write_uint32(self.file_length)?;

        println!("File version is {}", self.version);

        writer.write(self.version)?;

        println!("File will have {} sections", self.number_sections);

        writer.write_uint16(self.number_sections)?;

        writer.write(self.shstrtab_index)?;

        Ok(())

    }

}