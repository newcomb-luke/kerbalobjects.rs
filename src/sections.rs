use std::{error::Error, hash::Hash, hash::Hasher};
use std::collections::{HashMap, hash_map::DefaultHasher};

use crate::{KOFileReader, KOFileWriter, SectionHeader, RelInstruction};

pub struct StringTable {
    strings: Vec<String>,
    name: String,
    size: u32,
}

impl StringTable {

    pub fn new(name: &str) -> StringTable {
        StringTable {
            strings: vec![String::new()],
            name: name.to_owned(),
            size: 1,
        }
    }

    pub fn read(reader: &mut KOFileReader, header: &SectionHeader) -> Result<StringTable, Box<dyn Error>> {

        // Consume that first \0, because the string table is intialized to a length of 1 already
        reader.next()?;

        let mut strtab = StringTable::new(header.name());

        while strtab.size() < header.section_size() {
            strtab.add_no_check(&reader.read_string()?);
        }

        if strtab.size() > header.section_size() {
            return Err("Error reading string table, size mismatch".into());
        }

        Ok(strtab)

    }

    pub fn write(&self, writer: &mut KOFileWriter) -> Result<(), Box<dyn Error>> {

        for s in self.strings.iter() {
            writer.write_string(s)?;
        }

        Ok(())
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn add(&mut self, s: &str) -> usize {

        let strval = s.to_owned();

        if self.strings.contains(&strval) {
            match self.strings.binary_search(&strval) {
                Ok(idx) => { return idx; },
                _ => unreachable!()
            }
        }

        self.strings.push(strval);

        self.size += s.len() as u32 + 1;

        self.strings.len() - 1
    }

    pub fn add_no_check(&mut self, s: &str) -> usize {

        self.strings.push(s.to_owned());

        self.size += s.len() as u32 + 1;

        self.strings.len() - 1
    }

    pub fn get(&self, index: usize) -> Result<&String, Box<dyn Error>>{
        match self.strings.get(index) {
            Some(s) => Ok(s),
            None => Err(format!("Could not find string at index {} in string table.", index).into()),
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn debug_print(&self) {
        println!("\n\tString table: {}", self.name);
        println!("\t\tSize: {}", self.size);

        for (i, s) in self.strings.iter().enumerate() {
            println!("\t\tString {}: \"{}\"", i, s);
        }
    }

}

pub struct SymbolTable {
    symbols: Vec<Symbol>,
    name: String,
    size: u32,
    hash_to_index: HashMap<u64, usize>,
}

impl SymbolTable {

    pub fn new(name: &str) -> SymbolTable {
        SymbolTable {
            symbols: Vec::new(),
            name: name.to_owned(),
            size: 0,
            hash_to_index: HashMap::new(),
        }
    }

    pub fn read(reader: &mut KOFileReader, header: &SectionHeader, symstrtab: &StringTable, symdata: &SymbolDataSection) -> Result<SymbolTable, Box<dyn Error>> {

        let mut symtab = SymbolTable::new(header.name());

        while symtab.size() < header.section_size() {
            symtab.add_no_check(Symbol::read(symstrtab, symdata, reader)?);
        }

        if symtab.size() > header.section_size() {
            return Err("Error reading symbol table, size mismatch".into());
        }

        Ok(symtab)

    }

    pub fn write(&self, writer: &mut KOFileWriter) -> Result<(), Box<dyn Error>> {

        for s in self.symbols.iter() {
            s.write(writer)?;
        }

        Ok(())
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn add(&mut self, s: Symbol) -> usize {

        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        let s_hash = hasher.finish();

        let s_index = self.symbols.len();

        match self.hash_to_index.get(&s_hash) {
            Some(index) => { return *index; },
            None => { self.hash_to_index.insert(s_hash, s_index); }
        }

        self.symbols.push(s);

        self.size += self.symbols.last().unwrap().width();

        s_index
    }

    pub fn add_no_check(&mut self, s: Symbol) -> usize {
        self.symbols.push(s);

        self.size += self.symbols.last().unwrap().width();

        self.symbols.len() - 1
    }

    pub fn get(&self, index: usize) -> Result<&Symbol, Box<dyn Error>>{
        match self.symbols.get(index) {
            Some(s) => Ok(s),
            None => Err(format!("Could not find symbol at index {} in symbol table.", index).into()),
        }
    }

    pub fn get_symbols(&self) -> &Vec<Symbol> {
        &self.symbols
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn debug_print(&self) {
        println!("\n\tSymbol table: {}", self.name);
        println!("\t\tSize: {}", self.size);

        for (i, s) in self.symbols.iter().enumerate() {
            print!("\t\tSymbol {}: ", i);

            s.debug_print();
        }
    }
}

#[derive(Debug, Clone)]
pub struct Symbol {
    name_index: usize,
    value_index: usize,
    name: String,
    value: KOSValue,
    size: u16,
    info: SymbolInfo,
    symbol_type: SymbolType,
    section_index: usize,
}

impl Hash for Symbol {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.to_string().hash(state);
    }
}

impl Symbol {

    pub fn new(name: &str, value: KOSValue, size: u16, info: SymbolInfo, symbol_type: SymbolType, section_index: usize) -> Symbol {
        Symbol {
            name_index: 0,
            value_index: 0,
            name: name.to_owned(),
            value,
            size,
            info,
            symbol_type,
            section_index
        }
    }

    pub fn read(symstrtab: &StringTable, symdata: &SymbolDataSection, reader: &mut KOFileReader) -> Result<Symbol, Box<dyn Error>> {
        let name_index = reader.read_uint32()? as usize;
        let value_index = reader.read_uint32()? as usize;
        let size = reader.read_uint16()?;
        let info = SymbolInfo::from(reader.next()?)?;
        let symbol_type = SymbolType::from(reader.next()?)?;
        let section_index = reader.read_uint16()? as usize;

        Ok(Symbol {
            name_index,
            value_index,
            name: symstrtab.get(name_index)?.to_owned(),
            value: symdata.get(value_index)?.to_owned(),
            size,
            info,
            symbol_type,
            section_index,
        })
    }

    pub fn write(&self, writer: &mut KOFileWriter) -> Result<(), Box<dyn Error>> {

        writer.write_uint32(self.name_index as u32)?;

        writer.write_uint32(self.value_index as u32)?;

        writer.write_uint16(self.size)?;

        writer.write(self.info.to_byte())?;

        writer.write(self.symbol_type.to_byte())?;

        writer.write_uint16(self.section_index as u16)?;

        Ok(())
    }

    pub fn size(&self) -> u16 {
        self.size
    }

    /// Symbol table entries are always 14 bytes wide
    pub fn width(&self) -> u32 {
        14
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn value(&self) -> &KOSValue {
        &self.value
    }

    pub fn get_type(&self) -> SymbolType {
        self.symbol_type
    }

    pub fn get_info(&self) -> SymbolInfo {
        self.info
    }

    pub fn set_name_index(&mut self, index: usize) {
        self.name_index = index;
    }

    pub fn set_value_index(&mut self, index: usize) {
        self.value_index = index;
    }

    pub fn debug_print(&self) {
        print!("Name: {}, ", self.name);
        print!("Value index: {}, ", self.value_index);
        print!("Size: {}, ", self.size);
        print!("Info: {:?}, ", self.info);
        print!("Type: {:?}, ", self.get_type());
        println!("Section index: {}", self.section_index);
    }

    pub fn to_string(&self) -> String {
        format!("{}:{}:{}:{:?}:{:?}:{}", self.name, self.value_index, self.size, self.info, self.symbol_type, self.section_index)
    }

}

pub struct SymbolDataSection {
    values: Vec<KOSValue>,
    name: String,
    size: u32,
    hash_to_index: HashMap<u64, usize>,
}

impl SymbolDataSection {

    pub fn new(name: &str) -> SymbolDataSection {
        SymbolDataSection {
            values: Vec::new(),
            name: name.to_owned(),
            size: 0,
            hash_to_index: HashMap::new(),
        }
    }

    pub fn read(reader: &mut KOFileReader, header: &SectionHeader) -> Result<SymbolDataSection, Box<dyn Error>> {

        let mut symdata = SymbolDataSection::new(header.name());

        while symdata.size() < header.section_size() {
            let val = KOSValue::read(reader)?;

            symdata.add_no_check(val);
        }

        if symdata.size() > header.section_size() {
            return Err("Error reading symbol data table, size mismatch".into());
        }

        Ok(symdata)
    }

    pub fn write(&self, writer: &mut KOFileWriter) -> Result<(), Box<dyn Error>> {

        for value in self.values.iter() {
            value.write(writer)?;
        }

        Ok(())
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn add(&mut self, value: KOSValue) -> usize {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        let val_hash = hasher.finish();

        let val_index = self.values.len();

        match self.hash_to_index.get(&val_hash) {
            Some(index) => { return *index; },
            None => { self.hash_to_index.insert(val_hash, val_index); }
        }

        val_index
    }

    pub fn add_no_check(&mut self, value: KOSValue) -> usize {
        self.values.push(value);

        self.size += self.values.last().unwrap().size();

        self.values.len() - 1
    }

    pub fn get(&self, index: usize) -> Result<&KOSValue, Box<dyn Error>>{
        match self.values.get(index) {
            Some(s) => Ok(s),
            None => Err(format!("Could not find value at index {} in symbol data section.", index).into()),
        }
    }

    pub fn get_values(&self) -> &Vec<KOSValue> {
        &self.values
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn debug_print(&self) {
        println!("\n\tSymbol data section: {}", self.name);
        println!("\t\tSize: {}", self.size);

        for (i, s) in self.values.iter().enumerate() {
            println!("\t\tValue {}: \"{:?}\"", i, s);
        }
    }

}

pub struct RelSection {
    instructions: Vec<RelInstruction>,
    name: String,
    size: u32,
}

impl RelSection {

    pub fn new(name: &str) -> RelSection {
        RelSection {
            instructions: Vec::new(),
            name: name.to_owned(),
            size: 0
        }
    }

    pub fn read(reader: &mut KOFileReader, header: &SectionHeader) -> Result<RelSection, Box<dyn Error>> {
        let mut rel_section = RelSection::new(header.name());

        while rel_section.size() < header.section_size() {
            rel_section.add(RelInstruction::read(reader)?);
        }

        if rel_section.size() > header.section_size() {
            return Err("Error reading relocatable instruction section, size mismatch".into());
        }

        Ok(rel_section)
    }

    pub fn write(&self, writer: &mut KOFileWriter) -> Result<(), Box<dyn Error>> {

        for instruction in self.instructions.iter() {
            instruction.write(writer)?;
        }

        Ok(())
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn add(&mut self, instr: RelInstruction) -> usize {
        self.instructions.push(instr);

        self.size += self.instructions.last().unwrap().size();

        self.instructions.len() - 1
    }

    pub fn get(&self, index: usize) -> Result<&RelInstruction, Box<dyn Error>> {
        match self.instructions.get(index) {
            Some(i) => Ok(i),
            None => Err(format!("Could not find instruction at index {} in relocatable instruction section.", index).into()),
        }
    }

    pub fn get_instructions(&self) -> &Vec<RelInstruction> {
        &self.instructions
    }

    pub fn debug_print(&self) {
        println!("\n\tRel section: {}", self.name);
        println!("\t\tSize: {}", self.size);
        
        for instruction in self.instructions.iter() {
            instruction.debug_print();
        }
    }

}

pub struct SubrtSection {
    instructions: Vec<RelInstruction>,
    name: String,
    size: u32,
}

impl SubrtSection {
    pub fn new(name: &str) -> SubrtSection {
        SubrtSection {
            instructions: Vec::new(),
            name: name.to_owned(),
            size: 0
        }
    }

    pub fn read(reader: &mut KOFileReader, header: &SectionHeader) -> Result<SubrtSection, Box<dyn Error>> {
        let mut subrt_section = SubrtSection::new(header.name());

        while subrt_section.size() < header.section_size() {
            subrt_section.add(RelInstruction::read(reader)?);
        }

        if subrt_section.size() > header.section_size() {
            return Err("Error reading subroutine instruction section, size mismatch".into());
        }

        Ok(subrt_section)
    }

    pub fn write(&self, writer: &mut KOFileWriter) -> Result<(), Box<dyn Error>> {

        for instruction in self.instructions.iter() {
            instruction.write(writer)?;
        }

        Ok(())
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn add(&mut self, instr: RelInstruction) -> usize {
        self.instructions.push(instr);

        self.size += self.instructions.last().unwrap().size();

        self.instructions.len() - 1
    }

    pub fn get(&self, index: usize) -> Result<&RelInstruction, Box<dyn Error>> {
        match self.instructions.get(index) {
            Some(i) => Ok(i),
            None => Err(format!("Could not find instruction at index {} in relocatable instruction section.", index).into()),
        }
    }

    pub fn get_instructions(&self) -> &Vec<RelInstruction> {
        &self.instructions
    }

    pub fn debug_print(&self) {
        println!("\nSubrt section: {}", self.name);
        println!("\t\tSize: {}", self.size);
        
        for instruction in self.instructions.iter() {
            instruction.debug_print();
        }
    }
}

pub struct DebugSection {
    entries: Vec<DebugEntry>,
    name: String,
    size: u32,
}

impl DebugSection {
    
    pub fn new(name: &str) -> DebugSection {
        DebugSection {
            entries: Vec::new(),
            name: name.to_owned(),
            size: 0
        }
    }

    pub fn read(reader: &mut KOFileReader, header: &SectionHeader) -> Result<DebugSection, Box<dyn Error>> {
        let mut debug_section = DebugSection::new(header.name());

        while debug_section.size() < header.section_size() {
            debug_section.add(DebugEntry::read(reader)?);
        }

        if debug_section.size() > header.section_size() {
            return Err("Error reading debug information section, size mismatch".into());
        }

        Ok(debug_section)
    }

    pub fn write(&self, writer: &mut KOFileWriter) -> Result<(), Box<dyn Error>> {

        for entry in self.entries.iter() {
            entry.write(writer)?;
        }

        Ok(())
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn add(&mut self, entry: DebugEntry) -> usize {
        self.entries.push(entry);

        self.size += self.entries.last().unwrap().size();

        self.entries.len() - 1
    }

    pub fn get(&self, index: usize) -> Result<&DebugEntry, Box<dyn Error>> {
        match self.entries.get(index) {
            Some(e) => Ok(e),
            None => Err(format!("Could not find debug entry at index {} in debug information section.", index).into()),
        } 
    }

    pub fn get_entries(&self) -> &Vec<DebugEntry> {
        &self.entries
    }

}

pub struct DebugEntry {
    line: u16,
    num_ranges: u8,
    ranges: Vec<(u32, u32)>
}

impl DebugEntry {

    pub fn new(line: u16, ranges: Vec<(u32, u32)>) -> DebugEntry {
        DebugEntry {
            line,
            num_ranges: ranges.len() as u8,
            ranges
        }
    }

    pub fn read(reader: &mut KOFileReader) -> Result<DebugEntry, Box<dyn Error>> {

        let line = reader.read_uint16()?;

        let num_ranges = reader.next()?;

        let mut ranges = Vec::with_capacity(num_ranges as usize);

        for _ in 0..num_ranges {
            let range_start = reader.read_uint32()?;
            let range_stop = reader.read_uint32()?;

            ranges.push( (range_start, range_stop) );
        }

        Ok(DebugEntry::new(line, ranges))

    }

    pub fn write(&self, writer: &mut KOFileWriter) -> Result<(), Box<dyn Error>> {

        writer.write_uint16(self.line)?;

        writer.write(self.num_ranges)?;

        for (range_start, range_stop) in self.ranges.iter() {
            writer.write_uint32(*range_start)?;
            writer.write_uint32(*range_stop)?;
        }

        Ok(())
    }

    pub fn size(&self) -> u32 {
        3 + self.num_ranges as u32 * 8
    }

}

#[derive(Debug, Clone, PartialEq)]
pub enum KOSValue {
    NULL,
    BOOL(bool),
    BYTE(i8),
    INT16(i16),
    INT32(i32),
    FLOAT(f32),
    DOUBLE(f64),
    STRING(String),
    ARGMARKER,
    SCALARINT(i32),
    SCALARDOUBLE(f64),
    BOOLEANVALUE(bool),
    STRINGVALUE(String),
}

impl Hash for KOSValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let s = format!("{:?}", self);
        s.hash(state);
    }
}

impl KOSValue {

    pub fn read(reader: &mut KOFileReader) -> Result<KOSValue, Box<dyn Error>> {
        let value_type: usize = reader.next()? as usize;

        Ok(match value_type {
            0 => KOSValue::NULL,
            1 => KOSValue::BOOL(reader.read_boolean()?),
            2 => KOSValue::BYTE(reader.read_byte()?),
            3 => KOSValue::INT16(reader.read_int16()?),
            4 => KOSValue::INT32(reader.read_int32()?),
            5 => KOSValue::FLOAT(reader.read_float()?),
            6 => KOSValue::DOUBLE(reader.read_double()?),
            7 => KOSValue::STRING(reader.read_kos_string()?),
            8 => KOSValue::ARGMARKER,
            9 => KOSValue::SCALARINT(reader.read_int32()?),
            10 => KOSValue::SCALARDOUBLE(reader.read_double()?),
            11 => KOSValue::BOOLEANVALUE(reader.read_boolean()?),
            12 => KOSValue::STRINGVALUE(reader.read_kos_string()?),
            _ => return Err(format!("Unknown kOS value type encountered: {:x}", value_type).into()),
        })
    }

    pub fn write(&self, writer: &mut KOFileWriter) -> Result<(), Box<dyn Error>> {

        match self {
            KOSValue::NULL => writer.write(0)?,
            KOSValue::BOOL(b) => {
                writer.write(1)?;
                writer.write_boolean(*b)?;
            },
            KOSValue::BYTE(b) => {
                writer.write(2)?;
                writer.write_byte(*b)?;
            },
            KOSValue::INT16(i) => {
                writer.write(3)?;
                writer.write_int16(*i)?;
            },
            KOSValue::INT32(i) => {
                writer.write(4)?;
                writer.write_int32(*i)?;
            },
            KOSValue::FLOAT(f) => {
                writer.write(5)?;
                writer.write_float(*f)?;
            },
            KOSValue::DOUBLE(d) => {
                writer.write(6)?;
                writer.write_double(*d)?;
            },
            KOSValue::STRING(s) => {
                writer.write(7)?;
                writer.write_kos_string(s)?;
            },
            KOSValue::ARGMARKER => writer.write(8)?,
            KOSValue::SCALARINT(i) => {
                writer.write(9)?;
                writer.write_int32(*i)?;
            },
            KOSValue::SCALARDOUBLE(d) => {
                writer.write(10)?;
                writer.write_double(*d)?;
            },
            KOSValue::BOOLEANVALUE(b) => {
                writer.write(11)?;
                writer.write_boolean(*b)?;
            },
            KOSValue::STRINGVALUE(s) => {
                writer.write(12)?;
                writer.write_kos_string(s)?;
            }
        }

        Ok(())
    }

    pub fn size(&self) -> u32 {
        match self {
            KOSValue::NULL => 1,
            KOSValue::BOOL(_) => 2,
            KOSValue::BYTE(_) => 2,
            KOSValue::INT16(_) => 3,
            KOSValue::INT32(_) => 5,
            KOSValue::FLOAT(_) => 5,
            KOSValue::DOUBLE(_) => 9,
            KOSValue::STRING(s) => s.len() as u32 + 2,
            KOSValue::ARGMARKER => 1,
            KOSValue::SCALARINT(_) => 5,
            KOSValue::SCALARDOUBLE(_) => 9,
            KOSValue::BOOLEANVALUE(_) => 2,
            KOSValue::STRINGVALUE(s) => s.len() as u32 + 2
        }
    }

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