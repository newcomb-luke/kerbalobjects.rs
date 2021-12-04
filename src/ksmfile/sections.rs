use std::collections::HashMap;
use std::iter::Peekable;
use std::slice::Iter;

use crate::{FromBytes, KOSValue, ReadError, ReadResult, ToBytes};

use super::Instr;

#[derive(Debug)]
pub struct ArgumentSection {
    num_index_bytes: usize,
    arguments: Vec<KOSValue>,
    kos_index_map: HashMap<usize, usize>,
    size_bytes: usize,
}

impl ArgumentSection {
    pub fn new() -> Self {
        ArgumentSection {
            num_index_bytes: 1,
            arguments: Vec::with_capacity(16),
            kos_index_map: HashMap::with_capacity(16),
            size_bytes: 3,
        }
    }

    pub fn num_index_bytes(&self) -> usize {
        self.num_index_bytes
    }

    pub fn add(&mut self, argument: KOSValue) -> usize {
        let size = argument.size_bytes();
        let index = self.arguments.len();
        let kos_index = self.size_bytes;

        self.arguments.push(argument);
        self.kos_index_map.insert(self.size_bytes, index);
        self.size_bytes += size;

        kos_index
    }

    pub fn get(&self, index: usize) -> Option<&KOSValue> {
        let vec_index = *self.kos_index_map.get(&index)?;

        self.arguments.get(vec_index)
    }

    pub fn arguments(&self) -> Iter<KOSValue> {
        self.arguments.iter()
    }

    pub fn size_bytes(&self) -> usize {
        self.size_bytes
    }

    pub fn recalculate_index_bytes(&mut self) {
        self.num_index_bytes = if self.size_bytes <= 256 {
            1
        } else if self.size_bytes <= 65535 {
            2
        } else if self.size_bytes <= 1677215 {
            3
        } else {
            4
        };
    }
}

impl ToBytes for ArgumentSection {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        b'%'.to_bytes(buf);
        b'A'.to_bytes(buf);
        (self.num_index_bytes as u8).to_bytes(buf);

        for argument in self.arguments.iter() {
            argument.to_bytes(buf);
        }
    }
}

impl FromBytes for ArgumentSection {
    fn from_bytes(source: &mut Peekable<Iter<u8>>, debug: bool) -> ReadResult<Self>
    where
        Self: Sized,
    {
        if debug {
            println!("Reading ArgumentSection");
        }

        let header =
            u16::from_bytes(source, debug).map_err(|_| ReadError::MissingArgumentSectionError)?;

        // %A in hex, little-endian
        if header != 0x4125 {
            return Err(ReadError::ExpectedArgumentSectionError(header));
        }

        let num_index_bytes =
            u8::from_bytes(source, debug).map_err(|_| ReadError::NumIndexBytesReadError)? as usize;

        if num_index_bytes < 1 || num_index_bytes > 4 {
            return Err(ReadError::InvalidNumIndexBytesError(num_index_bytes));
        }

        if debug {
            println!("\tNumber of index bytes: {}", num_index_bytes);
        }

        let mut arg_section = ArgumentSection {
            num_index_bytes,
            arguments: Vec::new(),
            kos_index_map: HashMap::new(),
            size_bytes: 3,
        };

        loop {
            if let Some(next) = source.peek() {
                if **next == b'%' {
                    break;
                } else {
                    let argument = KOSValue::from_bytes(source, debug)?;

                    if debug {
                        println!("\tRead argument: {:?}", argument);
                    }

                    arg_section.add(argument);
                }
            } else {
                return Err(ReadError::ArgumentSectionReadError);
            }
        }

        Ok(arg_section)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CodeType {
    Function,
    Initialization,
    Main,
    Unknown,
}

impl From<u8> for CodeType {
    fn from(byte: u8) -> Self {
        match byte {
            b'F' => CodeType::Function,
            b'I' => CodeType::Initialization,
            b'M' => CodeType::Main,
            _ => CodeType::Unknown,
        }
    }
}

impl From<CodeType> for u8 {
    fn from(c: CodeType) -> Self {
        match c {
            CodeType::Function => b'F',
            CodeType::Initialization => b'I',
            CodeType::Main => b'M',
            CodeType::Unknown => b'U',
        }
    }
}

impl ToBytes for CodeType {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        buf.push((*self).into());
    }
}

impl FromBytes for CodeType {
    fn from_bytes(source: &mut Peekable<Iter<u8>>, _debug: bool) -> ReadResult<Self>
    where
        Self: Sized,
    {
        let value = *source.next().ok_or(ReadError::CodeTypeReadError)?;
        let code_type = CodeType::from(value);

        match code_type {
            CodeType::Unknown => Err(ReadError::UnknownCodeTypeReadError(value as char)),
            _ => Ok(code_type),
        }
    }
}

#[derive(Debug)]
pub struct CodeSection {
    section_type: CodeType,
    instructions: Vec<Instr>,
}

impl CodeSection {
    pub fn new(section_type: CodeType) -> Self {
        CodeSection {
            section_type,
            instructions: Vec::new(),
        }
    }

