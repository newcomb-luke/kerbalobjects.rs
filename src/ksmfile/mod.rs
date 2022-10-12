use std::{
    io::{Read, Write},
    iter::Peekable,
    slice::Iter,
};

use flate2::write::GzEncoder;
use flate2::{read::GzDecoder, Compression};

use crate::FromBytes;
use crate::ToBytes;
use crate::{
    errors::{ReadError, ReadResult},
    ksmfile::sections::CodeSection,
};

pub mod errors;
pub mod sections;

use sections::{ArgumentSection, DebugSection};

pub mod instructions;
pub use instructions::Instr;

const KSM_MAGIC_NUMBER: u32 = 0x4558036b;

pub struct KSMFile {
    header: KSMHeader,
    arg_section: ArgumentSection,
    code_sections: Vec<CodeSection>,
    debug_section: DebugSection,
}

impl KSMFile {
    pub fn new() -> Self {
        KSMFile {
            header: KSMHeader::new(),
            arg_section: ArgumentSection::new(),
            code_sections: Vec::new(),
            debug_section: DebugSection::new(1),
        }
    }

    pub fn arg_section(&self) -> &ArgumentSection {
        &self.arg_section
    }

    pub fn arg_section_mut(&mut self) -> &mut ArgumentSection {
        &mut self.arg_section
    }

    pub fn code_sections(&self) -> Iter<CodeSection> {
        self.code_sections.iter()
    }

    pub fn add_code_section(&mut self, code_section: CodeSection) {
        self.code_sections.push(code_section);
    }

    pub fn debug_section(&self) -> &DebugSection {
        &self.debug_section
    }

    pub fn debug_section_mut(&mut self) -> &mut DebugSection {
        &mut self.debug_section
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
    fn from_bytes(source: &mut Peekable<Iter<u8>>) -> ReadResult<Self>
    where
        Self: Sized,
    {
        let zipped_contents: Vec<u8> = source.map(|b| *b).collect();
        let mut decoder = GzDecoder::new(zipped_contents.as_slice());

        let mut decompressed: Vec<u8> = Vec::with_capacity(2048);

        decoder
            .read_to_end(&mut decompressed)
            .map_err(|e| ReadError::KSMDecompressionError(e))?;

        let mut decompressed_source = decompressed.iter().peekable();

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
                if **next == b'D' {
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

pub struct KSMHeader {
    magic: u32,
}

impl KSMHeader {
    pub fn new() -> Self {
        KSMHeader {
            magic: KSM_MAGIC_NUMBER,
        }
    }
}

impl ToBytes for KSMHeader {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        self.magic.to_bytes(buf);
    }
}

impl FromBytes for KSMHeader {
    fn from_bytes(source: &mut Peekable<Iter<u8>>) -> ReadResult<Self>
    where
        Self: Sized,
    {
        let magic = u32::from_bytes(source)
            .map_err(|_| ReadError::KSMHeaderReadError("file magic"))?;

        if magic != KSM_MAGIC_NUMBER {
            return Err(ReadError::InvalidKSMFileMagicError);
        }

        Ok(KSMHeader::new())
    }
}
