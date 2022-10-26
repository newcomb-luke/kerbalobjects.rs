//! kOS instructions as stored in a KSM file.
use crate::ksm::errors::InstrParseError;
use crate::ksm::sections::ArgIndex;
use crate::ksm::IntSize;
use crate::{BufferIterator, FromBytes, Opcode, ToBytes};

/// An instruction in a KSM file
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Instr {
    /// An instruction that takes no operands
    ZeroOp(Opcode),
    /// An instruction that takes one operand
    OneOp(Opcode, ArgIndex),
    /// An instruction that takes two operands
    TwoOp(Opcode, ArgIndex, ArgIndex),
}

impl Instr {
    /// Appends this instruction to the provided byte buffer.
    ///
    /// Requires the current number of argument index bytes, so that operands can be
    /// written using the correct bit width.
    pub fn write(&self, buf: &mut Vec<u8>, index_bytes: IntSize) {
        match self {
            Instr::ZeroOp(opcode) => {
                opcode.to_bytes(buf);
            }
            Instr::OneOp(opcode, op1) => {
                opcode.to_bytes(buf);
                op1.write(buf, index_bytes);
            }
            Instr::TwoOp(opcode, op1, op2) => {
                opcode.to_bytes(buf);
                op1.write(buf, index_bytes);
                op2.write(buf, index_bytes);
            }
        }
    }

    /// Parses bytes from the provided source, and converts them into an instruction
    ///
    /// Requires the number of argument index bytes that operands are expected to be.
    pub fn parse(
        source: &mut BufferIterator,
        index_bytes: IntSize,
    ) -> Result<Self, InstrParseError> {
        let opcode = Opcode::from_bytes(source)
            .map_err(|e| InstrParseError::OpcodeParseError(source.current_index(), e))?;

        Ok(match opcode.num_operands() {
            0 => Instr::ZeroOp(opcode),
            1 => {
                let op1 = ArgIndex::parse(source, index_bytes)
                    .map_err(|_| InstrParseError::MissingOperand(1))?;

                Instr::OneOp(opcode, op1)
            }
            2 => {
                let op1 = ArgIndex::parse(source, index_bytes)
                    .map_err(|_| InstrParseError::MissingOperand(1))?;
                let op2 = ArgIndex::parse(source, index_bytes)
                    .map_err(|_| InstrParseError::MissingOperand(2))?;

                Instr::TwoOp(opcode, op1, op2)
            }
            _ => unreachable!(),
        })
    }

    /// Returns how large this instruction will be if it is written with the provided
    /// number of argument index bytes
    pub fn size_bytes(&self, index_bytes: IntSize) -> usize {
        match self {
            Self::ZeroOp(_) => 1,
            Self::OneOp(_, _) => 1 + u8::from(index_bytes) as usize,
            Self::TwoOp(_, _, _) => 1 + 2 * (u8::from(index_bytes) as usize),
        }
    }

    /// Returns the opcode of this instruction.
    pub fn opcode(&self) -> Opcode {
        match self {
            Self::ZeroOp(opcode) => *opcode,
            Self::OneOp(opcode, _) => *opcode,
            Self::TwoOp(opcode, _, _) => *opcode,
        }
    }
}
