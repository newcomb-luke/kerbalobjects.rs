use std::hash::Hash;
use std::hash::Hasher;
use std::iter::Peekable;
use std::slice::Iter;

pub mod errors;
pub mod kofile;
pub mod ksmfile;

use errors::{ReadError, ReadResult};

/// Allows a type to be converted to bytes and appended to a Vec<u8>
pub trait ToBytes {
    fn to_bytes(&self, buf: &mut Vec<u8>);
}

/// Allows a type to be converted from a raw Peekable<Iter<u8>> to itself.
pub trait FromBytes {
    /// Parses a value from the byte iterator.
    ///
    /// Returns a ReadResult\<Self\>
    fn from_bytes(source: &mut Peekable<Iter<u8>>) -> ReadResult<Self>
    where
        Self: Sized;
}

/// An internal value within Kerbal Operating System.
///
/// These are documented within the GitHub repo's [KSM Docs](https://github.com/newcomb-luke/kerbalobjects.rs/blob/main/docs/KSM-file-format.md#argument-section).
/// These are used as operands to instructions and stored in the KO file's data section,
/// and a KSM file's argument section.
///
/// Each value takes up 1 byte just for the "data type" so that kOS knows how to load the value.
///
/// The "Value" types (ScalarInt, ScalarDouble, BoolValue, StringValue) are different from their
/// non-value counterparts in that the "Value" types have more built-in suffixes, and are the
/// type used when there are any user-created values, as opposed to instruction operands. See
/// KSM docs for more information.
///
#[derive(Debug, PartialEq, Clone)]
pub enum KOSValue {
    /// A null value, rarely used. Only takes up 1 byte.
    Null,
    /// A boolean. Takes up 2 bytes.
    Bool(bool),
    /// A signed byte. Takes up 2 bytes.
    Byte(i8),
    /// A signed 16-bit integer. Takes up 3 bytes.
    Int16(i16),
    /// A signed 32-bit integer. Takes up 5 bytes.
    Int32(i32),
    /// A 32-bit floating point number. Takes up 5 bytes.
    Float(f32),
    /// A 64-bit floating point number. Takes up 9 bytes.
    Double(f64),
    /// A string. Takes up 2 + length bytes.
    String(String),
    /// An argument marker. Takes up 1 byte.
    ArgMarker,
    /// A signed 32-bit integer. Takes up 5 bytes.
    ScalarInt(i32),
    /// A 64-bit floating point number. Takes up 9 bytes.
    ScalarDouble(f64),
    /// A boolean. Takes up 2 bytes.
    BoolValue(bool),
    /// A string. Takes up 2 + length bytes.
    StringValue(String),
}

impl KOSValue {
    /// Returns the size of the value in bytes.
    pub fn size_bytes(&self) -> usize {
        match &self {
            Self::Null | Self::ArgMarker => 1,
            Self::Bool(_) | Self::Byte(_) | Self::BoolValue(_) => 2,
            Self::Int16(_) => 3,
            Self::Int32(_) | Self::Float(_) | Self::ScalarInt(_) => 5,
            Self::Double(_) | Self::ScalarDouble(_) => 9,
            Self::String(s) | Self::StringValue(s) => {
                2 + s.len() // 1 byte for the type, 1 byte for the length, and then the string
            }
        }
    }
}

impl Hash for KOSValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Null => 0.hash(state),
            Self::Bool(b) => {
                1.hash(state);
                b.hash(state);
            }
            Self::Byte(b) => {
                2.hash(state);
                b.hash(state);
            }
            Self::Int16(i) => {
                3.hash(state);
                i.hash(state);
            }
            Self::Int32(i) => {
                4.hash(state);
                i.hash(state);
            }
            Self::Float(f) => {
                5.hash(state);
                f.to_bits().hash(state);
            }
            Self::Double(d) => {
                6.hash(state);
                d.to_bits().hash(state);
            }
            Self::String(s) => {
                7.hash(state);
                s.hash(state);
            }
            Self::ArgMarker => {
                8.hash(state);
            }
            Self::ScalarInt(i) => {
                9.hash(state);
                i.hash(state);
            }
            Self::ScalarDouble(d) => {
                10.hash(state);
                d.to_bits().hash(state);
            }
            Self::BoolValue(b) => {
                11.hash(state);
                b.hash(state);
            }
            Self::StringValue(s) => {
                12.hash(state);
                s.hash(state);
            }
        }
    }
}

