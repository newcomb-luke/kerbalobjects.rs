//! Errors specifically for when parsing a KO file

use crate::ko::WritableKOFile;

/// A result with the error type of FinalizeError
pub type FinalizeResult = Result<WritableKOFile, FinalizeError>;

/// An error encountered when finalizing values in a KO file
#[derive(Debug)]
pub enum FinalizeError {
    /// An error returned when there is no section header table entry for a section in the KO file
    InvalidSectionIndexError(&'static str, usize),
}
impl std::error::Error for FinalizeError {}

impl std::fmt::Display for FinalizeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FinalizeError::InvalidSectionIndexError(section_name, section_index) => {
                write!(f, "Error updating KOFile headers. Section {} has index {}, which no section header exists for.", section_name, section_index)
            }
        }
    }
}
