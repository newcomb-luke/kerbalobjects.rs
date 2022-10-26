//! Errors specifically for when parsing a KO file

use crate::ko::sections::SectionKind;
use crate::{KOSValueParseError, OpcodeParseError};
use thiserror::Error;

/// An error encountered when parsing a KO file
#[derive(Debug, Error, Clone)]
pub enum KOParseError {
    /// Error while reading a KO file header
    #[error("Error while reading KO file header: {0}")]
    HeaderError(HeaderParseError),
    /// Error while reading a KO file section header table, where there is no Null section header
    #[error(
        "KO file section header tables MUST contain a null section header at index 0 in the table, found section kind {0:?}"
    )]
    MissingNullSectionHeader(SectionKind),
    /// Error while reading a KO file section header table, where there is a stray Null section header
    #[error(
    "KO file null section header table entries must only be the first entry in the table, although the section header at index is of kind null {0}"
    )]
    StrayNullSectionHeader(u16),
    /// Error while reading a KO file string table
    #[error("Error while reading KO file string table: {0}")]
    StringTableParseError(StringTableParseError),
    /// Error while reading a KO file symbol table
    #[error("Error while reading KO file symbol table: {0}")]
    SymbolTableParseError(SymbolTableParseError),
    /// Error while reading a KO file data section
    #[error("Error while reading KO file data section: {0}")]
    DataSectionParseError(DataSectionParseError),
    /// Error while reading a KO file function section
    #[error("Error while reading KO file function section: {0}")]
    FunctionSectionParseError(FunctionSectionParseError),
    /// Error while reading a KO file relocation data section
    #[error("Error while reading KO file relocation data section: {0}")]
    ReldSectionParseError(ReldSectionParseError),
    /// Error that the debug section of KerbalObject files is not yet defined, just reserved
    #[error("Debug sections are currently unsupported and undefined, but the section kind was used for section header {0}")]
    DebugSectionUnsupportedError(u16),
    /// Error while reading KO file section header table
    #[error("Error while reading KO file section header table: {0}")]
    SectionHeaderParseError(SectionHeaderParseError),
    /// A null (0) section header table index was provided in the header
    #[error("Error while reading KO file: .shstrtab section index must not be 0")]
    NullShStrTabIndexError,
    /// An invalid section header table index was provided in the header
    #[error("Error while reading KO file: .shstrtab section index `{0}` is out of bounds of the number of sections `{1}`")]
    InvalidShStrTabIndexError(u16, u16),
}

/// An error encountered when parsing a KO file's header
#[derive(Debug, Error, Copy, Clone)]
pub enum HeaderParseError {
    /// Reached EOF before reading KO file magic
    #[error("Input buffer was empty")]
    MissingMagicError,
    /// Encountered an invalid KO file magic value, which means this was not a KO file, or corrupt
    #[error("Input file does not appear to be a KO file. Expected the first four bytes to be {0:08x}, found {1:08x}")]
    InvalidMagicError(u32, u32),
    /// Reached EOF before reading KO file version
    #[error("Reached end of file trying to read KerbalObject file version")]
    MissingVersionError,
    /// Encountered an unsupported KO file version
    #[error("Only KerbalObject files version {0} can be read with this library version, tried to read file with version {1}")]
    UnsupportedVersionError(u8, u8),
    /// Reached EOF before reading the number of section headers in the file
    #[error("Reached end of file trying to read the number of headers in the header table")]
    MissingNumHeaders,
    /// Encountered an invalid section header string table index
    #[error("Reached end of file trying to read section header string table index")]
    MissingShStrTabIndex,
}

/// An error encountered when parsing a KO file's string table
#[derive(Debug, Error, Clone)]
pub enum StringTableParseError {
    /// Reached EOF before reaching expected size
    #[error("Reached end of file at file byte {0}, expected total size: {1}")]
    EOFError(usize, u32),
    /// A string that was read is not valid UTF-8
    #[error("String number {0} that was read is invalid UTF-8 bytes: {1}")]
    InvalidUtf8Error(usize, std::string::FromUtf8Error),
}

/// An error encountered when parsing a section header from the section header table
#[derive(Debug, Error, Copy, Clone)]
pub enum SectionHeaderParseError {
    /// Reached EOF before reading name index
    #[error("Reached end of file trying to read section name index")]
    MissingNameIdxError,
    /// Reached EOF before reading section kind
    #[error("Reached end of file trying to read section kind for section header at byte {0}")]
    MissingSectionKindError(usize),
    /// Encountered invalid section kind
    #[error(
        "Section kind invalid for section header at byte {0}, expected 0, 1, 2, 3, 4, 5, or 6, found {1}"
    )]
    InvalidSectionKindError(usize, u8),
    /// Reached EOF before reading section size
    #[error("Reached end of file trying to read section size for section header at byte {0}")]
    MissingSizeError(usize),
}

