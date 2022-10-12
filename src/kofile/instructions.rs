use crate::{FromBytes, Opcode, ReadResult, ToBytes};
use std::iter::Peekable;
use std::slice::Iter;

use crate::errors::ReadError;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Instr {
    ZeroOp(Opcode),
    OneOp(Opcode, usize),
    TwoOp(Opcode, usize, usize),
}

impl Instr {
    pub fn size_bytes(&self) -> usize {
        match &self {
            Self::ZeroOp(_) => 1,
            Self::OneOp(_, _) => 5,
            Self::TwoOp(_, _, _) => 9,
        }
    }

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
    fn from_bytes(source: &mut Peekable<Iter<u8>>) -> ReadResult<Self> {
        let opcode = Opcode::from_bytes(source)?;

        Ok(match opcode.num_operands() {
            0 => Instr::ZeroOp(opcode),
            1 => {
                let op1 =
                    u32::from_bytes(source).map_err(|_| ReadError::OperandReadError)?;
                Instr::OneOp(opcode, op1 as usize)
            }
            _ => {
                let op1 =
                    u32::from_bytes(source).map_err(|_| ReadError::OperandReadError)?;
                let op2 =
                    u32::from_bytes(source).map_err(|_| ReadError::OperandReadError)?;
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
