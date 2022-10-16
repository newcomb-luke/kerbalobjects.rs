//! The module for describing, reading, and writing Kerbal Machine Code files
use std::io::{Read, Write};
use std::slice::Iter;

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
pub use instructions::Instr;

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
    pub fn add_argument(&mut self, argument: KOSValue) {
        self.arg_section.add(argument);
    }

    /// Returns an iterator over all of the code sections in this file
    pub fn code_sections(&self) -> Iter<CodeSection> {
        self.code_sections.iter()
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
