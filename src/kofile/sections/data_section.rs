use crate::kofile::sections::SectionIndex;
use crate::kofile::SectionFromBytes;
use crate::{FromBytes, KOSValue, ReadResult, ToBytes};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::iter::Peekable;
use std::slice::Iter;

#[derive(Debug)]
pub struct DataSection {
    hashes: Vec<u64>,
    data: Vec<KOSValue>,
    size: usize,
    section_index: usize,
}

impl SectionIndex for DataSection {
    fn section_index(&self) -> usize {
        self.section_index
    }
}

impl DataSection {
    pub fn new(amount: usize, section_index: usize) -> Self {
        DataSection {
            hashes: Vec::with_capacity(amount),
            data: Vec::with_capacity(amount),
            size: 0,
            section_index,
        }
    }

    pub fn find(&self, value: &KOSValue) -> Option<usize> {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        let hash = hasher.finish();

        self.hashes.iter().position(|item| *item == hash)
    }

    pub fn add_checked(&mut self, value: KOSValue) -> usize {
        match self.find(&value) {
            Some(index) => index,
            None => self.add(value),
        }
    }

    pub fn add(&mut self, value: KOSValue) -> usize {
        self.size += value.size_bytes();

        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        let hash = hasher.finish();

        self.hashes.push(hash);
        self.data.push(value);

        self.data.len() - 1
    }

    pub fn get(&self, index: usize) -> Option<&KOSValue> {
        self.data.get(index)
    }

    pub fn data(&self) -> Iter<KOSValue> {
        self.data.iter()
    }

    pub fn size(&self) -> u32 {
        self.size as u32
    }
}

impl SectionFromBytes for DataSection {
    fn from_bytes(
        source: &mut Peekable<Iter<u8>>,
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
            size,
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
