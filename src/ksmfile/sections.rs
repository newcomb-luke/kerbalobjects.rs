use std::collections::HashMap;
use std::iter::Peekable;
use std::slice::Iter;

use crate::{FromBytes, KOSValue, ReadError, ReadResult, ToBytes};

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

        self.arguments.push(argument);
        self.kos_index_map.insert(self.size_bytes, index);
        self.size_bytes += size;

        index
    }

    pub fn get(&self, index: usize) -> Option<&KOSValue> {
        let vec_index = *self.kos_index_map.get(&index)?;

        self.arguments.get(vec_index)
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
        println!("Reading ArgumentSection");

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

        println!("\tNumber of index bytes: {}", num_index_bytes);

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

                    println!("\tRead argument: {:?}", argument);

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

pub struct CodeSection {}
