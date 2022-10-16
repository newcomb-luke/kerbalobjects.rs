//! Errors for reading KSM or KO files
use thiserror::Error;

pub use crate::ksm::errors::*;

/// The result of attempting to read an object file, KO or KSM. The error type is a ReadError
pub type ReadResult<T> = Result<T, ReadError>;

#[allow(missing_docs)]
/// An error type which encompasses reading KSM and KO files.
#[derive(Debug, Error)]
pub enum ReadError {
    #[error("Error reading Kerbal Machine Code file")]
    KSMReadError(#[from] KSMReadError),
    #[error("Error reading file, unexpected EOF.")]
    UnexpectedEOF,
    #[error("Error reading opcode for instruction, ran out of bytes.")]
    OpcodeReadError,
    #[error("Error reading opcode for instruction, value {0:x} is not a valid opcode.")]
    BogusOpcodeReadError(u8),
    #[error("Error reading operand for instruction, ran out of bytes.")]
    OperandReadError,
    #[error("Error reading symbol binding, ran out of bytes.")]
    SymBindReadError,
    #[error("Error reading symbol binding, unknown binding with value {0:x}")]
    UnknownSimBindReadError(u8),
    #[error("Error reading symbol type, ran out of bytes.")]
    SymTypeReadError,
    #[error("Error reading symbol type, unknown type with value {0:x}.")]
    UnknownSimTypeReadError(u8),
    #[error("Error reading symbol while parsing constant for {0}, ran out of bytes.")]
    KOSymbolConstantReadError(&'static str),
    #[error("Error reading kOS value, unknown type with value {0:x}.")]
    KOSValueTypeReadError(u8),
    #[error("Error reading kOS value, ran out of bytes.")]
    KOSValueReadError,
    #[error("Error reading section kind, ran out of bytes.")]
    SectionKindReadError,
    #[error("Error reading section kind, unknown kind with value {0:x}")]
    UnknownSectionKindReadError(u8),
    #[error("Error reading section header while parsing constant {0}, ran out of bytes.")]
    SectionHeaderConstantReadError(&'static str),
    #[error("Error reading string table, ran out of bytes.")]
    StringTableReadError,
    #[error(
        "Error reading KerbalObject file header while parsing constant {0}, ran out of bytes."
    )]
    KOHeaderReadError(&'static str),
    #[error(
        "Error reading kerbal machine code file while parsing constant {0}, ran out of bytes."
    )]
    KSMHeaderReadError(&'static str),
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

#[allow(missing_docs)]
/// An error for simply reading a KOSvalue
#[derive(Debug, Error)]
pub enum KOSValueReadError {
    #[error("boolean")]
    BoolReadError,
    #[error("unsigned byte")]
    U8ReadError,
    #[error("signed byte")]
    I8ReadError,
    #[error("unsigned 16-bit integer")]
    U16ReadError,
    #[error("signed 16-bit integer")]
    I16ReadError,
    #[error("unsigned 32-bit integer")]
    U32ReadError,
    #[error("signed 32-bit integer")]
    I32ReadError,
    #[error("float")]
    F32ReadError,
    #[error("double")]
    F64ReadError,
    #[error("string")]
    StringReadError,
}
