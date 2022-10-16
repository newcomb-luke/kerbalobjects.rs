//! kOS instructions as stored in a KO file.
use crate::errors::ReadError;
use crate::{FileIterator, FromBytes, Opcode, ReadResult, ToBytes};

/// An instruction in a KO file
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Instr {
    /// An instruction that takes no operands
    ZeroOp(Opcode),
    /// An instruction that takes one operand
    OneOp(Opcode, usize),
    /// An instruction that takes two operands
    TwoOp(Opcode, usize, usize),
}

impl Instr {
    /// Returns the size in bytes that this instruction would take up in a KO file
    pub fn size_bytes(&self) -> usize {
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
}

impl ToBytes for Instr {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        match self {
            Self::ZeroOp(opcode) => {
                opcode.to_bytes(buf);
            }
            Self::OneOp(opcode, operand) => {
                opcode.to_bytes(buf);
                (*operand as u32).to_bytes(buf);
            }
            Self::TwoOp(opcode, operand1, operand2) => {
                opcode.to_bytes(buf);
                (*operand1 as u32).to_bytes(buf);
                (*operand2 as u32).to_bytes(buf);
            }
        }
    }
}

// TODO: Docs and rewrite
impl FromBytes for Instr {
    fn from_bytes(source: &mut FileIterator) -> ReadResult<Self> {
        let opcode = Opcode::from_bytes(source)?;

        Ok(match opcode.num_operands() {
            0 => Instr::ZeroOp(opcode),
            1 => {
                let op1 = u32::from_bytes(source).map_err(|_| ReadError::OperandReadError)?;
                Instr::OneOp(opcode, op1 as usize)
            }
            _ => {
                let op1 = u32::from_bytes(source).map_err(|_| ReadError::OperandReadError)?;
                let op2 = u32::from_bytes(source).map_err(|_| ReadError::OperandReadError)?;
                Instr::TwoOp(opcode, op1 as usize, op2 as usize)
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
        let instr = Instr::OneOp(Opcode::Push, 1);
        let mut buf = Vec::with_capacity(4);

        instr.to_bytes(&mut buf);

        assert_eq!(buf, vec![0x4e, 0x01, 0x00, 0x00, 0x00]);
    }
}
