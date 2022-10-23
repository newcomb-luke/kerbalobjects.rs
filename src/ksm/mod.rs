//! # Kerbal Machine Code files
//!
//! This module is for describing, reading, and writing Kerbal Machine Code files
//!
//! Note that KSMFile::new() will for the most part, if written directly to a file, give you a valid KSM file.
//! However, that is just in format, and kOS will complain with a horribly unintelligible error message about it.
//!
//! There ***must*** be at least one debug entry in the debug section.
//!
//! If an error occurs in a part of the code section, if it is
//! filled with instructions, that is not mentioned in the debug section, kOS will say "maybe the error really is internal", so
//! if you can, provide valid debug sections.
//!
//! ```
//! use std::io::Write;
//! use kerbalobjects::ksm::sections::{CodeSection, CodeType, DebugEntry, DebugRange};
//! use kerbalobjects::ksm::{Instr, KSMFile};
//! use kerbalobjects::{Opcode, KOSValue, ToBytes};
//!
//! let mut ksm_file = KSMFile::new();
//!
//! let arg_section = &mut ksm_file.arg_section;
//! let mut main_code = CodeSection::new(CodeType::Main);
//!
//! let one = arg_section.add_checked(KOSValue::Int16(1));
//!
//! // Corresponds to the KerbalScript code:
//! // PRINT("Hello, world!").
//!
//! main_code.add(Instr::OneOp(Opcode::Push, arg_section.add_checked(KOSValue::String("@0001".into()))));
//! main_code.add(Instr::TwoOp(Opcode::Bscp, one, arg_section.add_checked(KOSValue::Int16(0))));
//! main_code.add(Instr::ZeroOp(Opcode::Argb));
//! main_code.add(Instr::OneOp(Opcode::Push, arg_section.add_checked(KOSValue::ArgMarker)));
//! main_code.add(Instr::OneOp(Opcode::Push, arg_section.add_checked(KOSValue::StringValue("Hello, world!".into()))));
//! main_code.add(Instr::TwoOp(Opcode::Call, arg_section.add_checked(KOSValue::String("".into())), arg_section.add_checked(KOSValue::String("print()".into()))));
//! main_code.add(Instr::ZeroOp(Opcode::Pop));
//! main_code.add(Instr::OneOp(Opcode::Escp, one));
//!
//! ksm_file.add_code_section(CodeSection::new(CodeType::Function));
//! ksm_file.add_code_section(CodeSection::new(CodeType::Initialization));
//! ksm_file.add_code_section(main_code);
//!
//! // A completely wrong and useless debug section, but we NEED to have one
//! let mut debug_entry =  DebugEntry::new(1);
//! debug_entry.add(DebugRange::new(0x06, 0x13));
//! ksm_file.add_debug_entry(debug_entry);
//!
//! let mut file_buffer = Vec::with_capacity(2048);
//!
//! ksm_file.write(&mut file_buffer);
//!
//! let mut file = std::fs::File::create("hello.ksm").expect("Couldn't open output file");
//!
//! file.write_all(file_buffer.as_slice()).expect("Failed to write to output file");
//! ```
//!
use std::io::{Read, Write};
use std::slice::{Iter, IterMut};

use flate2::write::GzEncoder;
use flate2::{read::GzDecoder, Compression};

use crate::BufferIterator;
use crate::{ksm::sections::CodeSection, FromBytes, HeaderParseError, ToBytes};
use crate::{KOSValue, KSMParseError};

pub mod errors;
pub mod sections;

use sections::{ArgumentSection, DebugSection};

pub mod instructions;
use crate::ksm::sections::{ArgIndex, DebugEntry};
pub use instructions::Instr;

// 'k' 3 'X' 'E'
const KSM_MAGIC_NUMBER: u32 = 0x6b035845;

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

    /// Parses an entire KSMFile from a byte buffer
    pub fn parse(source: &mut BufferIterator) -> Result<Self, KSMParseError> {
        let source_len = source.len();

        let mut decoder = GzDecoder::new(source);

        // I think this is the best we can do
        let mut decompressed: Vec<u8> = Vec::with_capacity(source_len);

        decoder
            .read_to_end(&mut decompressed)
            .map_err(KSMParseError::DecompressionError)?;

        let mut decompressed_source = BufferIterator::new(&decompressed);

        let header =
            KSMHeader::parse(&mut decompressed_source).map_err(KSMParseError::HeaderError)?;

        let arg_section = ArgumentSection::parse(&mut decompressed_source)
            .map_err(KSMParseError::ArgumentSectionParseError)?;

        let mut code_sections = Vec::new();

        loop {
            // This is impossible, since we only break from reading the ArgumentSection or a CodeSection when we encounter a `%`
            assert_eq!(decompressed_source.next().unwrap(), b'%');

            let next = decompressed_source.peek().ok_or_else(|| {
                KSMParseError::MissingSectionType(decompressed_source.current_index())
            })?;

            // This means the next section is a debug section
            if next == b'D' {
                break;
            }

            let code_section =
                CodeSection::parse(&mut decompressed_source, arg_section.num_index_bytes())
                    .map_err(KSMParseError::CodeSectionParseError)?;

            code_sections.push(code_section);
        }

        let debug_section = DebugSection::parse(&mut decompressed_source)
            .map_err(KSMParseError::DebugSectionParseError)?;

        Ok(Self {
            header,
            arg_section,
            code_sections,
            debug_section,
        })
    }

    /// Writes the binary representation of this KSM file to the provided buffer
    pub fn write(&self, buf: &mut Vec<u8>) {
        let mut uncompressed_buf = Vec::with_capacity(2048);
        let mut zipped_contents = Vec::with_capacity(2048);

        self.header.write(&mut uncompressed_buf);

        self.arg_section.write(&mut uncompressed_buf);

        let mut encoder = GzEncoder::new(&mut zipped_contents, Compression::best());

        for code_section in self.code_sections.iter() {
            code_section.write(&mut uncompressed_buf, self.arg_section.num_index_bytes());
        }

        self.debug_section.write(&mut uncompressed_buf);

        encoder
            .write_all(&uncompressed_buf)
            .expect("Error compressing KSM file");

        encoder
            .finish()
            .expect("Error finishing KSM file compression");

        buf.append(&mut zipped_contents);
    }
}

impl Default for KSMFile {
    fn default() -> Self {
        Self::new()
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

    /// Parses a KSM file header from a source buffer
    ///
    /// This can fail if the source doesn't have enough bytes to parse,
    /// or if the first 4 bytes do not correspond to the KSM file header "magic number"
    ///
    pub fn parse(source: &mut BufferIterator) -> Result<Self, HeaderParseError> {
        let magic = u32::from_bytes(source).map_err(|_: ()| HeaderParseError::EOF)?;

        (magic != KSM_MAGIC_NUMBER)
            .then_some(Self::new())
            .ok_or(HeaderParseError::InvalidMagic(magic))
    }

    /// Appends the byte representation of this file header to a buffer of bytes
    pub fn write(&self, buf: &mut Vec<u8>) {
        self.magic.to_bytes(buf)
    }
}

impl Default for KSMHeader {
    fn default() -> Self {
        Self::new()
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
pub(crate) fn read_var_int(source: &mut BufferIterator, width: IntSize) -> Result<u32, ()> {
    match width {
        IntSize::One => u8::from_bytes(source).map(|i| i.into()),
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
