//! A module describing a relocation data section in a Kerbal Object file
use crate::ko::errors::ReldSectionParseError;
use crate::ko::symbols::ReldEntry;
use crate::ko::SectionIdx;
use crate::{BufferIterator, WritableBuffer};
use std::slice::Iter;

/// A wrapper type that represents an index into a relocation data section of a KO file.
///
/// This type implements From<usize> and usize implements From<ReldIdx>, but this is provided
/// so that it takes 1 extra step to convert raw integers into ReldIdx which could stop potential
/// logical bugs.
///
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ReldIdx(usize);

impl From<usize> for ReldIdx {
    fn from(i: usize) -> Self {
        Self(i)
    }
}

impl From<u8> for ReldIdx {
    fn from(i: u8) -> Self {
        Self(i as usize)
    }
}

impl From<u16> for ReldIdx {
    fn from(i: u16) -> Self {
        Self(i as usize)
    }
}

impl From<u32> for ReldIdx {
    fn from(i: u32) -> Self {
        Self(i as usize)
    }
}

impl From<ReldIdx> for usize {
    fn from(reld_idx: ReldIdx) -> Self {
        reld_idx.0
    }
}

/// A relocation data section in a KerbalObject file.
///
/// This section stores a
/// list of entries which describe which symbols should replace instruction operands in the
/// final linked binary.
///
/// This does things like replace an external symbol with the correct value from a different
/// file.
///
/// See the [Kerbal Object file format docs](https://github.com/newcomb-luke/kerbalobjects.rs/blob/main/docs/KO-file-format.md)
/// to learn more, as if you are not familiar with the concept from ELF files, this may be confusing.
///
#[derive(Debug)]
pub struct ReldSection {
    entries: Vec<ReldEntry>,
    size: u32,
    section_index: SectionIdx,
}

impl ReldSection {
    /// Creates a new relocation data section with the provided section index.
    pub fn new(section_index: SectionIdx) -> Self {
        Self {
            entries: Vec::new(),
            size: 0,
            section_index,
        }
    }

    /// Creates a new relocation data section with the provided section index, with
    /// internal data structures pre-allocated for the provided amount of items
    pub fn with_capacity(amount: usize, section_index: SectionIdx) -> Self {
        Self {
            entries: Vec::with_capacity(amount),
            size: 0,
            section_index,
        }
    }

    /// Adds a new relocation data entry to this section.
    ///
    /// Returns the entry's index into this section
    pub fn add(&mut self, entry: ReldEntry) -> ReldIdx {
        self.size += entry.size_bytes();
        self.entries.push(entry);
        ReldIdx::from(self.entries.len() - 1)
    }

    /// Gets a relocation data entry at the provided index into this section, or None
    /// if the index doesn't exist
    pub fn get(&self, index: ReldIdx) -> Option<&ReldEntry> {
        self.entries.get(usize::from(index))
    }

    /// Returns an iterator over all relocation data entries in this section
    pub fn entries(&self) -> Iter<ReldEntry> {
        self.entries.iter()
    }

    /// The size of this relocation data section in bytes
    pub fn size(&self) -> u32 {
        self.size
    }

    /// The index of this section's section header
    pub fn section_index(&self) -> SectionIdx {
        self.section_index
    }

    /// Parses a relocation data section from the provided byte buffer
    pub fn parse(
        source: &mut BufferIterator,
        size: u32,
        section_index: SectionIdx,
    ) -> Result<Self, ReldSectionParseError> {
        let mut bytes_read = 0;
        let mut entries = Vec::new();

        while bytes_read < size {
            let entry = ReldEntry::parse(source).map_err(|e| {
                ReldSectionParseError::ReldEntryParseError(entries.len(), source.current_index(), e)
            })?;
            bytes_read += entry.size_bytes();

            entries.push(entry);
        }

        Ok(Self {
            entries,
            size,
            section_index,
        })
    }

    /// Converts this relocation data section to its binary representation and appends it to the provided buffer
    pub fn write(&self, buf: &mut impl WritableBuffer) {
        for reld_entry in self.entries.iter() {
            reld_entry.write(buf);
        }
    }
}
