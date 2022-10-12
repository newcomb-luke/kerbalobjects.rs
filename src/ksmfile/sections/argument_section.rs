use crate::{FromBytes, KOSValue, ReadError, ReadResult, ToBytes};
use std::collections::HashMap;
use std::iter::Peekable;
use std::slice::Iter;

#[derive(Debug)]
pub struct ArgumentSection {
    num_index_bytes: usize,
    arguments: Vec<KOSValue>,
    kos_index_map: HashMap<usize, usize>,
    size_bytes: usize,
}

impl ArgumentSection {
    pub fn new() -> Self {
        ArgumentSection {
            num_index_bytes: 1,
            arguments: Vec::with_capacity(16),
            kos_index_map: HashMap::with_capacity(16),
            size_bytes: 3,
        }
    }

    pub fn num_index_bytes(&self) -> usize {
        self.num_index_bytes
    }

    pub fn add(&mut self, argument: KOSValue) -> usize {
        let size = argument.size_bytes();
        let index = self.arguments.len();
        let kos_index = self.size_bytes;

        self.arguments.push(argument);
        self.kos_index_map.insert(self.size_bytes, index);
        self.size_bytes += size;

        kos_index
    }

    pub fn get(&self, index: usize) -> Option<&KOSValue> {
        let vec_index = *self.kos_index_map.get(&index)?;

        self.arguments.get(vec_index)
    }

    pub fn arguments(&self) -> Iter<KOSValue> {
        self.arguments.iter()
    }

    pub fn size_bytes(&self) -> usize {
        self.size_bytes
    }

    pub fn recalculate_index_bytes(&mut self) {
        self.num_index_bytes = if self.size_bytes <= 256 {
            1
        } else if self.size_bytes <= 65535 {
            2
        } else if self.size_bytes <= 1677215 {
            3
        } else {
            4
        };
    }
}

impl Default for ArgumentSection {
    fn default() -> Self {
        Self::new()
    }
}

impl ToBytes for ArgumentSection {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        b'%'.to_bytes(buf);
        b'A'.to_bytes(buf);
        (self.num_index_bytes as u8).to_bytes(buf);

        for argument in self.arguments.iter() {
            argument.to_bytes(buf);
        }
    }
}

impl FromBytes for ArgumentSection {
    fn from_bytes(source: &mut Peekable<Iter<u8>>) -> ReadResult<Self>
    where
        Self: Sized,
    {
        #[cfg(feature = "print_debug")]
        {
            println!("Reading ArgumentSection");
        }

        let header = u16::from_bytes(source).map_err(|_| ReadError::MissingArgumentSectionError)?;

        // %A in hex, little-endian
        if header != 0x4125 {
            return Err(ReadError::ExpectedArgumentSectionError(header));
        }

        let num_index_bytes =
            u8::from_bytes(source).map_err(|_| ReadError::NumIndexBytesReadError)? as usize;

        if !(1..=4).contains(&num_index_bytes) {
            return Err(ReadError::InvalidNumIndexBytesError(num_index_bytes));
        }

        #[cfg(feature = "print_debug")]
        {
            println!("\tNumber of index bytes: {}", num_index_bytes);
        }

        let mut arg_section = ArgumentSection {
            num_index_bytes,
            arguments: Vec::new(),
            kos_index_map: HashMap::new(),
            size_bytes: 3,
        };

        loop {
            if let Some(next) = source.peek() {
                if **next == b'%' {
                    break;
                } else {
                    let argument = KOSValue::from_bytes(source)?;

                    #[cfg(feature = "print_debug")]
                    {
                        println!("\tRead argument: {:?}", argument);
                    }

                    arg_section.add(argument);
                }
            } else {
                return Err(ReadError::ArgumentSectionReadError);
            }
        }

        Ok(arg_section)
    }
}
