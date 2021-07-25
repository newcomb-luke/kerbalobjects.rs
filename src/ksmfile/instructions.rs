use crate::{errors::ReadError, FromBytes, Opcode, ReadResult, ToBytes};
use std::iter::Peekable;
use std::slice::Iter;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Instr {
    ZeroOp(Opcode),
    OneOp(Opcode, usize),
    TwoOp(Opcode, usize, usize),
}

impl Instr {
    fn write_operand(operand: usize, buf: &mut Vec<u8>, num_index_bytes: usize) {
        match num_index_bytes {
            1 => {
                (operand as u8).to_bytes(buf);
            }
            2 => {
                buf.extend_from_slice(&(operand as u16).to_be_bytes());
            }
            3 => {
                let slice = &(operand as u32).to_be_bytes();

                for i in 1..4 {
                    buf.push(slice[i]);
                }
            }
            4 => {
                buf.extend_from_slice(&(operand as u32).to_be_bytes());
            }
            _ => unreachable!(),
        }
    }

    fn read_operand(
        source: &mut Peekable<Iter<u8>>,
        debug: bool,
        num_index_bytes: usize,
    ) -> ReadResult<usize> {
        Ok(match num_index_bytes {
            1 => (u8::from_bytes(source, debug).map_err(|_| ReadError::OperandReadError))? as usize,
            2 => {
                let mut slice = [0u8; 2];
                for i in 0..2 {
                    if let Some(&byte) = source.next() {
                        slice[i] = byte;
                    } else {
                        return Err(ReadError::OperandReadError);
                    }
                }
                u16::from_be_bytes(slice) as usize
            }
            3 => {
                let first =
                    u8::from_bytes(source, debug).map_err(|_| ReadError::OperandReadError)? as u32;
                let second =
                    u8::from_bytes(source, debug).map_err(|_| ReadError::OperandReadError)? as u32;
                let third =
                    u8::from_bytes(source, debug).map_err(|_| ReadError::OperandReadError)? as u32;

                let mut full = third;
                full += second << 8;
                full += first << 16;

                (full) as usize
            }
            4 => {
                let mut slice = [0u8; 4];
                for i in 0..4 {
                    if let Some(&byte) = source.next() {
                        slice[i] = byte;
                    } else {
                        return Err(ReadError::OperandReadError);
                    }
                }
                u32::from_be_bytes(slice) as usize
            }
            _ => unreachable!(),
        })
    }

    pub fn to_bytes(&self, buf: &mut Vec<u8>, num_index_bytes: usize) {
        match self {
            Instr::ZeroOp(opcode) => {
                opcode.to_bytes(buf);
            }
            Instr::OneOp(opcode, op1) => {
                opcode.to_bytes(buf);
                Self::write_operand(*op1, buf, num_index_bytes);
            }
            Instr::TwoOp(opcode, op1, op2) => {
                opcode.to_bytes(buf);
                Self::write_operand(*op1, buf, num_index_bytes);
                Self::write_operand(*op2, buf, num_index_bytes);
            }
        }
    }

    pub fn from_bytes(
        source: &mut Peekable<Iter<u8>>,
        debug: bool,
        num_index_bytes: usize,
    ) -> ReadResult<Self> {
        let opcode = Opcode::from_bytes(source, debug)?;

        Ok(match opcode.num_operands() {
            0 => Instr::ZeroOp(opcode),
            1 => {
                let op1 = Instr::read_operand(source, debug, num_index_bytes)?;

                Instr::OneOp(opcode, op1)
            }
            2 => {
                let op1 = Instr::read_operand(source, debug, num_index_bytes)?;
                let op2 = Instr::read_operand(source, debug, num_index_bytes)?;

                Instr::TwoOp(opcode, op1, op2)
            }
            _ => unreachable!(),
        })
    }

    pub fn opcode(&self) -> Opcode {
        match self {
            Self::ZeroOp(opcode) => *opcode,
            Self::OneOp(opcode, _) => *opcode,
            Self::TwoOp(opcode, _, _) => *opcode,
        }
    }
}
