//! A module describing a function section in a Kerbal Object file
use crate::ko::errors::FunctionSectionParseError;
use crate::ko::{Instr, SectionIdx};
use crate::{BufferIterator, WritableBuffer};
use std::slice::Iter;

/// A wrapper type that represents an index into a function section of a KO file.
///
/// This type implements From<usize> and usize implements From<InstrIdx>, but this is provided
/// so that it takes 1 extra step to convert raw integers into InstrIdx which could stop potential
/// logical bugs.
///
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct InstrIdx(u32);

impl From<usize> for InstrIdx {
    fn from(i: usize) -> Self {
        Self(i as u32)
    }
}

impl From<u8> for InstrIdx {
    fn from(i: u8) -> Self {
        Self(i as u32)
    }
}

impl From<u16> for InstrIdx {
    fn from(i: u16) -> Self {
        Self(i as u32)
    }
}

impl From<u32> for InstrIdx {
    fn from(i: u32) -> Self {
        Self(i)
    }
}

impl From<InstrIdx> for u32 {
    fn from(instr_idx: InstrIdx) -> Self {
        instr_idx.0
    }
}

impl From<InstrIdx> for usize {
    fn from(instr_idx: InstrIdx) -> Self {
        instr_idx.0 as usize
    }
}

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
    section_index: SectionIdx,
}

impl FuncSection {
    /// Creates a new function section, with the provided section index
    pub fn new(section_index: SectionIdx) -> Self {
        Self {
            instructions: Vec::new(),
            size: 0,
            section_index,
        }
    }

    /// Creates a new data section with the provided section index, with
    /// internal data structures pre-allocated for the provided amount of instructions
    pub fn with_capacity(amount: usize, section_index: SectionIdx) -> Self {
        Self {
            instructions: Vec::with_capacity(amount),
            size: 0,
            section_index,
        }
    }

    /// Adds an instruction to this function section, returning the index of the instruction
    /// into this function section.
    pub fn add(&mut self, instr: Instr) -> InstrIdx {
        let index = self.instructions.len();

        self.size += instr.size_bytes() as u32;
        self.instructions.push(instr);

        InstrIdx::from(index)
    }

    /// The size of this function section in bytes
    pub fn size(&self) -> u32 {
        self.size
    }

    /// Gets the instruction that resides at the provided index, or None if none exists
    pub fn get(&self, index: InstrIdx) -> Option<&Instr> {
        self.instructions.get(usize::from(index))
    }

    /// Returns an iterator over all of the instructions in this section
    pub fn instructions(&self) -> Iter<Instr> {
        self.instructions.iter()
    }

    /// The index of this section's section header
    pub fn section_index(&self) -> SectionIdx {
        self.section_index
    }

    /// Parses a function section from the provided byte buffer
    pub fn parse(
        source: &mut BufferIterator,
        size: u32,
        section_index: SectionIdx,
    ) -> Result<Self, FunctionSectionParseError> {
        let mut read_bytes = 0;
        let mut instructions = Vec::new();

        while read_bytes < size {
            let instr = Instr::parse(source).map_err(|e| {
                FunctionSectionParseError::InstrParseError(source.current_index(), e)
            })?;
            read_bytes += instr.size_bytes();

            instructions.push(instr);
        }

        Ok(Self {
            instructions,
            size,
            section_index,
        })
    }

    /// Converts this function section to its binary representation and appends it to the provided buffer
    pub fn write(&self, buf: &mut impl WritableBuffer) {
        for instr in self.instructions.iter() {
            instr.write(buf);
        }
    }
}
