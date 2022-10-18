//! The module for describing, reading, and writing Kerbal Machine Code files
use std::io::{Read, Write};
use std::slice::{Iter, IterMut};

use flate2::write::GzEncoder;
use flate2::{read::GzDecoder, Compression};

use crate::{
    errors::{ReadError, ReadResult},
    ksm::sections::CodeSection,
};
use crate::{FileIterator, FromBytes};
use crate::{KOSValue, ToBytes};

pub mod errors;
pub mod sections;

use sections::{ArgumentSection, DebugSection};

pub mod instructions;
use crate::ksm::sections::{ArgIndex, DebugEntry};
pub use instructions::Instr;

// 'k' 3 'X' 'E'
const KSM_MAGIC_NUMBER: u32 = 0x4558036b;

/// An in-memory representation of a KSM file
///
/// Can be modified, written, or read.
#[derive(Debug)]
pub struct KSMFile {
    /// This file's header
    pub header: KSMHeader,
    /// This file's argument section
    pub arg_section: ArgumentSection,
    /// This file's debug section
    pub debug_section: DebugSection,
    code_sections: Vec<CodeSection>,
}

impl KSMFile {
    /// Creates a new KSM file that is empty except for the required argument section, and debug section
    pub fn new() -> Self {
        Self {
            header: KSMHeader::new(),
            arg_section: ArgumentSection::new(),
            code_sections: Vec::new(),
            debug_section: DebugSection::new(),
        }
    }

    /// Creates a new KSM file using the provided argument section, code sections, and debug section
    pub fn new_from_parts(
        arg_section: ArgumentSection,
        code_sections: Vec<CodeSection>,
        debug_section: DebugSection,
    ) -> Self {
        Self {
            header: KSMHeader::new(),
            arg_section,
            code_sections,
            debug_section,
        }
    }

    /// A convenience function to add a value to this file's argument section
    pub fn add_argument(&mut self, argument: KOSValue) -> ArgIndex {
        self.arg_section.add(argument)
    }

    /// A convenience function to add a debug entry to this file's debug section
    pub fn add_debug_entry(&mut self, entry: DebugEntry) {
        self.debug_section.add(entry)
    }

    /// Returns an iterator over all of the code sections in this file
    pub fn code_sections(&self) -> Iter<CodeSection> {
        self.code_sections.iter()
    }

    /// Returns a mutable iterator over all of the code sections in this file
    pub fn code_sections_mut(&mut self) -> IterMut<CodeSection> {
        self.code_sections.iter_mut()
    }

    /// Adds a new code section to this file
    pub fn add_code_section(&mut self, code_section: CodeSection) {
        self.code_sections.push(code_section);
    }
}

impl Default for KSMFile {
    fn default() -> Self {
        Self::new()
    }
}

impl ToBytes for KSMFile {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        let mut uncompressed_buf = Vec::with_capacity(2048);
        let mut zipped_contents = Vec::with_capacity(2048);

        self.header.to_bytes(&mut uncompressed_buf);

        self.arg_section.to_bytes(&mut uncompressed_buf);

        let mut encoder = GzEncoder::new(&mut zipped_contents, Compression::best());

        for code_section in self.code_sections.iter() {
            code_section.to_bytes(&mut uncompressed_buf, self.arg_section.num_index_bytes());
        }

        self.debug_section.to_bytes(&mut uncompressed_buf);

        encoder
            .write_all(&uncompressed_buf)
            .expect("Error compressing KSM file");

        encoder
            .finish()
            .expect("Error finishing KSM file compression");

        buf.append(&mut zipped_contents);
    }
}

impl FromBytes for KSMFile {
    fn from_bytes(source: &mut FileIterator) -> ReadResult<Self>
    where
        Self: Sized,
    {
        let mut decoder = GzDecoder::new(source);

        let mut decompressed: Vec<u8> = Vec::with_capacity(2048);

        decoder
            .read_to_end(&mut decompressed)
            .map_err(ReadError::KSMDecompressionError)?;

        let mut decompressed_source = FileIterator::new(&decompressed);

        let header = KSMHeader::from_bytes(&mut decompressed_source)?;

        let arg_section = ArgumentSection::from_bytes(&mut decompressed_source)?;

        let mut code_sections = Vec::new();

        loop {
            let delimiter = u8::from_bytes(&mut decompressed_source)
                .map_err(|_| ReadError::MissingCodeSectionError)?;

            if delimiter != b'%' {
                return Err(ReadError::ExpectedCodeSectionError(delimiter));
            }

            if let Some(next) = decompressed_source.peek() {
                if next == b'D' {
                    break;
                }

                let code_section = CodeSection::from_bytes(
                    &mut decompressed_source,
                    arg_section.num_index_bytes(),
                )?;

                code_sections.push(code_section);
            } else {
                return Err(ReadError::MissingCodeSectionError);
            }
        }

        let debug_section = DebugSection::from_bytes(&mut decompressed_source)?;

        let ksm = KSMFile {
            header,
            arg_section,
            code_sections,
            debug_section,
        };

        Ok(ksm)
    }
}