impl ToBytes for KOSValue {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        match self {
            Self::Null => {
                buf.push(0);
            }
            Self::Bool(b) => {
                buf.push(1);
                b.to_bytes(buf);
            }
            Self::Byte(b) => {
                buf.push(2);
                b.to_bytes(buf);
            }
            Self::Int16(i) => {
                buf.push(3);
                i.to_bytes(buf);
            }
            Self::Int32(i) => {
                buf.push(4);
                i.to_bytes(buf);
            }
            Self::Float(f) => {
                buf.push(5);
                f.to_bytes(buf);
            }
            Self::Double(f) => {
                buf.push(6);
                f.to_bytes(buf);
            }
            Self::String(s) => {
                buf.push(7);
                buf.push(s.len() as u8);
                s.to_bytes(buf);
            }
            Self::ArgMarker => {
                buf.push(8);
            }
            Self::ScalarInt(i) => {
                buf.push(9);
                i.to_bytes(buf);
            }
            Self::ScalarDouble(f) => {
                buf.push(10);
                f.to_bytes(buf);
            }
            Self::BoolValue(b) => {
                buf.push(11);
                b.to_bytes(buf);
            }
            Self::StringValue(s) => {
                buf.push(12);
                buf.push(s.len() as u8);
                s.to_bytes(buf);
            }
        }
    }
}

impl FromBytes for KOSValue {
    fn from_bytes(source: &mut Peekable<Iter<u8>>) -> ReadResult<Self>
    where
        Self: Sized,
    {
        let kos_type_value = *source.next().ok_or(ReadError::KOSValueReadError)?;

        match kos_type_value {
            t if t < 13 => match t {
                0 => Ok(KOSValue::Null),
                1 => bool::from_bytes(source).map(|b| KOSValue::Bool(b)),
                2 => i8::from_bytes(source).map(|b| KOSValue::Byte(b)),
                3 => i16::from_bytes(source).map(|i| KOSValue::Int16(i)),
                4 => i32::from_bytes(source).map(|i| KOSValue::Int32(i)),
                5 => f32::from_bytes(source).map(|f| KOSValue::Float(f)),
                6 => f64::from_bytes(source).map(|d| KOSValue::Double(d)),
                7 => String::from_bytes(source).map(|s| KOSValue::String(s)),
                8 => Ok(KOSValue::ArgMarker),
                9 => i32::from_bytes(source).map(|i| KOSValue::ScalarInt(i)),
                10 => f64::from_bytes(source).map(|d| KOSValue::ScalarDouble(d)),
                11 => bool::from_bytes(source).map(|b| KOSValue::BoolValue(b)),
                12 => String::from_bytes(source).map(|s| KOSValue::StringValue(s)),
                _ => unreachable!(),
            }
            .map_err(|_| ReadError::KOSValueReadError),
            _ => Err(ReadError::KOSValueTypeReadError(kos_type_value)),
        }
    }
}

impl ToBytes for bool {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        buf.push(if *self { 1 } else { 0 });
    }
}

impl ToBytes for u8 {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        buf.push(*self);
    }
}

impl ToBytes for i8 {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        buf.push((*self) as u8);
    }
}

impl ToBytes for u16 {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_le_bytes());
    }
}

impl ToBytes for i16 {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_le_bytes());
    }
}

impl ToBytes for u32 {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_le_bytes());
    }
}

impl ToBytes for i32 {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_le_bytes());
    }
}

impl ToBytes for f32 {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_le_bytes());
    }
}

impl ToBytes for f64 {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_le_bytes());
    }
}

impl ToBytes for &str {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self.as_bytes());
    }
}

impl ToBytes for String {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self.as_bytes());
    }
}

impl FromBytes for bool {
    fn from_bytes(source: &mut Peekable<Iter<u8>>) -> ReadResult<Self> {
        source
            .next()
            .map(|&x| x != 0)
            .ok_or(ReadError::UnexpectedEOF)
    }
}

impl FromBytes for u8 {
    fn from_bytes(source: &mut Peekable<Iter<u8>>) -> ReadResult<Self> {
        source.next().map(|&x| x).ok_or(ReadError::UnexpectedEOF)
    }
}

