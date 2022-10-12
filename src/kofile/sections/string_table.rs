use crate::kofile::sections::SectionIndex;
use crate::kofile::SectionFromBytes;
use crate::{ReadError, ReadResult, ToBytes};
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::iter::Peekable;
use std::slice::Iter;

#[derive(Debug)]
pub struct StringTable {
    hashes: Vec<u64>,
    contents: Vec<String>,
    section_index: usize,
    size: usize,
}

impl SectionIndex for StringTable {
    fn section_index(&self) -> usize {
        self.section_index
    }
}

impl StringTable {
    pub fn new(amount: usize, section_index: usize) -> Self {
        let empty = String::new();
        let mut hasher = DefaultHasher::new();
        hasher.write(empty.as_bytes());
        let hash = hasher.finish();

        let mut contents = Vec::with_capacity(amount);
        contents.push(empty);
        let mut hashes = Vec::with_capacity(amount);
        hashes.push(hash);

        StringTable {
            hashes,
            contents,
            section_index,
            size: 1,
        }
    }

    pub fn get(&self, index: usize) -> Option<&String> {
        self.contents.get(index)
    }

    pub fn find(&self, s: &str) -> Option<usize> {
        let mut hasher = DefaultHasher::new();
        hasher.write(s.as_bytes());
        let hash = hasher.finish();

        self.hashes.iter().position(|item| *item == hash)
    }

    pub fn strings(&self) -> Iter<String> {
        self.contents.iter()
    }

    pub fn add_checked(&mut self, new_str: &str) -> usize {
        match self.find(new_str) {
            Some(index) => index,
            None => self.add(new_str),
        }
    }

    pub fn add(&mut self, new_str: &str) -> usize {
        let mut hasher = DefaultHasher::new();
        hasher.write(new_str.as_bytes());
        let hash = hasher.finish();

        self.hashes.push(hash);
        self.size += new_str.len() + 1;
        self.contents.push(String::from(new_str));

        self.contents.len() - 1
    }

    pub fn size(&self) -> u32 {
        self.size as u32
    }
}

impl SectionFromBytes for StringTable {
    fn from_bytes(
        source: &mut Peekable<Iter<u8>>,
        size: usize,
        section_index: usize,
    ) -> ReadResult<Self> {
        let mut contents = Vec::new();
        let mut hashes = Vec::new();
        let mut read = 0;

        while read < size {
            let mut b = Vec::new();

            while read < size {
                let c = *source.next().ok_or(ReadError::StringTableReadError)?;
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