/// A KSM file header.
///
/// The spec only requires it to contain the file's magic to identify it as a KSM file.
/// So currently this type isn't extremely useful.
#[derive(Debug)]
pub struct KSMHeader {
    magic: u32,
}

impl KSMHeader {
    /// Creates a new KSM file header
    pub fn new() -> Self {
        KSMHeader {
            magic: KSM_MAGIC_NUMBER,
        }
    }
}

impl Default for KSMHeader {
    fn default() -> Self {
        Self::new()
    }
}

impl ToBytes for KSMHeader {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        self.magic.to_bytes(buf);
    }
}

impl FromBytes for KSMHeader {
    fn from_bytes(source: &mut FileIterator) -> ReadResult<Self>
    where
        Self: Sized,
    {
        let magic =
            u32::from_bytes(source).map_err(|_| ReadError::KSMHeaderReadError("file magic"))?;

        if magic != KSM_MAGIC_NUMBER {
            return Err(ReadError::InvalidKSMFileMagicError);
        }

        Ok(KSMHeader::new())
    }
}

/// Describes the number of bytes that are required to store a given integer.
///
/// This is used to keep track of the number of bytes required to index into
/// the KSM argument section, as well as the number of bytes required to represent
/// a range in the debug section of a KSM file.
///
/// This provides an advantage over a raw integer type, because these
/// values are the only ones currently supported by kOS and are discrete.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[repr(u8)]
pub enum IntSize {
    /// 1
    One = 1,
    /// 2
    Two = 2,
    /// 3
    Three = 3,
    /// 4
    Four = 4,
}

impl TryFrom<u8> for IntSize {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::One),
            2 => Ok(Self::Two),
            3 => Ok(Self::Three),
            4 => Ok(Self::Four),
            _ => Err(value),
        }
    }
}

impl From<IntSize> for u8 {
    fn from(num: IntSize) -> Self {
        num as u8
    }
}

// An internal function for reading an integer with a variable number of bytes.
pub(crate) fn read_var_int(source: &mut FileIterator, width: IntSize) -> Result<u32, ()> {
    match width {
        IntSize::One => u8::from_bytes(source).map(|i| i.into()).map_err(|_| ()),
        IntSize::Two => {
            let mut slice = [0u8; 2];
            for b in &mut slice {
                *b = source.next().ok_or(())?;
            }
            Ok(u16::from_be_bytes(slice).into())
        }
        IntSize::Three => {
            let mut slice = [0u8; 4];
            for b in &mut slice[1..4] {
                *b = source.next().ok_or(())?;
            }

            Ok(u32::from_be_bytes(slice))
        }
        IntSize::Four => {
            let mut slice = [0u8; 4];
            for b in &mut slice {
                *b = source.next().ok_or(())?;
            }
            Ok(u32::from_be_bytes(slice))
        }
    }
}

// An internal function for writing an integer with a variable number of bytes.
pub(crate) fn write_var_int(value: u32, buf: &mut Vec<u8>, width: IntSize) {
    match width {
        IntSize::One => {
            (value as u8).to_bytes(buf);
        }
        IntSize::Two => {
            buf.extend_from_slice(&(value as u16).to_be_bytes());
        }
        IntSize::Three => {
            let slice = &value.to_be_bytes();
            buf.extend_from_slice(&slice[1..4]);
        }
        IntSize::Four => {
            buf.extend_from_slice(&value.to_be_bytes());
        }
    }
}

// An internal function for finding out the fewest number of bytes required to hold a value,
// used for determining the size of var ints
pub(crate) fn fewest_bytes_to_hold(value: u32) -> IntSize {
    if value <= u8::MAX as u32 {
        IntSize::One
    } else if value <= u16::MAX as u32 {
        IntSize::Two
    } else if value <= 1677215u32 {
        // 1677215 is the largest value that can be stored in 24 unsigned bits
        IntSize::Three
    } else {
        IntSize::Four
    }
}
