use crate::{errors::ReadError, FromBytes, Opcode, ReadResult, ToBytes};
use std::iter::Peekable;
use std::slice::Iter;

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
                (operand as u16).to_bytes(buf);
            }
            3 => {
                let converted = operand as u32;
                let upper = (converted >> 16) as u8;
                let lower = (converted & 0x0000ffff) as u16;

                upper.to_bytes(buf);
                lower.to_bytes(buf);
            }
            4 => {
                (operand as u32).to_bytes(buf);
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
                (u16::from_bytes(source, debug).map_err(|_| ReadError::OperandReadError))? as usize
            }
            3 => {
                let mut upper =
                    u8::from_bytes(source, debug).map_err(|_| ReadError::OperandReadError)? as u32;
                upper = upper << 24;
                let lower =
                    u16::from_bytes(source, debug).map_err(|_| ReadError::OperandReadError)? as u32;

                (upper | lower) as usize
            }
            4 => {
                (u32::from_bytes(source, debug).map_err(|_| ReadError::OperandReadError))? as usize
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
}
