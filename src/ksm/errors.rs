//! Errors specifically for when parsing a KSM file

use crate::ksm::sections::ArgSectionReadError;
use thiserror::Error;

/// An error encountered when reading a KSM file
#[derive(Debug, Error)]
pub enum KSMReadError {
    /// An error reading the argument section
    #[error(transparent)]
    ArgumentSectionReadError(ArgumentSectionReadError),
}

/// An error reading the argument section
#[derive(Error, Debug)]
#[error("Error reading argument section")]
pub struct ArgumentSectionReadError {
    /// The source/cause of the error
    #[source]
    pub source: ArgSectionReadError,
}
