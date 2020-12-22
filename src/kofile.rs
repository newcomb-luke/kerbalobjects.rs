use std::{error::Error};

use crate::{FILE_VERSION, MAGIC_NUMBER, KOFileReader, KOFileWriter, HeaderTable, SymbolDataSection, StringTable, SymbolTable, RelSection, DebugSection, SymbolType, SectionType, SectionHeader, Symbol};

/// A structure that represents an entire KO file
pub struct KOFile {
    file_length: u32,
    version: u8,
    number_sections: u16,
    header_table: HeaderTable,
    symstrtab: StringTable,
    symdata: SymbolDataSection,
    symtab: SymbolTable,
    debug_section: Option<DebugSection>,
    code_sections: Vec<RelSection>,
    string_tables: Vec<StringTable>,
    is_entry_point: bool,
}

impl KOFile {

    /// Creates a new KO file from nothing
    pub fn new() -> KOFile {
        KOFile {
            file_length: 0,
            version: FILE_VERSION,
            number_sections: 4,
            header_table: HeaderTable::new(vec![
                SectionHeader::new(SectionType::NULL, 0, 0, ""),
                SectionHeader::new(SectionType::STRTAB, 0, 0, ".symstrtab"),
                SectionHeader::new(SectionType::DATA, 0, 0, ".data"),
                SectionHeader::new(SectionType::SYMTAB, 0, 0, ".symtab"),
            ]),
            symstrtab: StringTable::new(".symstrtab"),
            symdata: SymbolDataSection::new(".data"),
            symtab: SymbolTable::new(".symtab"),
            debug_section: None,
            code_sections: Vec::new(),
            string_tables: Vec::new(),
            is_entry_point: false
        }
    }

    /// Reads a KO file from the provided reader and returns it if it was successful
    pub fn read(reader: &mut KOFileReader) -> Result<KOFile, Box<dyn Error>> {

        let (file_length, version, number_sections) = KOHeader::read(reader)?;

        let header_table = HeaderTable::read(reader, number_sections)?;

        // This will validate the KO file's section order conventions
        header_table.validate_conventions()?;

        // Read all of the sections that are supposed to be there
        let symstrtab = StringTable::read(reader, header_table.get_header(1)?)?;
        let symdata = SymbolDataSection::read(reader, header_table.get_header(2)?)?;
        let symtab = SymbolTable::read(reader, header_table.get_header(3)?, &symstrtab, &symdata)?;

        let mut code_sections = Vec::new();
        let mut string_tables = Vec::new();
        let mut debug_section = None;

        let mut is_entry_point = false;

        // All sections after the first 4 that are required
        for i in 4..number_sections as usize {
            let header_ref = header_table.get_header(i)?;

            // If it is a relocatable section, it is a function, or the main entry point or the .init section
            if header_ref.get_type() == SectionType::REL {
                // Read in the section
                let rel = RelSection::read(reader, header_table.get_header(i)?)?;

                // Check if this is a .text section which would make this KO file the entry point of a program
                if rel.name() == ".text" {
                    is_entry_point = true;
                }

                code_sections.push(rel);
            // We can have random string tables but none of them will be recognized, still add it to the list thought
            } else if header_ref.get_type() == SectionType::STRTAB {
                // Read it
                let strtab = StringTable::read(reader, header_table.get_header(i)?)?;

                // Add it
                string_tables.push(strtab);
            // If it is a debug section, we can only have one of those per file
            } else if header_ref.get_type() == SectionType::DEBUG {
                debug_section = Some(DebugSection::read(reader, header_table.get_header(i)?)?);
            }
        }

        // Create the struct
        let kofile = KOFile {
            file_length,
            version,
            number_sections,
            header_table,
            symstrtab,
            symdata,
            symtab,
            debug_section,
            code_sections,
            string_tables,
            is_entry_point
        };

        Ok(kofile)
    }

