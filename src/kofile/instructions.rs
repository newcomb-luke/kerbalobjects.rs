use crate::{FromBytes, ToBytes};
use std::slice::Iter;

use super::symbols::KOSymbol;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Operand<'sym, 'name> {
    symbol: &'sym KOSymbol<'name>,
    index: usize,
}

impl<'sym, 'name> Operand<'sym, 'name> {
    pub fn new(symbol: &'sym KOSymbol<'name>, index: usize) -> Self {
        Operand { symbol, index }
    }

    pub fn symbol(&self) -> &KOSymbol {
        self.symbol
    }

    pub fn index(&self) -> usize {
        self.index
    }
}

pub enum Instr<'sym, 'name> {
    ZeroOp(Opcode),
    OneOp(Opcode, Operand<'sym, 'name>),
    TwoOp(Opcode, Operand<'sym, 'name>, Operand<'sym, 'name>),
}

impl<'sym, 'name> Instr<'sym, 'name> {
    pub fn size_bytes(&self) -> usize {
        match &self {
            Self::ZeroOp(_) => 1,
            Self::OneOp(_, _) => 5,
            Self::TwoOp(_, _, _) => 9,
        }
    }
}

impl<'sym, 'name> ToBytes for Instr<'sym, 'name> {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        match self {
            Self::ZeroOp(opcode) => {
                opcode.to_bytes(buf);
            }
            Self::OneOp(opcode, operand) => {
                opcode.to_bytes(buf);
                (operand.index as u32).to_bytes(buf);
            }
            Self::TwoOp(opcode, operand1, operand2) => {
                opcode.to_bytes(buf);
                (operand1.index as u32).to_bytes(buf);
                (operand2.index as u32).to_bytes(buf);
            }
        }
    }
}