/// An error encountered when parsing a relocation entry from the relocation data table
#[derive(Debug, Error, Copy, Clone)]
pub enum ReldEntryParseError {
    /// Reached EOF before reading section index
    #[error("Reached end of file trying to read entry section index")]
    MissingSectionIndexError,
    /// Reached EOF before reading instruction index
    #[error("Reached end of file trying to read entry instruction index")]
    MissingInstructionIndexError,
    /// Reached EOF before reading instruction operand index
    #[error("Reached end of file trying to read entry operand index")]
    MissingOperandIndexError,
    /// Encountered invalid operand index while reading
    #[error("Entry has an invalid operand index of {0}, expected either 1 or 2")]
    InvalidOperandIndexError(u8),
    /// Reached EOF before reading symbol table index
    #[error("Reached end of file trying to read entry symbol index")]
    MissingSymbolIndexError,
}

/// An error encountered when parsing a KOSymbol from the symbol table
#[derive(Debug, Error, Copy, Clone)]
pub enum SymbolParseError {
    /// Reached EOF before reading name index
    #[error("Reached end of file trying to read symbol name index")]
    MissingNameIndexError,
    /// Reached EOF before reading value index
    #[error("Reached end of file trying to read symbol value index")]
    MissingValueIndexError,
    /// Reached EOF before reading symbol size
    #[error("Reached end of file trying to read symbol size")]
    MissingSymbolSizeError,
    /// Reached EOF before reading symbol binding
    #[error("Reached end of file trying to read symbol binding")]
    MissingSymbolBindingError,
    /// Encountered an invalid symbol binding value
    #[error("Symbol has an invalid symbol binding value {0}, expected 0, 1, or 2")]
    InvalidSymbolBindingError(u8),
    /// Reached EOF before reading symbol type
    #[error("Reached end of file trying to read symbol type")]
    MissingSymbolTypeError,
    /// Encountered an invalid symbol type value
    #[error("Symbol has an invalid symbol type value {0}, expected 0, 1, 2, 3, or 4")]
    InvalidSymbolTypeError(u8),
    /// Reached EOF before reading symbol section index
    #[error("Reached end of file trying to read symbol section index")]
    MissingSectionIndexError,
}

/// An error encountered while parsing a symbol table from a KO file
#[derive(Debug, Error, Copy, Clone)]
pub enum SymbolTableParseError {
    /// Error while attempting to parse a symbol table entry
    #[error("Failed to parse symbol number {0} at byte offset {1}: {2}")]
    SymbolParseError(usize, usize, SymbolParseError),
}

/// An error encountered while parsing a data section from a KO file
#[derive(Debug, Error, Copy, Clone)]
pub enum DataSectionParseError {
    /// Error while attempting to parse a data section entry
    #[error("Error reading KOSValue at file byte offset {0}: {1}")]
    KOSValueParseError(usize, KOSValueParseError),
}

/// An error parsing a function section instruction
#[derive(Error, Debug, Copy, Clone)]
pub enum InstrParseError {
    /// Error reading invalid function section instruction opcode
    #[error("Error reading opcode at file byte offset {0}: {1}")]
    OpcodeParseError(usize, OpcodeParseError),
    /// Error running out of bytes while reading function section instruction operand
    #[error("Reached EOF while reading operand {0}")]
    MissingOperand(usize),
}

/// An error encountered while parsing a function section from a KO file
#[derive(Debug, Error, Copy, Clone)]
pub enum FunctionSectionParseError {
    /// Error reading invalid function section instruction
    #[error("Error reading instruction at file byte offset {0}: {1}")]
    InstrParseError(usize, InstrParseError),
}

/// An error encountered while parsing a relocation data section from a KO file
#[derive(Debug, Error, Copy, Clone)]
pub enum ReldSectionParseError {
    /// Error reading invalid relocation data section entry
    #[error("Failed to parse relocation data entry {0} at byte offset {1}: {2}")]
    ReldEntryParseError(usize, usize, ReldEntryParseError),
}

/// An error encountered when calling .validate() on a KOFile instance to attempt to convert it to
/// a writable format
#[derive(Debug, Error, Clone)]
pub enum ValidationError {
    /// Error when there is no section header table entry for a section in the KO file
    #[error(
        "Error validating KOFile: An inserted section of kind {0:?} has no corresponding section header entry at index {1}"
    )]
    InvalidSectionIndexError(SectionKind, u16),
    /// Error when the section header table entry kind for a section doesn't match the section's kind
    #[error("Error validating KOFile: Section kind mismatch. An inserted section of kind {0:?} has a corresponding section header entry at index {1} that is of type {2:?}")]
    SectionKindMismatchError(SectionKind, u16, SectionKind),
    /// Error when a section header in the KOFile doesn't have a corresponding section
    #[error("Error validating KOFile: An inserted section header at index {0} of kind {1:?} and name {2} has no corresponding section within the file")]
    InvalidSectionHeaderError(u16, SectionKind, String),
    /// Error when a section header in the KOFile's name index is invalid
    #[error("Error validating KOFile: An inserted section header at index {0} of kind {1:?} has an invalid name index of {2}")]
    InvalidSectionHeaderNameIndexError(u16, SectionKind, usize),
}
