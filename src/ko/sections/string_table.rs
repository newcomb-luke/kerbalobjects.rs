//! A module describing a string table section in a Kerbal Object file
use crate::ko::errors::StringTableParseError;
use crate::ko::SectionIdx;
use crate::{BufferIterator, WritableBuffer};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::Hasher;
use std::slice::Iter;

/// A wrapper type that represents an index into a string table of a KO file.
///
/// This type implements From<usize> and From<u32> and usize/32 implements From<StringIdx>, but this is provided
/// so that it takes 1 extra step to convert raw integers into StringIdx which could stop potential
/// logical bugs.
///
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct StringIdx(u32);

impl From<usize> for StringIdx {
    fn from(i: usize) -> Self {
        Self(i as u32)
    }
}

impl From<u8> for StringIdx {
    fn from(i: u8) -> Self {
        Self(i as u32)
    }
}

impl From<u16> for StringIdx {
    fn from(i: u16) -> Self {
        Self(i as u32)
    }
}

impl From<u32> for StringIdx {
    fn from(i: u32) -> Self {
        Self(i)
    }
}

impl From<StringIdx> for usize {
    fn from(str_idx: StringIdx) -> Self {
        str_idx.0 as usize
    }
}

impl From<StringIdx> for u32 {
    fn from(str_idx: StringIdx) -> Self {
        str_idx.0
    }
}

impl StringIdx {
    /// A constant representing the index of the empty string present in all string tables (0)
    pub const EMPTY: StringIdx = StringIdx(0u32);
}

/// A string table section in a KerbalObject file.
///
/// A module describing a string table section in a Kerbal Object file. This section stores
/// a list of strings, which other Kerbal Object constructs can index into.
///
/// Each section contains as its first index (index 0) an empty string.
///
#[derive(Debug)]
pub struct StringTable {
    // This may seem a bit redundant, and it is, but this is an O(1) method of getting the index,
    // after we have already hashed it, and it doesn't have to store any Strings.
    map: HashMap<u64, usize>,
    contents: Vec<String>,
    section_index: SectionIdx,
    size: u32,
}

impl StringTable {
    /// Creates a new string table section with the provided section index
    pub fn new(section_index: SectionIdx) -> Self {
        // DRY
        Self::with_capacity(0, section_index)
    }

    /// Creates a new string table section with the provided section index, with
    /// internal data structures pre-allocated for the provided number of strings, +1 for
    /// the null string at index 0.
    pub fn with_capacity(amount: usize, section_index: SectionIdx) -> Self {
        let empty = String::new();
        let mut hasher = DefaultHasher::new();
        hasher.write(empty.as_bytes());
        let hash = hasher.finish();

        // Initialize with index 0 being the empty string
        let mut contents = Vec::with_capacity(1 + amount);
        contents.push(empty);
        let mut map = HashMap::with_capacity(1 + amount);
        map.insert(hash, 0);

        Self {
            map,
            contents,
            section_index,
            size: 1,
        }
    }

    /// Gets the string at the index into this string table, or None if it doesn't
    /// exist
    pub fn get(&self, index: StringIdx) -> Option<&String> {
        self.contents.get(usize::from(index))
    }

    /// Locates a given string in this string table and returns the index
    /// if it does exist, or None if it doesn't
    pub fn position(&self, s: impl AsRef<str>) -> Option<StringIdx> {
        let mut hasher = DefaultHasher::new();
        hasher.write(s.as_ref().as_bytes());
        let hash = hasher.finish();

        self.map.get(&hash).cloned().map(|v| StringIdx::from(v))
    }

    // This doesn't make any sense to have, but I already wrote it
    /*
    /// Finds a given string in this string table and returns a reference to it
    /// if it does exist, or None if it doesn't
    pub fn find(&self, s: impl AsRef<str>) -> Option<&String> {
        let mut hasher = DefaultHasher::new();
        hasher.write(s.as_ref().as_bytes());
        let hash = hasher.finish();

        let idx = *self.map.get(&hash)?;

        self.contents.get(idx)
    }
     */

    /// Returns an iterator over all of the strings contained in this string table
    pub fn strings(&self) -> Iter<String> {
        self.contents.iter()
    }

    /// Adds a string to this string table, but checks if it is already in this table.
    /// Returns the index of the string, if it already existed, the index is just
    /// found and returned. If it doesn't, it is inserted and the new index is returned.
    pub fn add_checked(&mut self, new_str: impl Into<String>) -> StringIdx {
        let s = new_str.into();

        let mut hasher = DefaultHasher::new();
        hasher.write(s.as_bytes());
        let hash = hasher.finish();

        if let Some(i) = self.map.get(&hash) {
            StringIdx::from(*i)
        } else {
            self.add(s)
        }
    }

    /// Unconditionally adds a string to this string table and returns the
    /// index of it
    pub fn add(&mut self, new_str: impl Into<String>) -> StringIdx {
        let s = new_str.into();

        let mut hasher = DefaultHasher::new();
        hasher.write(s.as_bytes());
        let hash = hasher.finish();

        let idx = self.contents.len();
        self.map.insert(hash, idx);
        self.size += (s.len() + 1) as u32;
        self.contents.push(s);

        StringIdx(idx as u32)
    }

    /// The size of this string table section in bytes
    pub fn size(&self) -> u32 {
        self.size
    }

    /// The index of this section's section header
    pub fn section_index(&self) -> SectionIdx {
        self.section_index
    }

    /// Parses a StringTable using the provided buffer, its expected size, and the section's section index
    pub fn parse(
        source: &mut BufferIterator,
        size: u32,
        section_index: SectionIdx,
    ) -> Result<Self, StringTableParseError> {
        let mut contents = Vec::new();
        let mut map = HashMap::new();
        let mut read = 0;

        while read < size {
            let mut b = Vec::new();

            while read < size {
                let c = source.next().ok_or(StringTableParseError::EOFError(
                    source.current_index(),
                    size,
                ))?;
                read += 1;

                if c == b'\0' {
                    break;
                }

                b.push(c);
            }

            let s = String::from_utf8(b)
                .map_err(|e| StringTableParseError::InvalidUtf8Error(contents.len(), e))?;

            let mut hasher = DefaultHasher::new();
            hasher.write(s.as_bytes());
            let hash = hasher.finish();

            let idx = contents.len();
            map.insert(hash, idx);
            contents.push(s);
        }

        Ok(Self {
            map,
            contents,
            section_index,
            size,
        })
    }

    /// Writes the binary representation of this string table to the provided buffer
    pub fn write(&self, buf: &mut impl WritableBuffer) {
        for string in self.contents.iter() {
            buf.write_bytes(string.as_bytes());
            buf.write(0);
        }
    }
}
