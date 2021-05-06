use std::iter::Peekable;
use std::slice::Iter;

use crate::{FromBytes, ToBytes};

use crate::errors::{ReadError, ReadResult};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SymBind {
    Local,
    Global,
    Extern,
    Unknown,
}

impl From<u8> for SymBind {
    fn from(byte: u8) -> SymBind {
        match byte {
            0 => Self::Local,
            1 => Self::Global,
            2 => Self::Extern,
            _ => Self::Unknown,
        }
    }
}

impl From<SymBind> for u8 {
    fn from(bind: SymBind) -> u8 {
        match bind {
            SymBind::Local => 0,
            SymBind::Global => 1,
            SymBind::Extern => 2,
            SymBind::Unknown => 3,
        }
    }
}

impl ToBytes for SymBind {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        buf.push((*self).into());
    }
}

impl FromBytes for SymBind {
    fn from_bytes(source: &mut Peekable<Iter<u8>>) -> ReadResult<Self>
    where
        Self: Sized,
    {
        let value = *source.next().ok_or(ReadError::SymBindReadError)?;
        let sym_bind = SymBind::from(value);

        match sym_bind {
            SymBind::Unknown => Err(ReadError::UnknownSimBindReadError(value)),
            _ => Ok(sym_bind),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SymType {
    NoType,
    Object,
    Func,
    Section,
    File,
    Unknown,
}

impl From<u8> for SymType {
    fn from(byte: u8) -> SymType {
        match byte {
            0 => Self::NoType,
            1 => Self::Object,
            2 => Self::Func,
            3 => Self::Section,
            4 => Self::File,
            _ => Self::Unknown,
        }
    }
}

impl From<SymType> for u8 {
    fn from(sym_type: SymType) -> u8 {
        match sym_type {
            SymType::NoType => 0,
            SymType::Object => 1,
            SymType::Func => 2,
            SymType::Section => 3,
            SymType::File => 4,
            SymType::Unknown => 5,
        }
    }
}

impl ToBytes for SymType {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        buf.push((*self).into());
    }
}

impl FromBytes for SymType {
    fn from_bytes(source: &mut Peekable<Iter<u8>>) -> ReadResult<Self>
    where
        Self: Sized,
    {
        let value = *source.next().ok_or(ReadError::SymTypeReadError)?;
        let sym_type = SymType::from(value);

        match sym_type {
            SymType::Unknown => Err(ReadError::UnknownSimTypeReadError(value)),
            _ => Ok(sym_type),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KOSymbol {
    name_idx: usize,
    value_idx: usize,
    size: u16,
    sym_bind: SymBind,
    sym_type: SymType,
    sh_idx: u16,
}

impl KOSymbol {
    pub fn new(
        value_idx: usize,
        size: u16,
        sym_bind: SymBind,
        sym_type: SymType,
        sh_idx: u16,
    ) -> Self {
        KOSymbol {
            name_idx: 0,
            value_idx,
            size,
            sym_bind,
            sym_type,
            sh_idx,
        }
    }

    pub fn set_name_idx(&mut self, name_idx: usize) {
        self.name_idx = name_idx;
    }

    pub fn name_idx(&self) -> usize {
        self.name_idx
    }

    pub fn value_idx(&self) -> usize {
        self.value_idx
    }

    pub fn size(&self) -> u16 {
        self.size
    }

    pub fn sym_bind(&self) -> SymBind {
        self.sym_bind
    }

    pub fn sym_type(&self) -> SymType {
        self.sym_type
    }

    pub fn sh_idx(&self) -> u16 {
        self.sh_idx
    }

    pub fn size_bytes() -> u32 {
        14
    }
}

impl ToBytes for KOSymbol {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        (self.name_idx as u32).to_bytes(buf);
        (self.value_idx as u32).to_bytes(buf);
        self.size.to_bytes(buf);
        self.sym_bind.to_bytes(buf);
        self.sym_type.to_bytes(buf);
        self.sh_idx.to_bytes(buf);
    }
}

impl FromBytes for KOSymbol {
    fn from_bytes(source: &mut Peekable<Iter<u8>>) -> ReadResult<Self>
    where
        Self: Sized,
    {
        let name_idx = u32::from_bytes(source)
            .map_err(|_| ReadError::KOSymbolConstantReadError("name index"))?
            as usize;
        let value_idx = u32::from_bytes(source)
            .map_err(|_| ReadError::KOSymbolConstantReadError("value index"))?
            as usize;
        let size =
            u16::from_bytes(source).map_err(|_| ReadError::KOSymbolConstantReadError("size"))?;
        let sym_bind = SymBind::from_bytes(source)?;
        let sym_type = SymType::from_bytes(source)?;
        let sh_idx = u16::from_bytes(source)
            .map_err(|_| ReadError::KOSymbolConstantReadError("section index"))?;

        Ok(KOSymbol {
            name_idx,
            value_idx,
            size,
            sym_bind,
            sym_type,
            sh_idx,
        })
    }
}
