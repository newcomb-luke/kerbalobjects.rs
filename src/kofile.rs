use std::{error::Error};

use crate::{FILE_VERSION, MAGIC_NUMBER, KOFileReader, KOFileWriter, HeaderTable, SymbolDataSection, StringTable, SymbolTable, RelSection, DebugSection, SubrtSection, SymbolType, SectionType, SectionHeader, Symbol};

pub struct KOFile {
    file_length: u32,
    version: u8,
    number_sections: u16,
    header_table: HeaderTable,
    symstrtab: StringTable,
    symdata: SymbolDataSection,
    symtab: SymbolTable,
    main_text: Option<RelSection>,
    init_section: Option<RelSection>,
    comment_section: Option<StringTable>,
    debug_section: Option<DebugSection>,
    subrt_sections: Vec<SubrtSection>,
    is_entry_point: bool,
}

impl KOFile {

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
            main_text: None,
            init_section: None,
            comment_section: None,
            debug_section: None,
            subrt_sections: Vec::new(),
            is_entry_point: false
        }
    }

    pub fn read(reader: &mut KOFileReader) -> Result<KOFile, Box<dyn Error>> {

        let (file_length, version, number_sections) = KOHeader::read(reader)?;

        let header_table = HeaderTable::read(reader, number_sections)?;

        header_table.validate_conventions()?;

        let symstrtab = StringTable::read(reader, header_table.get_header(1)?)?;

        let symdata = SymbolDataSection::read(reader, header_table.get_header(2)?)?;

        let symtab = SymbolTable::read(reader, header_table.get_header(3)?, &symstrtab, &symdata)?;

        let mut main_text = None;
        let mut init_section = None;
        let mut comment_section = None;
        let mut debug_section = None;

        let mut subrt_sections = Vec::new();

        let mut is_entry_point= false;

        for i in 4..number_sections as usize {
            let header_ref = header_table.get_header(i)?;

            // If it is a relocatable section, it is the main entry point or the .init section
            if header_ref.get_type() == SectionType::REL {
                // If it is labeled as .text, read it in as .text
                if header_ref.name() == ".text" {
                    if main_text.is_some() {
                        return Err("Multiple .text sections found in the same file".into());
                    }

                    let rel = RelSection::read(reader, header_table.get_header(i)?)?;

                    main_text = Some(rel);
                    is_entry_point = true;
                }
                // If it is labeled as .init, read it in as .init
                else if header_ref.name() == ".init" {
                    if init_section.is_some() {
                        return Err("Multiple .init sections found in the same file".into());
                    }
                    let rel = RelSection::read(reader, header_table.get_header(i)?)?;

                    init_section = Some(rel);
                }
                else {
                    return Err(format!("Found relocatable instruction section at index {}, but it is not labeled as .text or .init; invalid format.", i).into());
                }
            } else if header_ref.get_type() == SectionType::STRTAB {

                let strtab = StringTable::read(reader, header_table.get_header(i)?)?;

                comment_section = Some(strtab);
            } else if header_ref.get_type() == SectionType::DATA {
                debug_section = Some(DebugSection::read(reader, header_table.get_header(i)?)?);
            } else {
                let subrt = SubrtSection::read(reader, header_table.get_header(i)?)?;

                subrt_sections.push( subrt );
            }
        }

        let kofile = KOFile {
            file_length,
            version,
            number_sections,
            header_table,
            symstrtab,
            symdata,
            symtab,
            main_text,
            init_section,
            comment_section,
            debug_section,
            subrt_sections,
            is_entry_point
        };

        Ok(kofile)
    }

    pub fn write(&mut self, writer: &mut KOFileWriter) -> Result<(), Box<dyn Error>> {

        self.regenerate_headers();

        KOHeader::write(writer, (self.file_length, self.version, self.number_sections))?;

        self.header_table.write(writer)?;

        self.symstrtab.write(writer)?;

        self.symdata.write(writer)?;

        self.symtab.write(writer)?;

        match &self.main_text {
            Some(main) => main.write(writer)?,
            None => (),
        }

        match &self.init_section {
            Some(init) => init.write(writer)?,
            None => (),
        }

        match &self.comment_section {
            Some(comment) => comment.write(writer)?,
            None => (),
        }

        match &self.debug_section {
            Some(debug) => debug.write(writer)?,
            None => (),
        }

        for subrt_section in self.subrt_sections.iter() {
            subrt_section.write(writer)?;
        }

        Ok(())
    }

    pub fn is_entry_point(&self) -> bool {
        self.is_entry_point
    }

    pub fn get_header_table(&mut self) -> &mut HeaderTable {
        &mut self.header_table
    }

    pub fn get_symstrtab(&mut self) -> &mut StringTable {
        &mut self.symstrtab
    }

    pub fn get_symdata(&mut self) -> &mut SymbolDataSection {
        &mut self.symdata
    }

    pub fn get_symtab(&mut self) -> &mut SymbolTable {
        &mut self.symtab
    }

    pub fn get_main_text(&mut self) -> Option<&mut RelSection> {
        match &mut self.main_text {
            Some(main_text) => Some(main_text),
            None => None,
        }
    }

    pub fn get_init(&mut self) -> Option<&mut RelSection> {
        match &mut self.init_section {
            Some(init_section) => Some(init_section),
            None => None,
        }
    }

    pub fn get_comment(&mut self) -> Option<&mut StringTable> {
        match &mut self.comment_section {
            Some(comment_section) => Some(comment_section),
            None => None,
        }
    }

    pub fn get_debug(&mut self) -> Option<&mut DebugSection> {
        match &mut self.debug_section {
            Some(debug_section) => Some(debug_section),
            None => None,
        }
    }

    pub fn get_subrt_sections(&mut self) -> &Vec<SubrtSection> {
        &self.subrt_sections
    }

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

    pub fn add_subrt_section(&mut self, subrt_section: SubrtSection) {
        self.subrt_sections.push(subrt_section);
    }

    pub fn set_main_text(&mut self, main_text: RelSection) {
        self.main_text = Some(main_text);
    }

    pub fn set_init(&mut self, init: RelSection) {
        self.init_section = Some(init);
    }

    pub fn set_comment(&mut self, comment: StringTable) {
        self.comment_section = Some(comment);
    }

    pub fn set_debug(&mut self, debug: DebugSection) {
        self.debug_section = Some(debug);
    }

    pub fn length(&self) -> u32 {
        self.file_length
    }

    pub fn version(&self) -> u8 {
        self.version
    }

    pub fn num_sections(&self) -> u16 {
        self.number_sections
    }

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

        let mut main_text_sh = match &self.main_text {
            Some(text) => {
                let sh = SectionHeader::new(SectionType::REL, 0, text.size(), text.name());
                header_size += sh.size();
                number_sections += 1;
                Some(sh)
            },
            None => None
        };

        let mut subrt_shs = Vec::new();

        for subrt in self.subrt_sections.iter() {
            let subrt_sh = SectionHeader::new(SectionType::SUBRT, 0, subrt.size(), subrt.name());
            header_size += subrt_sh.size();
            number_sections += 1;
            subrt_shs.push( subrt_sh );
        }

        let mut init_sh = match &self.init_section {
            Some(init) => {
                let sh = SectionHeader::new(SectionType::REL, 0, init.size(), init.name());
                header_size += sh.size();
                number_sections += 1;
                Some(sh)
            },
            None => None
        };

        let mut comment_sh = match &self.comment_section {
            Some(comment) => {
                let sh = SectionHeader::new(SectionType::STRTAB, 0, comment.size(), comment.name());
                header_size += sh.size();
                number_sections += 1;
                Some(sh)
            },
            None => None
        };

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

        match &mut main_text_sh {
            Some(sh) => {
                sh.set_offset(current_offset);
                current_offset += sh.section_size();
            },
            None => {}
        }

        for subrt_sh in subrt_shs.iter_mut() {
            subrt_sh.set_offset(current_offset);
            current_offset += subrt_sh.section_size();
        }

        match &mut init_sh {
            Some(sh) => {
                sh.set_offset(current_offset);
                current_offset += sh.section_size();
            },
            None => {}
        }

        match &mut comment_sh {
            Some(sh) => {
                sh.set_offset(current_offset);
                current_offset += sh.section_size();
            },
            None => {}
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

        match main_text_sh {
            Some(text) => {
                headers.push(text);
            },
            None => {}
        }

        for subrt_sh in subrt_shs {
            headers.push(subrt_sh);
        }

        match init_sh {
            Some(init) => {
                headers.push(init);
            },
            None => {}
        }

        match comment_sh {
            Some(comment) => {
                headers.push(comment);
            },
            None => {}
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