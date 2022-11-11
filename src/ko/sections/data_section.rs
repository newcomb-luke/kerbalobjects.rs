//! A module describing a data section in a Kerbal Object file
use crate::ko::errors::DataSectionParseError;
use crate::ko::SectionIdx;
use crate::{BufferIterator, FromBytes, KOSValue, ToBytes, WritableBuffer};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::slice::Iter;

/// A wrapper type that represents an index into a data section of a KO file.
///
/// This type implements From<usize> and usize implements From<DataIdx>, but this is provided
/// so that it takes 1 extra step to convert raw integers into DataIdx which could stop potential
/// logical bugs.
///
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct DataIdx(u32);

impl From<usize> for DataIdx {
    fn from(i: usize) -> Self {
        Self(i as u32)
    }
}

impl From<u8> for DataIdx {
    fn from(i: u8) -> Self {
        Self(i as u32)
    }
}

impl From<u16> for DataIdx {
    fn from(i: u16) -> Self {
        Self(i as u32)
    }
}

impl From<u32> for DataIdx {
    fn from(i: u32) -> Self {
        Self(i)
    }
}

impl From<DataIdx> for u32 {
    fn from(data_idx: DataIdx) -> Self {
        data_idx.0
    }
}

impl From<DataIdx> for usize {
    fn from(data_idx: DataIdx) -> Self {
        data_idx.0 as usize
    }
}

impl DataIdx {
    /// A useful constant as a placeholder for an operand that will just be later replaced by a Relocation Data Table entry.
    /// This is set to u32::MAX because it is highly unlikely that it will actually be a valid index, and possible errors with
    /// forgetting to include a ReldEntry will be caught when the file is used.
    pub const PLACEHOLDER: DataIdx = DataIdx(u32::MAX);
}

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
    map: HashMap<u64, usize>,
    data: Vec<KOSValue>,
    size: u32,
    section_index: SectionIdx,
}

impl DataSection {
    /// Creates a new data section, with the provided section index
    pub fn new(section_index: SectionIdx) -> Self {
        Self {
            map: HashMap::new(),
            data: Vec::new(),
            size: 0,
            section_index,
        }
    }

    /// Creates a new data section with the provided section index, with
    /// internal data structures pre-allocated for the provided amount of items
    pub fn with_capacity(amount: usize, section_index: SectionIdx) -> Self {
        Self {
            map: HashMap::with_capacity(amount),
            data: Vec::with_capacity(amount),
            size: 0,
            section_index,
        }
    }

    /// Locates a given value in this data section and returns the index
    /// if it does exist, or None if it doesn't
    pub fn position(&self, value: &KOSValue) -> Option<DataIdx> {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        let hash = hasher.finish();

        self.map
            .get(&hash)
            .cloned()
            .map(|v| DataIdx::from(v as u32))
    }

    /// Add a new KOSValue to this data section, checking if it is a duplicate, and
    /// returning the index of the value. If it already exists, the index of that value is
    /// returned, if it does not, then it is added, and the new index is returned.
    pub fn add_checked(&mut self, value: KOSValue) -> DataIdx {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        let hash = hasher.finish();

        if let Some(i) = self.map.get(&hash) {
            DataIdx::from(*i)
        } else {
            self.size += value.size_bytes() as u32;

            let index = self.data.len();

            self.map.insert(hash, index);
            self.data.push(value);

            DataIdx::from(index)
        }
    }

    /// Unconditionally adds a KOSValue to this data section, and returns the index into this
    /// data section that it resides at
    pub fn add(&mut self, value: KOSValue) -> DataIdx {
        self.size += value.size_bytes() as u32;

        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        let hash = hasher.finish();

        self.map.insert(hash, self.data.len());
        self.data.push(value);

        DataIdx::from(self.data.len() - 1)
    }

    /// Gets the KOSValue at the provided index into this data section if it exists,
    /// or None if it does not.
    pub fn get(&self, index: DataIdx) -> Option<&KOSValue> {
        self.data.get(usize::from(index))
    }

    /// Returns an iterator over all KOSValues in this data section
    pub fn data(&self) -> Iter<KOSValue> {
        self.data.iter()
    }

    /// The size of this data section in bytes
    pub fn size(&self) -> u32 {
        self.size
    }

    /// The index of this section's section header
    pub fn section_index(&self) -> SectionIdx {
        self.section_index
    }

    /// Parses a data section from the provided byte buffer
    pub fn parse(
        source: &mut BufferIterator,
        size: u32,
        section_index: SectionIdx,
    ) -> Result<Self, DataSectionParseError> {
        let mut bytes_read = 0;
        let mut data = Vec::new();
        let mut map = HashMap::new();

        while bytes_read < size as usize {
            let kos_value = KOSValue::from_bytes(source).map_err(|e| {
                DataSectionParseError::KOSValueParseError(source.current_index(), e)
            })?;
            bytes_read += kos_value.size_bytes() as usize;

            let mut hasher = DefaultHasher::new();
            kos_value.hash(&mut hasher);
            let hash = hasher.finish();

            map.insert(hash, data.len());
            data.push(kos_value);
        }

        Ok(Self {
            map,
            data,
            size,
            section_index,
        })
    }

    /// Converts this data section to its binary representation and appends it to the provided buffer
    pub fn write(&self, buf: &mut impl WritableBuffer) {
        for value in self.data.iter() {
            value.to_bytes(buf);
        }
    }
}

#[cfg(test)]
impl PartialEq for DataSection {
    fn eq(&self, other: &Self) -> bool {
        if self.data().count() != other.data().count() {
            return false;
        }

        for (r, h) in self.data().zip(other.data()) {
            if r != h {
                return false;
            }
        }

        true
    }
}