    /// Writes a KO file to the provided writer
    pub fn write(&mut self, writer: &mut KOFileWriter) -> Result<(), Box<dyn Error>> {

        // Make sure that all of the section headers are valid
        self.regenerate_headers();

        // Write the magic numbers and other KO file data
        KOHeader::write(writer, (self.file_length, self.version, self.number_sections))?;

        // Does what each says
        self.header_table.write(writer)?;

        self.symstrtab.write(writer)?;

        self.symdata.write(writer)?;

        self.symtab.write(writer)?;

        // Write each code section
        for code_section in self.code_sections.iter() {
            code_section.write(writer)?;
        }

        // Write each string table
        for string_table in self.string_tables.iter() {
            string_table.write(writer)?;
        }

        // If we have a debug section, write that
        match &self.debug_section {
            Some(debug) => debug.write(writer)?,
            None => (),
        }

        Ok(())
    }

    /// This function returns true if the KO file contains the main .text section
    pub fn is_entry_point(&self) -> bool {
        self.is_entry_point
    }

    /// Returns a mutable reference to the KO file's current header table
    pub fn get_mut_header_table(&mut self) -> &mut HeaderTable {
        &mut self.header_table
    }

    /// Returns a mutable reference to the KO file's symbol string table
    pub fn get_mut_symstrtab(&mut self) -> &mut StringTable {
        &mut self.symstrtab
    }

    /// Returns a mutable reference to the KO file's symbol data section
    pub fn get_mut_symdata(&mut self) -> &mut SymbolDataSection {
        &mut self.symdata
    }

    /// Returns a mutable reference to the KO file's symbol table
    pub fn get_mut_symtab(&mut self) -> &mut SymbolTable {
        &mut self.symtab
    }

    /// Returns a reference to the KO file's current header table
    pub fn get_header_table(&mut self) -> &HeaderTable {
        &self.header_table
    }

    /// Returns a reference to the KO file's symbol string table
    pub fn get_symstrtab(&mut self) -> &StringTable {
        &self.symstrtab
    }

    /// Returns a reference to the KO file's symbol data section
    pub fn get_symdata(&mut self) -> &SymbolDataSection {
        &self.symdata
    }

    /// Returns a reference to the KO file's symbol table
    pub fn get_symtab(&mut self) -> &SymbolTable {
        &self.symtab
    }

    /// Returns an option that either contains this KO file's debug section if it has one, or None
    pub fn get_debug(&mut self) -> Option<&mut DebugSection> {
        match &mut self.debug_section {
            Some(debug_section) => Some(debug_section),
            None => None,
        }
    }

    /// Returns a reference to the internal vector of code sections
    pub fn get_code_sections(&mut self) -> &Vec<RelSection> {
        &self.code_sections
    }

    /// Returns a reference to the internal vector of string tables
    pub fn get_string_tables(&mut self) -> &Vec<StringTable> {
        &self.string_tables
    }

    /// Adds a symbol to the symbol table and symbol string tables in the proper way
    pub fn add_symbol(&mut self, symbol: Symbol) -> usize {

        let mut symbol = symbol;

        let name_index = self.symstrtab.add(symbol.name());

        symbol.set_name_index(name_index);

        match symbol.get_type() {
            SymbolType::FUNC => {},
            SymbolType::SECTION => {},
            SymbolType::NOTYPE | SymbolType::OBJECT => {
                let value_index = self.symdata.add(symbol.value().clone());

                symbol.set_value_index(value_index);
            }
        }

        self.symtab.add(symbol)
    }

    /// This function adds a code section to the internal vector of code sections
    pub fn add_code_section(&mut self, code_section: RelSection) {
        // Just in case, check if this is a .text section so we know if this is an entry point
        if code_section.name() == ".text" {
            self.is_entry_point = true;
        }
        // Add it to the vector
        self.code_sections.push(code_section);
    }

    /// This function adds a string table to the internal vector of string tables
    pub fn add_string_table(&mut self, string_table: StringTable) {
        self.string_tables.push(string_table);
    }

    /// This function sets this KO file's debug section
    pub fn set_debug(&mut self, debug: DebugSection) {
        self.debug_section = Some(debug);
    }

    /// Returns the length of the entire file in bytes
    pub fn length(&self) -> u32 {
        self.file_length
    }

    /// Returns the current KO file version number of this file
    pub fn version(&self) -> u8 {
        self.version
    }

    /// Returns the number of sections that this KO file contains
    pub fn num_sections(&self) -> u16 {
        self.number_sections
    }

