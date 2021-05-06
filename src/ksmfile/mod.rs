use crate::errors::{ReadError, ReadResult};
use crate::FromBytes;
use crate::ToBytes;
use flate2::write::GzEncoder;
use flate2::{read::GzDecoder, Compression};
use std::{
    io::{Read, Write},
    iter::Peekable,
    slice::Iter,
};

pub mod instructions;

const KSM_MAGIC_NUMBER: u32 = 0x4558036b;

pub struct KSMFile {
    header: KSMHeader,
}

impl KSMFile {
    pub fn new() -> Self {
        KSMFile {
            header: KSMHeader::new(),
        }
    }
}

impl ToBytes for KSMFile {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        let mut uncompressed_buf = Vec::with_capacity(2048);
        let mut zipped_contents = Vec::with_capacity(2048);

        self.header.to_bytes(&mut uncompressed_buf);

        let mut encoder = GzEncoder::new(&mut zipped_contents, Compression::best());

        encoder
            .write_all(&uncompressed_buf)
            .expect("Error compressing KSM file");

        encoder
            .finish()
            .expect("Error finishing KSM file compression");

        buf.append(&mut zipped_contents);

        unimplemented!();
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

        let ksm = KSMFile { header };

        unimplemented!();
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
        let magic =
            u32::from_bytes(source).map_err(|_| ReadError::KSMHeaderReadError("file magic"))?;

        if magic != KSM_MAGIC_NUMBER {
            return Err(ReadError::InvalidKSMFileMagicError);
        }

        Ok(KSMHeader::new())
    }
}
