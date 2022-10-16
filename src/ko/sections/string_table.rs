//! A module describing a string table section in a Kerbal Object file
use crate::ko::sections::SectionIndex;
use crate::ko::SectionFromBytes;
use crate::{FileIterator, ReadError, ReadResult, ToBytes};
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::slice::Iter;

/// A string table section in a KerbalObject file.
///
/// A module describing a string table section in a Kerbal Object file. This section stores
/// a list of strings, which other Kerbal Object constructs can index into.
///
/// Each section contains as its first index (index 0) an empty string.
///
#[derive(Debug)]
pub struct StringTable {
    hashes: Vec<u64>,
    contents: Vec<String>,
    /// The index of this section's section header
    pub section_index: usize,
    size: usize,
}

impl SectionIndex for StringTable {
    fn section_index(&self) -> usize {
        self.section_index
    }
}

impl StringTable {
    /// Creates a new string table section with the provided section index
    pub fn new(section_index: usize) -> Self {
        let empty = String::new();
        let mut hasher = DefaultHasher::new();
        hasher.write(empty.as_bytes());
        let hash = hasher.finish();

        let contents = vec![empty];
        let hashes = vec![hash];

        Self {
            hashes,
            contents,
            section_index,
            size: 1,
        }
    }

    /// Creates a new string table section with the provided section index, with
    /// internal data structures pre-allocated for the provided amount of bytes
    pub fn with_capacity(amount: usize, section_index: usize) -> Self {
        let empty = String::new();
        let mut hasher = DefaultHasher::new();
        hasher.write(empty.as_bytes());
        let hash = hasher.finish();

        let mut contents = Vec::with_capacity(amount);
        contents.push(empty);
        let mut hashes = Vec::with_capacity(amount);
        hashes.push(hash);

        Self {
            hashes,
            contents,
            section_index,
            size: 1,
        }
    }

    /// Gets the string at the index into this string table, or None if it doesn't
    /// exist
    pub fn get(&self, index: usize) -> Option<&String> {
        self.contents.get(index)
    }

    /// Finds a given string in this string table and returns the index
    /// if it does exist, or None if it doesn't
    pub fn find(&self, s: &str) -> Option<usize> {
        let mut hasher = DefaultHasher::new();
        hasher.write(s.as_bytes());
        let hash = hasher.finish();

        self.hashes.iter().position(|item| *item == hash)
    }

    /// Returns an iterator over all of the strings contained in this string table
    pub fn strings(&self) -> Iter<String> {
        self.contents.iter()
    }

    /// Adds a string to this string table, but checks if it is already in this table.
    /// Returns the index of the string, if it already existed, the index is just
    /// found and returned. If it doesn't, it is inserted and the new index is returned.
    pub fn add_checked(&mut self, new_str: &str) -> usize {
        match self.find(new_str) {
            Some(index) => index,
            None => self.add(new_str),
        }
    }

    /// Unconditionally adds a string to this string table and returns the
    /// index of it
    pub fn add(&mut self, new_str: &str) -> usize {
        let mut hasher = DefaultHasher::new();
        hasher.write(new_str.as_bytes());
        let hash = hasher.finish();

        self.hashes.push(hash);
        self.size += new_str.len() + 1;
        self.contents.push(String::from(new_str));

        self.contents.len() - 1
    }

    /// The size of this string table section in bytes
    pub fn size(&self) -> u32 {
        self.size as u32
    }
}

impl SectionFromBytes for StringTable {
    fn from_bytes(
        source: &mut FileIterator,
        size: usize,
        section_index: usize,
    ) -> ReadResult<Self> {
        let mut contents = Vec::new();
        let mut hashes = Vec::new();
        let mut read = 0;

        while read < size {
            let mut b = Vec::new();

            while read < size {
                let c = source.next().ok_or(ReadError::StringTableReadError)?;
                read += 1;

                if c == b'\0' {
                    break;
                }

                b.push(c);
            }

            let s = String::from_utf8(b).map_err(|_| ReadError::StringTableReadError)?;

            let mut hasher = DefaultHasher::new();
            hasher.write(s.as_bytes());
            let hash = hasher.finish();

            hashes.push(hash);
            contents.push(s);
        }

        #[cfg(feature = "print_debug")]
        {
            println!("String table strings: ");

            for s in contents.iter() {
                println!("\t{}", s);
            }
        }

        Ok(StringTable {
            hashes,
            contents,
            section_index,
            size,
        })
    }
}

impl ToBytes for StringTable {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        for string in self.contents.iter() {
            buf.extend_from_slice(string.as_bytes());
            buf.push(0);
        }
    }
}