    /// Regenerates all section headers and their associated data like sizes and offsets
    pub fn regenerate_headers(&mut self) {

        // Start at 11 because that is how long the KOHeader is
        let mut new_length = 11;
        let mut header_size = 0;
        let mut current_offset = new_length;
        let mut number_sections = 0;

        let null_sh = SectionHeader::new(SectionType::NULL, 0, 0, "");
        header_size += null_sh.size();
        number_sections += 1;

        let mut symstrtab_sh = SectionHeader::new(SectionType::STRTAB, 0, self.symstrtab.size(), self.symstrtab.name());
        header_size += symstrtab_sh.size();
        number_sections += 1;

        let mut symdata_sh = SectionHeader::new(SectionType::DATA, 0, self.symdata.size(), self.symdata.name());
        header_size += symdata_sh.size();
        number_sections += 1;

        let mut symtab_sh = SectionHeader::new(SectionType::SYMTAB, 0, self.symtab.size(), self.symtab.name());
        header_size += symtab_sh.size();
        number_sections += 1;

        let mut code_shs = Vec::new();
        let mut string_shs = Vec::new();

        for code_section in self.code_sections.iter() {
            let code_sh = SectionHeader::new(SectionType::REL, 0, code_section.size(), code_section.name());
            header_size += code_sh.size();
            number_sections += 1;
            code_shs.push(code_sh);
        }

        for string_table in self.string_tables.iter() {
            let string_sh = SectionHeader::new(SectionType::STRTAB, 0, string_table.size(), string_table.name());
            header_size += string_sh.size();
            number_sections += 1;
            string_shs.push(string_sh);
        }
        let mut debug_sh = match &self.debug_section {
            Some(debug) => {
                let sh = SectionHeader::new(SectionType::DATA, 0, debug.size(), debug.name());
                header_size += sh.size();
                number_sections += 1;
                Some(sh)
            },
            None => None
        };

        // Now we are finally done calculating the section header size!
        // Now we need to apply it
        current_offset += header_size;

        symstrtab_sh.set_offset(current_offset);
        current_offset += symstrtab_sh.section_size();

        symdata_sh.set_offset(current_offset);
        current_offset += symdata_sh.section_size();

        symtab_sh.set_offset(current_offset);
        current_offset += symtab_sh.section_size();

        for code_sh in code_shs.iter_mut() {
            code_sh.set_offset(current_offset);
            current_offset += code_sh.section_size();
        }

        for string_sh in string_shs.iter_mut() {
            string_sh.set_offset(current_offset);
            current_offset += string_sh.section_size();
        }

        match &mut debug_sh {
            Some(sh) => {
                sh.set_offset(current_offset);
                current_offset += sh.section_size();
            },
            None => {}
        }

        new_length = current_offset;

        // Now that we are finally done populating all of the offsets, it is time to make the final array

        let mut headers = vec![
            null_sh,
            symstrtab_sh,
            symdata_sh,
            symtab_sh
        ];

        for code_sh in code_shs {
            headers.push(code_sh);
        }

        for string_sh in string_shs {
            headers.push(string_sh);
        }

        match debug_sh {
            Some(debug) => {
                headers.push(debug);
            },
            None => {}
        }

        let new_headers = HeaderTable::new(headers);

        self.header_table = new_headers;
        self.file_length = new_length;
        self.number_sections = number_sections;
    }

}

pub struct KOHeader {}

impl KOHeader {
    pub fn new(file_length: u32, number_sections: u16) -> (u32, u8, u16) {
        (file_length, FILE_VERSION, number_sections)
    }

    pub fn read(reader: &mut KOFileReader) -> Result<(u32, u8, u16), Box<dyn Error>> {
        
        let magic_number = reader.read_uint32()?;

        if magic_number != MAGIC_NUMBER {
            return Err("All input files must be KerbalObject files.".into());
        }

        let file_length = reader.read_uint32()?;
        let version = reader.next()?;
        let number_sections = reader.read_uint16()?;

        Ok((file_length, version, number_sections))

    }

    pub fn write(writer: &mut KOFileWriter, header: (u32, u8, u16)) -> Result<(), Box<dyn Error>> {

        let (file_length, version, number_sections) = header;

        writer.write_uint32(MAGIC_NUMBER)?;

        writer.write_uint32(file_length)?;

        writer.write(version)?;

        writer.write_uint16(number_sections)?;

        Ok(())
    }
}