impl FromBytes for i8 {
    fn from_bytes(source: &mut Peekable<Iter<u8>>) -> ReadResult<Self> {
        source
            .next()
            .map(|&x| x as i8)
            .ok_or(ReadError::UnexpectedEOF)
    }
}

impl FromBytes for u16 {
    fn from_bytes(source: &mut Peekable<Iter<u8>>) -> ReadResult<Self> {
        let mut slice = [0u8; 2];
        for i in 0..2 {
            if let Some(&byte) = source.next() {
                slice[i] = byte;
            } else {
                return Err(ReadError::UnexpectedEOF);
            }
        }
        Ok(u16::from_le_bytes(slice))
    }
}

impl FromBytes for i16 {
    fn from_bytes(source: &mut Peekable<Iter<u8>>) -> ReadResult<Self> {
        let mut slice = [0u8; 2];
        for i in 0..2 {
            if let Some(&byte) = source.next() {
                slice[i] = byte;
            } else {
                return Err(ReadError::UnexpectedEOF);
            }
        }
        Ok(i16::from_le_bytes(slice))
    }
}

impl FromBytes for u32 {
    fn from_bytes(source: &mut Peekable<Iter<u8>>) -> ReadResult<Self> {
        let mut slice = [0u8; 4];
        for i in 0..4 {
            if let Some(&byte) = source.next() {
                slice[i] = byte;
            } else {
                return Err(ReadError::UnexpectedEOF);
            }
        }
        Ok(u32::from_le_bytes(slice))
    }
}

impl FromBytes for i32 {
    fn from_bytes(source: &mut Peekable<Iter<u8>>) -> ReadResult<Self> {
        let mut slice = [0u8; 4];
        for i in 0..4 {
            if let Some(&byte) = source.next() {
                slice[i] = byte;
            } else {
                return Err(ReadError::UnexpectedEOF);
            }
        }
        Ok(i32::from_le_bytes(slice))
    }
}

impl FromBytes for f32 {
    fn from_bytes(source: &mut Peekable<Iter<u8>>) -> ReadResult<Self> {
        let mut slice = [0u8; 4];
        for i in 0..4 {
            if let Some(&byte) = source.next() {
                slice[i] = byte;
            } else {
                return Err(ReadError::UnexpectedEOF);
            }
        }
        Ok(f32::from_le_bytes(slice))
    }
}

impl FromBytes for f64 {
    fn from_bytes(source: &mut Peekable<Iter<u8>>) -> ReadResult<Self> {
        let mut slice = [0u8; 8];
        for i in 0..8 {
            if let Some(&byte) = source.next() {
                slice[i] = byte;
            } else {
                return Err(ReadError::UnexpectedEOF);
            }
        }
        Ok(f64::from_le_bytes(slice))
    }
}

impl FromBytes for String {
    fn from_bytes(source: &mut Peekable<Iter<u8>>) -> ReadResult<Self> {
        let len = match source.next() {
            Some(v) => *v,
            None => {
                return Err(ReadError::UnexpectedEOF);
            }
        };
        let mut s = String::with_capacity(len as usize);
        for _ in 0..len {
            if let Some(&byte) = source.next() {
                s.push(byte as char);
            } else {
                return Err(ReadError::UnexpectedEOF);
            }
        }
        Ok(s)
    }
}

/// The opcode of a kOS machine code instruction.
///
/// Each instruction is made up of an opcode, and a series of zero or more operands.
/// This enum contains all currently supported kOS opcodes, and 2 additional ones.
///
/// Opcode::Bogus is the Opcode variant used to express that it is an unrecognized opcode.
/// This is inspired by the actual kOS C# code.
///
/// Opcode::Pushv is an internal value that is can be used by tools working with kOS machine code,
/// and it represents a normal Opcode::Push instruction, however specifying that the operand should be
/// stored as a KOSValue "Value" type (ScalarDouble, StringValue). This is mostly just useful for the
/// KASM assembler implementation.
///
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

