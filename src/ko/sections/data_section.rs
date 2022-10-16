//! A module describing a data section in a Kerbal Object file
use crate::ko::sections::SectionIndex;
use crate::ko::SectionFromBytes;
use crate::{FileIterator, FromBytes, KOSValue, ReadResult, ToBytes};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::slice::Iter;

/// A data section in a KerbalObject file.
///
/// This section is responsible
/// for storing all operands to all instructions contained within Function sections in this file,
/// but also for storing any data that is exported as global symbols in the symbol table.
///
/// Contrary to how Kerbal Machine Code argument sections work, data section indices are
/// 0-indexed, and indexes into the parsed vector of values, so the first index will be 0,
/// the second index will be 1, the third 2, etc.
///
#[derive(Debug)]
pub struct DataSection {
    hashes: Vec<u64>,
    data: Vec<KOSValue>,
    size: u32,
    /// The index of this section's section header
    pub section_index: usize,
}

impl SectionIndex for DataSection {
    fn section_index(&self) -> usize {
        self.section_index
    }
}

impl DataSection {
    /// Creates a new data section, with the provided section index
    pub fn new(section_index: usize) -> Self {
        Self {
            hashes: Vec::new(),
            data: Vec::new(),
            size: 0,
            section_index,
        }
    }

    /// Creates a new data section with the provided section index, with
    /// internal data structures pre-allocated for the provided amount of items
    pub fn with_capacity(amount: usize, section_index: usize) -> Self {
        Self {
            hashes: Vec::with_capacity(amount),
            data: Vec::with_capacity(amount),
            size: 0,
            section_index,
        }
    }

    /// Returns the index into this data section that the first occurrence of a KOSValue resides
    /// at, or None if no such value is in this section.
    pub fn find(&self, value: &KOSValue) -> Option<usize> {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        let hash = hasher.finish();

        self.hashes.iter().position(|item| *item == hash)
    }

    /// Add a new KOSValue to this data section, checking if it is a duplicate, and
    /// returning the index of the value. If it already exists, the index of that value is
    /// returned, if it does not, then it is added, and the new index is returned.
    pub fn add_checked(&mut self, value: KOSValue) -> usize {
        match self.find(&value) {
            Some(index) => index,
            None => self.add(value),
        }
    }

    /// Unconditionally adds a KOSValue to this data section, and returns the index into this
    /// data section that it resides at
    pub fn add(&mut self, value: KOSValue) -> usize {
        self.size += value.size_bytes() as u32;

        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        let hash = hasher.finish();

        self.hashes.push(hash);
        self.data.push(value);

        self.data.len() - 1
    }

    /// Gets the KOSValue at the provided index into this data section if it exists,
    /// or None if it does not.
    pub fn get(&self, index: usize) -> Option<&KOSValue> {
        self.data.get(index)
    }

    /// Returns an iterator over all KOSValues in this data section
    pub fn data(&self) -> Iter<KOSValue> {
        self.data.iter()
    }

    /// The size of this data section in bytes
    pub fn size(&self) -> u32 {
        self.size
    }
}

impl SectionFromBytes for DataSection {
    fn from_bytes(
        source: &mut FileIterator,
        size: usize,
        section_index: usize,
    ) -> ReadResult<Self> {
        let mut read = 0;
        let mut data = Vec::new();
        let mut hashes = Vec::new();

        while read < size {
            let kos_value = KOSValue::from_bytes(source)?;
            read += kos_value.size_bytes();

            let mut hasher = DefaultHasher::new();
            kos_value.hash(&mut hasher);
            let hash = hasher.finish();

            hashes.push(hash);
            data.push(kos_value);
        }

        #[cfg(feature = "print_debug")]
        {
            println!("Data section:");

            for d in data.iter() {
                println!("\t{:?}", d);
            }
        }

        Ok(DataSection {
            hashes,
            data,
            size: size as u32,
            section_index,
        })
    }
}

impl ToBytes for DataSection {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        for value in self.data.iter() {
            value.to_bytes(buf);
        }
    }
}
