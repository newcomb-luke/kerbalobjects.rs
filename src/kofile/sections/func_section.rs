use crate::kofile::sections::SectionIndex;
use crate::kofile::{Instr, SectionFromBytes};
use crate::{FromBytes, ReadResult, ToBytes};
use std::iter::Peekable;
use std::slice::Iter;

#[derive(Debug)]
pub struct FuncSection {
    instructions: Vec<Instr>,
    size: usize,
    section_index: usize,
}

impl SectionIndex for FuncSection {
    fn section_index(&self) -> usize {
        self.section_index
    }
}

impl FuncSection {
    pub fn new(amount: usize, section_index: usize) -> Self {
        FuncSection {
            instructions: Vec::with_capacity(amount),
            size: 0,
            section_index,
        }
    }

    pub fn add(&mut self, instr: Instr) -> usize {
        let index = self.instructions.len();

        self.size += instr.size_bytes();
        self.instructions.push(instr);

        index
    }

    pub fn get(&self, index: usize) -> Option<&Instr> {
        self.instructions.get(index)
    }

    pub fn size(&self) -> u32 {
        self.size as u32
    }

    pub fn instructions(&self) -> Iter<Instr> {
        self.instructions.iter()
    }
}

impl SectionFromBytes for FuncSection {
    fn from_bytes(
        source: &mut Peekable<Iter<u8>>,
        size: usize,
        section_index: usize,
    ) -> ReadResult<Self> {
        let mut new_size = 0;
        let mut instructions = Vec::new();

        while new_size < size {
            let instr = Instr::from_bytes(source)?;
            new_size += instr.size_bytes();

            instructions.push(instr);
        }

        Ok(FuncSection {
            instructions,
            size,
            section_index,
        })
    }
}

impl ToBytes for FuncSection {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        for instr in self.instructions.iter() {
            instr.to_bytes(buf);
        }
    }
}
