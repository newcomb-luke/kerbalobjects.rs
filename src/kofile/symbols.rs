use crate::ToBytes;

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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SymType {
    NoType,
    Object,
    Func,
    Section,
    Unknown,
}

impl From<u8> for SymType {
    fn from(byte: u8) -> SymType {
        match byte {
            0 => Self::NoType,
            1 => Self::Object,
            2 => Self::Func,
            3 => Self::Section,
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
            SymType::Unknown => 4,
        }
    }
}

pub struct KOSymbol<'name> {
    name: &'name str,
    name_idx: usize,
    value_idx: usize,
    size: u16,
    sym_bind: SymBind,
    sym_type: SymType,
    sh_idx: u16,
}

impl<'a> KOSymbol<'a> {
    pub fn new(
        value_idx: usize,
        size: u16,
        sym_bind: SymBind,
        sym_type: SymType,
        sh_idx: u16,
    ) -> Self {
        KOSymbol {
            name: "",
            name_idx: 0,
            value_idx,
            size,
            sym_bind,
            sym_type,
            sh_idx,
        }
    }

    pub fn set_name(&mut self, name_idx: usize, name: &'a str) {
        self.name = name;
        self.name_idx = name_idx;
    }

    pub fn name(&self) -> &str {
        self.name
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
}

impl<'a> ToBytes for KOSymbol<'a> {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        crate::push_32!(self.name_idx => buf);
        crate::push_32!(self.value_idx => buf);
        crate::push_16!(self.size => buf);
        buf.push(self.sym_bind.into());
        buf.push(self.sym_type.into());
        crate::push_16!(self.sh_idx => buf);
    }
}
