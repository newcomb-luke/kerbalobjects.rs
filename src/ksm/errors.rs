//! Errors specifically for when parsing a KSM file

use crate::{KOSValueParseError, OpcodeParseError};
use thiserror::Error;

/// An error encountered when parsing a KSM file
#[derive(Debug, Error)]
pub enum KSMParseError {
    /// Error while decompressing a KSM file
    #[error("Error while decompressing KSM file: {0}")]
    DecompressionError(std::io::Error),
    /// Error while reading a KSM file header
    #[error("Error while reading KSM file header: {0}")]
    HeaderError(HeaderParseError),
    /// Error while reading a KSM file argument section
    #[error("Error while parsing KSM file argument section: {0}")]
    ArgumentSectionParseError(ArgumentSectionParseError),
    /// Error while reading a KSM file code section
    #[error("Error while parsing KSM file code section: {0}")]
    CodeSectionParseError(CodeSectionParseError),
    /// Error while reading a section type
    #[error("Error while parsing KSM file, encountered % after argument section at index {0}, expected a debug or code section, but found end of file")]
    MissingSectionType(usize),
    /// Error while reading a KSM file debug section
    #[error("Error while parsing KSM file debug section: {0}")]
    DebugSectionParseError(DebugSectionParseError),
}

/// An error encountered when parsing a KSM file's header
#[derive(Debug, Error, Copy, Clone)]
pub enum HeaderParseError {
    /// Error running out of bytes while reading KSM header
    #[error("End of file reached while reading")]
    EOF,
    /// Error while reading invalid KSM header
    #[error("Not a KSM file. Expected first 4 bytes to be 0x6b035845, found 0x{0:x}")]
    InvalidMagic(u32),
}

/// An error parsing an argument section
#[derive(Error, Debug, Copy, Clone)]
pub enum ArgumentSectionParseError {
    /// Error running out of bytes while reading argument section header
    #[error("End of file reached. Expected %A")]
    MissingHeader,
    /// Error while reading invalid argument section header
    #[error("Expected %A as the start of the argument section, which is the required first section in a KSM file, found: 0x{0:x}")]
    InvalidHeader(u16),
    /// Error running out of bytes while reading argument section number of index bytes
    #[error("End of file reached while trying to parse the number of argument index bytes")]
    MissingNumArgIndexBytes,
    /// Error while reading invalid argument number of index bytes
    #[error("Invalid value of {0} for NumArgIndexBytes after section header. Supported values are 1, 2, 3, and 4")]
    InvalidNumArgIndexBytes(u8),
    /// Error running out of bytes while reading KOSValue in argument section
    #[error("End of file reached while reading next KOSValue. The argument section ends when a % is found, and a code section begins")]
    EOF,
    /// Error while reading invalid argument section KOSValue
    #[error("Error while parsing KOSValue at byte offset {0}: {1}")]
    KOSValueParseError(usize, KOSValueParseError),
}

/// An error parsing a code section
#[derive(Error, Debug, Copy, Clone)]
pub enum CodeSectionParseError {
    /// Error running out of bytes while reading code section type
    #[error("End of file reached while parsing code section type")]
    MissingCodeSectionType,
    /// Error reading invalid code section type
    #[error("Found 0x{0:x} after section header `%`, expected F, I, or M")]
    InvalidCodeSectionType(u8),
    /// Error running out of bytes while reading code section instruction
    #[error("End of file reached while reading next kOS instruction. The code section ends when a % is found, and a new code or debug section begins")]
    EOF,
    /// Error reading invalid code section instruction
    #[error("Error while parsing instruction: {0}")]
    InstrParseError(InstrParseError),
}

/// An error parsing a code section instruction
#[derive(Error, Debug, Copy, Clone)]
pub enum InstrParseError {
    /// Error reading invalid code section instruction opcode
    #[error("Error reading opcode at file byte offset {0}: {1}")]
    OpcodeParseError(usize, OpcodeParseError),
    /// Error running out of bytes while reading code section instruction operand
    #[error("Reached EOF while reading operand {0}")]
    MissingOperand(usize),
}

/// An error parsing a debug section
#[derive(Error, Debug, Copy, Clone)]
pub enum DebugSectionParseError {
    /// Error reading invalid debug section entry
    #[error("Error while parsing debug entry at offset {0}: {1}")]
    DebugEntryParseError(usize, DebugEntryParseError),
    /// Error running out of bytes while reading debug section range size
    #[error("End of file reached while trying to parse the debug range size")]
    MissingDebugRangeSize,
    /// Error reading invalid debug section range size
    #[error("Invalid value of {0} for range size after section header. Supported values are 1, 2, 3, and 4")]
    InvalidDebugRangeSize(u8),
}

/// An error parsing a debug section entry
#[derive(Error, Debug, Copy, Clone)]
pub enum DebugEntryParseError {
    /// Error running out of bytes while reading debug section entry line number
    #[error("Reached EOF while reading line number")]
    MissingLineNumber,
    /// Error running out of bytes while reading debug section entry number of ranges
    #[error("Reached EOF while reading the number of ranges")]
    MissingNumRanges,
    /// Error running out of bytes while reading debug section entry range
    #[error("Reached EOF while reading debug range {0}")]
    MissingRange(usize),
}