    pub fn instructions(&self) -> Iter<Instr> {
        self.instructions.iter()
    }

    pub fn add(&mut self, instr: Instr) {
        self.instructions.push(instr);
    }

    pub fn section_type(&self) -> CodeType {
        self.section_type
    }

    pub fn to_bytes(&self, buf: &mut Vec<u8>, num_index_bytes: usize) {
        buf.push(b'%');
        self.section_type.to_bytes(buf);

        for instr in self.instructions.iter() {
            instr.to_bytes(buf, num_index_bytes);
        }
    }

    pub fn from_bytes(
        source: &mut Peekable<Iter<u8>>,
        debug: bool,
        num_index_bytes: usize,
    ) -> ReadResult<Self> {
        if debug {
            print!("Reading code section, ");
        }

        let section_type = CodeType::from_bytes(source, debug)?;

        if debug {
            println!("{:?}", section_type);
        }

        let mut instructions = Vec::new();

        loop {
            if let Some(next) = source.peek() {
                if **next == b'%' {
                    break;
                }

                let instr = Instr::from_bytes(source, debug, num_index_bytes)?;

                if debug {
                    println!("\tRead instruction {:?}", instr);
                }

                instructions.push(instr);
            } else {
                return Err(ReadError::CodeSectionReadError);
            }
        }

        Ok(CodeSection {
            section_type,
            instructions,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DebugRange {
    start: usize,
    end: usize,
}

impl DebugRange {
    pub fn new(start: usize, end: usize) -> Self {
        DebugRange { start, end }
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }

    pub fn to_bytes(&self, buf: &mut Vec<u8>, range_size: usize) {
        DebugRange::write_variable(buf, self.start, range_size);
        DebugRange::write_variable(buf, self.end, range_size);
    }

    fn write_variable(buf: &mut Vec<u8>, data: usize, size: usize) {
        match size {
            1 => {
                (data as u8).to_bytes(buf);
            }
            2 => {
                (data as u16).to_bytes(buf);
            }
            3 => {
                let converted = data as u32;
                let upper = (data >> 16) as u8;
                let lower = (converted & 0x0000ffff) as u16;

                upper.to_bytes(buf);
                lower.to_bytes(buf);
            }
            4 => {
                (data as u32).to_bytes(buf);
            }
            _ => unreachable!(),
        }
    }

    pub fn from_bytes(
        source: &mut Peekable<Iter<u8>>,
        debug: bool,
        range_size: usize,
    ) -> ReadResult<Self> {
        let start = DebugRange::read_variable(source, debug, range_size)?;
        let end = DebugRange::read_variable(source, debug, range_size)?;

        Ok(DebugRange { start, end })
    }

    fn read_variable(
        source: &mut Peekable<Iter<u8>>,
        debug: bool,
        range_size: usize,
    ) -> ReadResult<usize> {
        Ok(match range_size {
            1 => {
                (u8::from_bytes(source, debug).map_err(|_| ReadError::DebugRangeReadError))?
                    as usize
            }
            2 => {
                (u16::from_bytes(source, debug).map_err(|_| ReadError::DebugRangeReadError))?
                    as usize
            }
            3 => {
                let mut upper = u8::from_bytes(source, debug)
                    .map_err(|_| ReadError::DebugRangeReadError)?
                    as u32;
                upper = upper << 24;
                let lower = u16::from_bytes(source, debug)
                    .map_err(|_| ReadError::DebugRangeReadError)?
                    as u32;

                (upper | lower) as usize
            }
            4 => {
                (u32::from_bytes(source, debug).map_err(|_| ReadError::DebugRangeReadError))?
                    as usize
            }
            _ => unreachable!(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct DebugEntry {
    line_number: isize,
    number_ranges: usize,
    ranges: Vec<DebugRange>,
}

impl DebugEntry {
    pub fn new(line_number: isize) -> Self {
        DebugEntry {
            line_number,
            number_ranges: 0,
            ranges: Vec::new(),
        }
    }

    pub fn add(&mut self, range: DebugRange) {
        self.ranges.push(range);
        self.number_ranges += 1;
    }

    pub fn line_number(&self) -> isize {
        self.line_number
    }

    pub fn number_ranges(&self) -> usize {
        self.number_ranges
    }

    pub fn ranges(&self) -> Iter<DebugRange> {
        self.ranges.iter()
    }

    pub fn get(&self, index: usize) -> Option<&DebugRange> {
        self.ranges.get(index)
    }

    pub fn to_bytes(&self, buf: &mut Vec<u8>, range_size: usize) {
        (self.line_number as i16).to_bytes(buf);
        (self.number_ranges as u8).to_bytes(buf);

        for range in self.ranges.iter() {
            range.to_bytes(buf, range_size);
        }
    }

    pub fn from_bytes(
        source: &mut Peekable<Iter<u8>>,
        debug: bool,
        range_size: usize,
    ) -> ReadResult<Self> {
        let line_number =
            i16::from_bytes(source, debug).map_err(|_| ReadError::DebugEntryReadError)? as isize;
        let number_ranges =
            u8::from_bytes(source, debug).map_err(|_| ReadError::DebugEntryReadError)? as usize;
        let mut ranges = Vec::new();

        for _ in 0..number_ranges {
            let range = DebugRange::from_bytes(source, debug, range_size)?;

            ranges.push(range);
        }

        Ok(DebugEntry {
            line_number,
            number_ranges,
            ranges,
        })
    }
}

#[derive(Debug)]
pub struct DebugSection {
    range_size: usize,
    debug_entries: Vec<DebugEntry>,
}

impl DebugSection {
    pub fn new(range_size: usize) -> Self {
        DebugSection {
            range_size,
            debug_entries: Vec::new(),
        }
    }

    pub fn add(&mut self, entry: DebugEntry) {
        self.debug_entries.push(entry);
    }

    pub fn debug_entries(&self) -> Iter<DebugEntry> {
        self.debug_entries.iter()
    }

    pub fn range_size(&self) -> usize {
        self.range_size
    }

    pub fn set_range_size(&mut self, range_size: usize) {
        self.range_size = range_size
    }
}

impl ToBytes for DebugSection {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        b'%'.to_bytes(buf);
        b'D'.to_bytes(buf);

        (self.range_size as u8).to_bytes(buf);

        for entry in self.debug_entries.iter() {
            entry.to_bytes(buf, self.range_size);
        }
    }
}

impl FromBytes for DebugSection {
    fn from_bytes(source: &mut Peekable<Iter<u8>>, debug: bool) -> ReadResult<Self>
    where
        Self: Sized,
    {
        if let Some(next) = source.next() {
            if *next != b'D' {
                Err(ReadError::ExpectedDebugSectionError(*next))
            } else {
                let range_size = u8::from_bytes(source, debug)
                    .map_err(|_| ReadError::DebugSectionReadError)?
                    as usize;
                let mut debug_entries = Vec::new();

                while let Some(_) = source.peek() {
                    let debug_entry = DebugEntry::from_bytes(source, debug, range_size)?;

                    debug_entries.push(debug_entry);
                }

                Ok(DebugSection {
                    range_size,
                    debug_entries,
                })
            }
        } else {
            Err(ReadError::MissingDebugSectionError)
        }
    }
}