impl Opcode {
    /// Returns the number of operands that this instruction type should have.
    pub fn num_operands(&self) -> usize {
        match self {
            Opcode::Eof => 0,
            Opcode::Eop => 0,
            Opcode::Nop => 0,
            Opcode::Sto => 1,
            Opcode::Uns => 0,
            Opcode::Gmb => 1,
            Opcode::Smb => 1,
            Opcode::Gidx => 0,
            Opcode::Sidx => 0,
            Opcode::Bfa => 1,
            Opcode::Jmp => 1,
            Opcode::Add => 0,
            Opcode::Sub => 0,
            Opcode::Mul => 0,
            Opcode::Div => 0,
            Opcode::Pow => 0,
            Opcode::Cgt => 0,
            Opcode::Clt => 0,
            Opcode::Cge => 0,
            Opcode::Cle => 0,
            Opcode::Ceq => 0,
            Opcode::Cne => 0,
            Opcode::Neg => 0,
            Opcode::Bool => 0,
            Opcode::Not => 0,
            Opcode::And => 0,
            Opcode::Or => 0,
            Opcode::Call => 2,
            Opcode::Ret => 1,
            Opcode::Push => 1,
            Opcode::Pop => 0,
            Opcode::Dup => 0,
            Opcode::Swap => 0,
            Opcode::Eval => 0,
            Opcode::Addt => 2,
            Opcode::Rmvt => 0,
            Opcode::Wait => 0,
            Opcode::Gmet => 1,
            Opcode::Stol => 1,
            Opcode::Stog => 1,
            Opcode::Bscp => 2,
            Opcode::Escp => 1,
            Opcode::Stoe => 1,
            Opcode::Phdl => 2,
            Opcode::Btr => 1,
            Opcode::Exst => 0,
            Opcode::Argb => 0,
            Opcode::Targ => 0,
            Opcode::Tcan => 0,

            Opcode::Prl => 1,
            Opcode::Pdrl => 2,
            Opcode::Lbrt => 1,

            Opcode::Pushv => 1,

            Opcode::Bogus => 0,
        }
    }
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

impl From<Opcode> for &str {
    fn from(opcode: Opcode) -> Self {
        match opcode {
            Opcode::Eof => "eof",
            Opcode::Eop => "eop",
            Opcode::Nop => "nop",
            Opcode::Sto => "sto",
            Opcode::Uns => "uns",
            Opcode::Gmb => "gmb",
            Opcode::Smb => "smb",
            Opcode::Gidx => "gidx",
            Opcode::Sidx => "sidx",
            Opcode::Bfa => "bfa",
            Opcode::Jmp => "jmp",
            Opcode::Add => "add",
            Opcode::Sub => "sub",
            Opcode::Mul => "mul",
            Opcode::Div => "div",
            Opcode::Pow => "pow",
            Opcode::Cgt => "cgt",
            Opcode::Clt => "clt",
            Opcode::Cge => "cge",
            Opcode::Cle => "cle",
            Opcode::Ceq => "ceq",
            Opcode::Cne => "cne",
            Opcode::Neg => "neg",
            Opcode::Bool => "bool",
            Opcode::Not => "not",
            Opcode::And => "and",
            Opcode::Or => "or",
            Opcode::Call => "call",
            Opcode::Ret => "ret",
            Opcode::Push => "push",
            Opcode::Pop => "pop",
            Opcode::Dup => "dup",
            Opcode::Swap => "swap",
            Opcode::Eval => "eval",
            Opcode::Addt => "addt",
            Opcode::Rmvt => "rmvt",
            Opcode::Wait => "wait",
            Opcode::Gmet => "gmet",
            Opcode::Stol => "stol",
            Opcode::Stog => "stog",
            Opcode::Bscp => "bscp",
            Opcode::Escp => "ecsp",
            Opcode::Stoe => "stoe",
            Opcode::Phdl => "phdl",
            Opcode::Btr => "btr",
            Opcode::Exst => "exst",
            Opcode::Argb => "argb",
            Opcode::Targ => "targ",
            Opcode::Tcan => "tcan",

            Opcode::Prl => "prl",
            Opcode::Pdrl => "pdrl",
            Opcode::Lbrt => "lbrt",

            Opcode::Pushv => "pushv",

            Opcode::Bogus => "bogus",
        }
    }
}

impl ToBytes for Opcode {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        buf.push((*self).into());
    }
}

