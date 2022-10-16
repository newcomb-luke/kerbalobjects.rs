//! kOS instructions as stored in a KSM file.
use crate::ksm::sections::{ArgIndex, NumArgIndexBytes};
use crate::{FileIterator, FromBytes, Opcode, ReadResult, ToBytes};

/// An instruction in a KSM file
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
    pub fn to_bytes(&self, buf: &mut Vec<u8>, num_index_bytes: NumArgIndexBytes) {
        match self {
            Instr::ZeroOp(opcode) => {
                opcode.to_bytes(buf);
            }
            Instr::OneOp(opcode, op1) => {
                opcode.to_bytes(buf);
                op1.to_bytes(buf, num_index_bytes);
            }
            Instr::TwoOp(opcode, op1, op2) => {
                opcode.to_bytes(buf);
                op1.to_bytes(buf, num_index_bytes);
                op2.to_bytes(buf, num_index_bytes);
            }
        }
    }

    /// Reads bytes from the provided source, and converts them into an instruction
    ///
    /// Requires the number of argument index bytes that operands are expected to be.
    pub fn from_bytes(
        source: &mut FileIterator,
        num_index_bytes: NumArgIndexBytes,
    ) -> ReadResult<Self> {
        let opcode = Opcode::from_bytes(source)?;

        Ok(match opcode.num_operands() {
            0 => Instr::ZeroOp(opcode),
            1 => {
                let op1 = ArgIndex::from_bytes(source, num_index_bytes)?;

                Instr::OneOp(opcode, op1)
            }
            2 => {
                let op1 = ArgIndex::from_bytes(source, num_index_bytes)?;
                let op2 = ArgIndex::from_bytes(source, num_index_bytes)?;

                Instr::TwoOp(opcode, op1, op2)
            }
            _ => unreachable!(),
        })
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
