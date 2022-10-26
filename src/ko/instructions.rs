//! kOS instructions as stored in a KO file.
use crate::ko::errors::InstrParseError;
use crate::ko::sections::DataIdx;
use crate::{BufferIterator, FromBytes, Opcode, ToBytes, WritableBuffer};

/// An instruction in a KO file
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Instr {
    /// An instruction that takes no operands
    ZeroOp(Opcode),
    /// An instruction that takes one operand
    OneOp(Opcode, DataIdx),
    /// An instruction that takes two operands
    TwoOp(Opcode, DataIdx, DataIdx),
}

impl Instr {
    /// Returns the size in bytes that this instruction would take up in a KO file
    pub fn size_bytes(&self) -> u32 {
        match &self {
            Self::ZeroOp(_) => 1,
            Self::OneOp(_, _) => 5,
            Self::TwoOp(_, _, _) => 9,
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

    /// Converts this instruction to its binary representation and appends it to the provided buffer
    pub fn write(&self, buf: &mut impl WritableBuffer) {
        match self {
            Self::ZeroOp(opcode) => {
                opcode.to_bytes(buf);
            }
            Self::OneOp(opcode, operand) => {
                opcode.to_bytes(buf);
                u32::from(*operand).to_bytes(buf);
            }
            Self::TwoOp(opcode, operand1, operand2) => {
                opcode.to_bytes(buf);
                u32::from(*operand1).to_bytes(buf);
                u32::from(*operand2).to_bytes(buf);
            }
        }
    }

    /// Parses a KO file instruction from the provided buffer
    pub fn parse(source: &mut BufferIterator) -> Result<Self, InstrParseError> {
        let opcode = Opcode::from_bytes(source)
            .map_err(|e| InstrParseError::OpcodeParseError(source.current_index(), e))?;

        Ok(match opcode.num_operands() {
            0 => Instr::ZeroOp(opcode),
            1 => {
                let op1 = DataIdx::from(
                    u32::from_bytes(source).map_err(|_| InstrParseError::MissingOperand(1))?,
                );
                Instr::OneOp(opcode, op1)
            }
            _ => {
                let op1 = DataIdx::from(
                    u32::from_bytes(source).map_err(|_| InstrParseError::MissingOperand(1))?,
                );
                let op2 = DataIdx::from(
                    u32::from_bytes(source).map_err(|_| InstrParseError::MissingOperand(2))?,
                );
                Instr::TwoOp(opcode, op1, op2)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_push() {
        let mnemonic = "push";
        let opcode: Opcode = mnemonic.into();

        assert_eq!(opcode, Opcode::Push);
    }

    #[test]
    fn write_push() {
        let instr = Instr::OneOp(Opcode::Push, DataIdx::from(1u32));
        let mut buf = Vec::with_capacity(4);

        instr.write(&mut buf);

        assert_eq!(buf, vec![0x4e, 0x01, 0x00, 0x00, 0x00]);
    }
}
