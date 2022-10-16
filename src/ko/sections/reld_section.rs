//! A module describing a relocation data section in a Kerbal Object file
use crate::ko::sections::SectionIndex;
use crate::ko::symbols::ReldEntry;
use crate::ko::SectionFromBytes;
use crate::{FileIterator, FromBytes, ReadResult, ToBytes};
use std::slice::Iter;

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
    /// The index of this section's section header
    pub section_index: usize,
}

impl SectionIndex for ReldSection {
    fn section_index(&self) -> usize {
        self.section_index
    }
}

impl ReldSection {
    /// Creates a new relocation data section with the provided section index.
    pub fn new(section_index: usize) -> Self {
        Self {
            entries: Vec::new(),
            size: 0,
            section_index,
        }
    }

    /// Creates a new relocation data section with the provided section index, with
    /// internal data structures pre-allocated for the provided amount of items
    pub fn with_capacity(amount: usize, section_index: usize) -> Self {
        Self {
            entries: Vec::with_capacity(amount),
            size: 0,
            section_index,
        }
    }

    /// Adds a new relocation data entry to this section.
    ///
    /// Returns the entry's index into this section
    pub fn add(&mut self, entry: ReldEntry) -> usize {
        self.size += entry.size_bytes();
        self.entries.push(entry);
        self.entries.len() - 1
    }

    /// Gets a relocation data entry at the provided index into this section, or None
    /// if the index doesn't exist
    pub fn get(&self, index: usize) -> Option<&ReldEntry> {
        self.entries.get(index)
    }

    /// Returns an iterator over all relocation data entries in this section
    pub fn entries(&self) -> Iter<ReldEntry> {
        self.entries.iter()
    }

    /// The size of this relocation data section in bytes
    pub fn size(&self) -> u32 {
        self.size
    }
}

impl ToBytes for ReldSection {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        for reld_entry in self.entries.iter() {
            reld_entry.to_bytes(buf);
        }
    }
}

impl SectionFromBytes for ReldSection {
    fn from_bytes(
        source: &mut FileIterator,
        size: usize,
        section_index: usize,
    ) -> ReadResult<Self> {
        let mut read = 0;
        let mut entries = Vec::new();

        #[cfg(feature = "print_debug")]
        {
            println!("Reld section:");
        }

        while read < size {
            let entry = ReldEntry::from_bytes(source)?;
            read += entry.size_bytes() as usize;

            #[cfg(feature = "print_debug")]
            {
                println!("\t{:?}", entry);
            }

            entries.push(entry);
        }

        Ok(ReldSection {
            entries,
            size: size as u32,
            section_index,
        })
    }
}
