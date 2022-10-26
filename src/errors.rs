//! Generic errors while reading KSM or KO files
use thiserror::Error;

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
