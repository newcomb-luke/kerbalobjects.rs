use crate::kofile::sections::SectionIndex;
use crate::kofile::symbols::ReldEntry;
use crate::kofile::SectionFromBytes;
use crate::{FromBytes, ReadResult, ToBytes};
use std::iter::Peekable;
use std::slice::Iter;

pub struct ReldSection {
    entries: Vec<ReldEntry>,
    size: u32,
    section_index: usize,
}

impl SectionIndex for ReldSection {
    fn section_index(&self) -> usize {
        self.section_index
    }
}

impl ReldSection {
    pub fn new(amount: usize, section_index: usize) -> Self {
        ReldSection {
            entries: Vec::with_capacity(amount),
            size: 0,
            section_index,
        }
    }

    pub fn add(&mut self, entry: ReldEntry) -> usize {
        self.size += entry.size_bytes();
        self.entries.push(entry);
        self.entries.len() - 1
    }

    pub fn get(&self, index: usize) -> Option<&ReldEntry> {
        self.entries.get(index)
    }

    pub fn entries(&self) -> Iter<ReldEntry> {
        self.entries.iter()
    }

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
        source: &mut Peekable<Iter<u8>>,
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