impl FromBytes for Opcode {
    fn from_bytes(source: &mut Peekable<Iter<u8>>) -> ReadResult<Self> {
        let value = *source.next().ok_or(ReadError::OpcodeReadError)?;
        let opcode = Opcode::from(value);

        match opcode {
            Opcode::Bogus => Err(ReadError::BogusOpcodeReadError(value)),
            _ => Ok(opcode),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_to_bytes() {
        let v = KOSValue::Null;

        let mut buf = Vec::with_capacity(1);

        v.to_bytes(&mut buf);

        assert_eq!(buf, vec![0]);
    }

    #[test]
    fn bool_to_bytes() {
        let v1 = KOSValue::Bool(true);
        let v2 = KOSValue::Bool(false);

        let mut buf = Vec::with_capacity(2);

        v1.to_bytes(&mut buf);

        assert_eq!(buf, vec![1, 1]);

        buf.clear();
        v2.to_bytes(&mut buf);

        assert_eq!(buf, vec![1, 0]);
    }

    #[test]
    fn byte_to_bytes() {
        let v1 = KOSValue::Byte(0);
        let v2 = KOSValue::Byte(-128);

        let mut buf = Vec::with_capacity(2);

        v1.to_bytes(&mut buf);

        assert_eq!(buf, vec![2, 0]);

        buf.clear();
        v2.to_bytes(&mut buf);

        assert_eq!(buf, vec![2, (-128 as i8) as u8]);
    }

    #[test]
    fn int16_to_bytes() {
        let v1 = KOSValue::Int16(526);

        let mut buf = Vec::with_capacity(3);

        v1.to_bytes(&mut buf);

        assert_eq!(buf, vec![3, 0b00001110, 0b00000010]);
    }

    #[test]
    fn int32_to_bytes() {
        let v1 = KOSValue::Int32(-764);

        let mut buf = Vec::with_capacity(5);

        v1.to_bytes(&mut buf);

        assert_eq!(buf, vec![4, 0b00000100, 0b11111101, 0b11111111, 0b11111111]);
    }

    #[test]
    fn float_to_bytes() {
        let v1 = KOSValue::Float(3.14159);

        let mut buf = Vec::with_capacity(5);

        v1.to_bytes(&mut buf);

        assert_eq!(buf, vec![5, 208, 15, 73, 64]);
    }

    #[test]
    fn double_to_bytes() {
        let v1 = KOSValue::Double(3.14159);

        let mut buf = Vec::with_capacity(9);

        v1.to_bytes(&mut buf);

        assert_eq!(buf, vec![6, 110, 134, 27, 240, 249, 33, 9, 64]);
    }

    #[test]
    fn string_to_bytes() {
        let v1 = KOSValue::String(String::from("test str"));

        let mut buf = Vec::with_capacity(10);

        v1.to_bytes(&mut buf);

        assert_eq!(
            buf,
            vec![7, 8, b't', b'e', b's', b't', b' ', b's', b't', b'r']
        );
    }

    #[test]
    fn argmarker_to_bytes() {
        let v1 = KOSValue::ArgMarker;

        let mut buf = Vec::with_capacity(1);

        v1.to_bytes(&mut buf);

        assert_eq!(buf, vec![8]);
    }

    #[test]
    fn scalarint_to_bytes() {
        let v1 = KOSValue::ScalarInt(-1267);

        let mut buf = Vec::with_capacity(5);

        v1.to_bytes(&mut buf);

        assert_eq!(buf, vec![9, 0b00001101, 0b11111011, 0b11111111, 0b11111111]);
    }

    #[test]
    fn scalardouble_to_bytes() {
        let v1 = KOSValue::ScalarDouble(3.14159);

        let mut buf = Vec::with_capacity(9);

        v1.to_bytes(&mut buf);

        assert_eq!(buf, vec![10, 110, 134, 27, 240, 249, 33, 9, 64]);
    }

    #[test]
    fn boolvalue_to_bytes() {
        let v1 = KOSValue::BoolValue(true);
        let v2 = KOSValue::BoolValue(false);

        let mut buf = Vec::with_capacity(2);

        v1.to_bytes(&mut buf);

        assert_eq!(buf, vec![11, 1]);

        buf.clear();
        v2.to_bytes(&mut buf);

        assert_eq!(buf, vec![11, 0]);
    }

    #[test]
    fn stringvalue_to_bytes() {
        let v1 = KOSValue::StringValue(String::from("hello"));

        let mut buf = Vec::with_capacity(7);

        v1.to_bytes(&mut buf);

        assert_eq!(buf, vec![12, 5, b'h', b'e', b'l', b'l', b'o']);
    }
}