impl<'sym, 'name> FromBytes for Instr<'sym, 'name> {
    fn from_bytes(source: &mut Iter<u8>) -> Self {
        todo!();
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Opcode {
    Bogus,
    Eof,
    Eop,
    Nop,
    Sto,
    Uns,
    Gmb,
    Smb,
    Gidx,
    Sidx,
    Bfa,
    Jmp,
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Cgt,
    Clt,
    Cge,
    Cle,
    Ceq,
    Cne,
    Neg,
    Bool,
    Not,
    And,
    Or,
    Call,
    Ret,
    Push,
    Pop,
    Dup,
    Swap,
    Eval,
    Addt,
    Rmvt,
    Wait,
    Gmet,
    Stol,
    Stog,
    Bscp,
    Escp,
    Stoe,
    Phdl,
    Btr,
    Exst,
    Argb,
    Targ,
    Tcan,
    Prl,
    Pdrl,
    Lbrt,

    Pushv,
}

impl From<u8> for Opcode {
    fn from(byte: u8) -> Self {
        match byte {
            0x31 => Opcode::Eof,
            0x32 => Opcode::Eop,
            0x33 => Opcode::Nop,
            0x34 => Opcode::Sto,
            0x35 => Opcode::Uns,
            0x36 => Opcode::Gmb,
            0x37 => Opcode::Smb,
            0x38 => Opcode::Gidx,
            0x39 => Opcode::Sidx,
            0x3a => Opcode::Bfa,
            0x3b => Opcode::Jmp,
            0x3c => Opcode::Add,
            0x3d => Opcode::Sub,
            0x3e => Opcode::Mul,
            0x3f => Opcode::Div,
            0x40 => Opcode::Pow,
            0x41 => Opcode::Cgt,
            0x42 => Opcode::Clt,
            0x43 => Opcode::Cge,
            0x44 => Opcode::Cle,
            0x45 => Opcode::Ceq,
            0x46 => Opcode::Cne,
            0x47 => Opcode::Neg,
            0x48 => Opcode::Bool,
            0x49 => Opcode::Not,
            0x4a => Opcode::And,
            0x4b => Opcode::Or,
            0x4c => Opcode::Call,
            0x4d => Opcode::Ret,
            0x4e => Opcode::Push,
            0x4f => Opcode::Pop,
            0x50 => Opcode::Dup,
            0x51 => Opcode::Swap,
            0x52 => Opcode::Eval,
            0x53 => Opcode::Addt,
            0x54 => Opcode::Rmvt,
            0x55 => Opcode::Wait,
            0x57 => Opcode::Gmet,
            0x58 => Opcode::Stol,
            0x59 => Opcode::Stog,
            0x5a => Opcode::Bscp,
            0x5b => Opcode::Escp,
            0x5c => Opcode::Stoe,
            0x5d => Opcode::Phdl,
            0x5e => Opcode::Btr,
            0x5f => Opcode::Exst,
            0x60 => Opcode::Argb,
            0x61 => Opcode::Targ,
            0x62 => Opcode::Tcan,

            0xce => Opcode::Prl,
            0xcd => Opcode::Pdrl,
            0xf0 => Opcode::Lbrt,

            0xfa => Opcode::Pushv,

            _ => Opcode::Bogus,
        }
    }
}

impl From<Opcode> for u8 {
    fn from(opcode: Opcode) -> Self {
        match opcode {
            Opcode::Bogus => 0x00,
            Opcode::Eof => 0x31,
            Opcode::Eop => 0x32,
            Opcode::Nop => 0x33,
            Opcode::Sto => 0x34,
            Opcode::Uns => 0x35,
            Opcode::Gmb => 0x36,
            Opcode::Smb => 0x37,
            Opcode::Gidx => 0x38,
            Opcode::Sidx => 0x39,
            Opcode::Bfa => 0x3a,
            Opcode::Jmp => 0x3b,
            Opcode::Add => 0x3c,
            Opcode::Sub => 0x3d,
            Opcode::Mul => 0x3e,
            Opcode::Div => 0x3f,
            Opcode::Pow => 0x40,
            Opcode::Cgt => 0x41,
            Opcode::Clt => 0x42,
            Opcode::Cge => 0x43,
            Opcode::Cle => 0x44,
            Opcode::Ceq => 0x45,
            Opcode::Cne => 0x46,
            Opcode::Neg => 0x47,
            Opcode::Bool => 0x48,
            Opcode::Not => 0x49,
            Opcode::And => 0x4a,
            Opcode::Or => 0x4b,
            Opcode::Call => 0x4c,
            Opcode::Ret => 0x4d,
            Opcode::Push => 0x4e,
            Opcode::Pop => 0x4f,
            Opcode::Dup => 0x50,
            Opcode::Swap => 0x51,
            Opcode::Eval => 0x52,
            Opcode::Addt => 0x53,
            Opcode::Rmvt => 0x54,
            Opcode::Wait => 0x55,
            Opcode::Gmet => 0x57,
            Opcode::Stol => 0x58,
            Opcode::Stog => 0x59,
            Opcode::Bscp => 0x5a,
            Opcode::Escp => 0x5b,
            Opcode::Stoe => 0x5c,
            Opcode::Phdl => 0x5d,
            Opcode::Btr => 0x5e,
            Opcode::Exst => 0x5f,
            Opcode::Argb => 0x60,
            Opcode::Targ => 0x61,
            Opcode::Tcan => 0x62,

            Opcode::Prl => 0xce,
            Opcode::Pdrl => 0xcd,
            Opcode::Lbrt => 0xf0,

            Opcode::Pushv => 0xfa,
        }
    }
}

impl From<&str> for Opcode {
    fn from(s: &str) -> Self {
        match s {
            "eof" => Opcode::Eof,
            "eop" => Opcode::Eop,
            "nop" => Opcode::Nop,
            "sto" => Opcode::Sto,
            "uns" => Opcode::Uns,
            "gmb" => Opcode::Gmb,
            "smb" => Opcode::Smb,
            "gidx" => Opcode::Gidx,
            "sidx" => Opcode::Sidx,
            "bfa" => Opcode::Bfa,
            "jmp" => Opcode::Jmp,
            "add" => Opcode::Add,
            "sub" => Opcode::Sub,
            "mul" => Opcode::Mul,
            "div" => Opcode::Div,
            "pow" => Opcode::Pow,
            "cgt" => Opcode::Cgt,
            "clt" => Opcode::Clt,
            "cge" => Opcode::Cge,
            "cle" => Opcode::Cle,
            "ceq" => Opcode::Ceq,
            "cne" => Opcode::Cne,
            "neg" => Opcode::Neg,
            "bool" => Opcode::Bool,
            "not" => Opcode::Not,
            "and" => Opcode::And,
            "or" => Opcode::Or,
            "call" => Opcode::Call,
            "ret" => Opcode::Ret,
            "push" => Opcode::Push,
            "pop" => Opcode::Pop,
            "dup" => Opcode::Dup,
            "swap" => Opcode::Swap,
            "eval" => Opcode::Eval,
            "addt" => Opcode::Addt,
            "rmvt" => Opcode::Rmvt,
            "wait" => Opcode::Wait,
            "gmet" => Opcode::Gmet,
            "stol" => Opcode::Stol,
            "stog" => Opcode::Stog,
            "bscp" => Opcode::Bscp,
            "escp" => Opcode::Escp,
            "stoe" => Opcode::Stoe,
            "phdl" => Opcode::Phdl,
            "btr" => Opcode::Btr,
            "exst" => Opcode::Exst,
            "argb" => Opcode::Argb,
            "targ" => Opcode::Targ,
            "tcan" => Opcode::Tcan,

            "prl" => Opcode::Prl,
            "pdrl" => Opcode::Pdrl,
            "lbrt" => Opcode::Lbrt,

            "pushv" => Opcode::Pushv,
            &_ => Opcode::Bogus,
        }
    }
}

impl ToBytes for Opcode {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        buf.push((*self).into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kofile::symbols::{KOSymbol, SymBind, SymType};

    #[test]
    fn parse_push() {
        let mnemonic = "push";
        let opcode: Opcode = mnemonic.into();

        assert_eq!(opcode, Opcode::Push);
    }

    #[test]
    fn write_push() {
        let sym = KOSymbol::new(0, 4, SymBind::Local, SymType::NoType, 0);
        let instr = Instr::OneOp(Opcode::Push, Operand::new(&sym, 1));
        let mut buf = Vec::with_capacity(4);

        instr.to_bytes(&mut buf);

        assert_eq!(buf, vec![0x4e, 0x00, 0x00, 0x00, 0x01]);
    }
}
