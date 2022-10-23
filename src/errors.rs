//! Generic errors while reading KSM or KO files
use thiserror::Error;

pub use crate::ksm::errors::*;

/// The result of attempting to read an object file, KO or KSM. The error type is a ReadError
pub type ReadResult<T> = Result<T, ReadError>;

/// An error type that describes an error while just parsing a KOSValue
#[derive(Debug, Error, Copy, Clone)]
pub enum KOSValueParseError {
    /// Error running out of bytes while reading KOSValue
    #[error("EOF reached while parsing KOSValue")]
    EOF,
    /// Error reading invalid KOSValue type
    #[error("Invalid KOSValue type: {0}")]
    InvalidType(u8),
}

#[allow(missing_docs)]
/// An error type that describes an error while just reading a KOSValue
#[derive(Debug, Error, Copy, Clone)]
pub enum KOSValueReadError {
    #[error("EOF reached while reading KOSValue")]
    EOF,
    #[error("Invalid KOSValue type: {0}")]
    InvalidType(u8),
}

#[allow(missing_docs)]
/// An error type that describes an error while just reading an Opcode
#[derive(Debug, Error, Copy, Clone)]
pub enum OpcodeReadError {
    #[error("EOF reached while reading Opcode")]
    EOF,
    #[error("Invalid Opcode: {0}")]
    InvalidOpcode(u8),
}

/// An error type that describes an error while just parsing an Opcode
#[derive(Debug, Error, Copy, Clone)]
pub enum OpcodeParseError {
    /// Error running out of bytes while reading instruction opcode
    #[error("EOF reached while parsing opcode")]
    EOF,
    /// Error reading invalid instruction opcode
    #[error("Invalid opcode: {0}")]
    InvalidOpcode(u8),
}

#[allow(missing_docs)]
/// An error type which encompasses reading KSM and KO files.
#[derive(Debug, Error)]
pub enum ReadError {
    #[error("Error reading KerbalObject file, unsupported KO file version {0}")]
    VersionMismatchError(u8),
    #[error("Error reading KerbalObject file, debug sections are currently unsupported.")]
    DebugSectionUnsupportedError,
    #[error("Error reading KerbalObject file, expected section: {0}")]
    MissingSectionError(&'static str),
    #[error("Error reading KerbalObject file, input file does not appear to be a KO file.")]
    InvalidKOFileMagicError,
    #[error(
        "Error reading Kerbal Machine Code file, input file does not appear to be a KSM file."
    )]
    InvalidKSMFileMagicError,
    #[error("Error while trying to decompress Kerbal Machine Code file: {0}")]
    KSMDecompressionError(std::io::Error),
    #[error(
        "Error reading Kerbal Machine Code file, expected argument section, ran out of bytes."
    )]
    MissingArgumentSectionError,
    #[error(
        "Error reading Kerbal Machine Code file, expected argument section, instead found {0:x}"
    )]
    ExpectedArgumentSectionError(u16),
    #[error("Error reading Kerbal Machine Code file argument section num index bytes, ran out of bytes.")]
    NumIndexBytesReadError,
    #[error("Error reading Kerbal Machine Code file argument section num index bytes, invalid number: {0}")]
    InvalidNumIndexBytesError(u8),
    #[error("Error reading Kerbal Machine Code file argument section, expected argument, ran out of bytes.")]
    ArgumentSectionReadError,
    #[error("Error reading KSM section type, ran out of bytes.")]
    CodeTypeReadError,
    #[error("Error reading KSM section type, unknown type with value: {0}")]
    UnknownCodeTypeReadError(char),
    #[error("Error reading KSM file, expected code section, ran out of bytes.")]
    MissingCodeSectionError,
    #[error("Error reading KSM file, expected code section, found {0:?}.")]
    ExpectedCodeSectionError(u8),
    #[error("Error reading relocation data, ran out of bytes.")]
    ReldReadError,
    #[error("Error reading code section, ran out of bytes.")]
    CodeSectionReadError,
    #[error("Error reading debug section, ran out of bytes.")]
    DebugRangeReadError,
    #[error("Error reading debug entry, ran out of bytes.")]
    DebugEntryReadError,
    #[error("Error reading KSM file debug section, expected debug section, ran out of bytes.")]
    MissingDebugSectionError,
    #[error("Error reading KSM file, expected debug section header, found {0:?}.")]
    ExpectedDebugSectionError(u8),
    #[error("Error reading KSM file debug section, ran out of bytes.")]
    DebugSectionReadError,
}
