pub type UpdateResult = Result<(), UpdateError>;

#[derive(Debug)]
pub enum UpdateError {
    InvalidSectionIndexError(&'static str, usize),
}
impl std::error::Error for UpdateError {}

impl std::fmt::Display for UpdateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpdateError::InvalidSectionIndexError(section_name, section_index) => {
                write!(f, "Error updating KOFile headers. Section {} has index {}, which no section header exists for.", section_name, section_index)
            }
        }
    }
}
