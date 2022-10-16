//! A module describing a function section in a Kerbal Object file
use crate::ko::sections::SectionIndex;
use crate::ko::{Instr, SectionFromBytes};
use crate::{FileIterator, FromBytes, ReadResult, ToBytes};
use std::slice::Iter;

/// A function section in a KerbalObject file.
///
/// This section stores a series
/// of instructions which define a function which can be included in the final binary produced by
/// the linker.
///
#[derive(Debug)]
pub struct FuncSection {
    instructions: Vec<Instr>,
    size: u32,
    /// The index of this section's section header
    pub section_index: usize,
}

impl SectionIndex for FuncSection {
    fn section_index(&self) -> usize {
        self.section_index
    }
}

impl FuncSection {
    /// Creates a new function section, with the provided section index
    pub fn new(section_index: usize) -> Self {
        Self {
            instructions: Vec::new(),
            size: 0,
            section_index,
        }
    }

    /// Creates a new data section with the provided section index, with
    /// internal data structures pre-allocated for the provided amount of items
    pub fn with_capacity(amount: usize, section_index: usize) -> Self {
        Self {
            instructions: Vec::with_capacity(amount),
            size: 0,
            section_index,
        }
    }

    /// Adds an instruction to this function section, returning the index of the instruction
    /// into this function section.
    pub fn add(&mut self, instr: Instr) -> usize {
        let index = self.instructions.len();

        self.size += instr.size_bytes() as u32;
        self.instructions.push(instr);

        index
    }

    /// The size of this function section in bytes
    pub fn size(&self) -> u32 {
        self.size
    }

    /// Gets the instruction that resides at the provided index, or None if none exists
    pub fn get(&self, index: usize) -> Option<&Instr> {
        self.instructions.get(index)
    }

    /// Returns an iterator over all of the instructions in this section
    pub fn instructions(&self) -> Iter<Instr> {
        self.instructions.iter()
    }
}

impl SectionFromBytes for FuncSection {
    fn from_bytes(
        source: &mut FileIterator,
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
            size: size as u32,
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
