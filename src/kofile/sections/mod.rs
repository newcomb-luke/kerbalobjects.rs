use std::iter::Peekable;
use std::slice::Iter;

use crate::errors::{ReadError, ReadResult};
use crate::{FromBytes, ToBytes};

mod data_section;
mod func_section;
mod reld_section;
mod string_table;
mod symbol_table;

pub use data_section::*;
pub use func_section::*;
pub use reld_section::*;
pub use string_table::*;
pub use symbol_table::*;

pub trait SectionIndex {
    fn section_index(&self) -> usize;
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SectionKind {
    Null,
    SymTab,
    StrTab,
    Func,
    Data,
    Debug,
    Reld,
    Unknown,
}

impl From<u8> for SectionKind {
    fn from(byte: u8) -> Self {
        match byte {
            0 => Self::Null,
            1 => Self::SymTab,
            2 => Self::StrTab,
            3 => Self::Func,
            4 => Self::Data,
            5 => Self::Debug,
            6 => Self::Reld,
            _ => Self::Unknown,
        }
    }
}

impl From<SectionKind> for u8 {
    fn from(kind: SectionKind) -> Self {
        match kind {
            SectionKind::Null => 0,
            SectionKind::SymTab => 1,
            SectionKind::StrTab => 2,
            SectionKind::Func => 3,
            SectionKind::Data => 4,
            SectionKind::Debug => 5,
            SectionKind::Reld => 6,
            SectionKind::Unknown => 255,
        }
    }
}

impl ToBytes for SectionKind {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        buf.push((*self).into());
    }
}

impl FromBytes for SectionKind {
    fn from_bytes(source: &mut Peekable<Iter<u8>>) -> ReadResult<Self>
    where
        Self: Sized,
    {
        let value = *source.next().ok_or(ReadError::SectionKindReadError)?;
        let kind = SectionKind::from(value);

        match kind {
            SectionKind::Unknown => Err(ReadError::UnknownSectionKindReadError(value)),
            _ => Ok(kind),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SectionHeader {
    name_idx: usize,
    sh_kind: SectionKind,
    size: u32,
}

impl SectionHeader {
    pub fn null() -> Self {
        SectionHeader {
            name_idx: 0,
            sh_kind: SectionKind::Null,
            size: 0,
        }
    }

    pub fn new(name_idx: usize, sh_kind: SectionKind) -> Self {
        SectionHeader {
            name_idx,
            sh_kind,
            size: 0,
        }
    }

    pub fn set_name_idx(&mut self, name_idx: usize) {
        self.name_idx = name_idx;
    }

    pub fn name_idx(&self) -> usize {
        self.name_idx
    }

    pub fn set_size(&mut self, size: u32) {
        self.size = size;
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn kind(&self) -> SectionKind {
        self.sh_kind
    }

    pub fn size_bytes() -> usize {
        9
    }
}

impl ToBytes for SectionHeader {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        (self.name_idx as u32).to_bytes(buf);
        self.sh_kind.to_bytes(buf);
        self.size.to_bytes(buf);
    }
}

impl FromBytes for SectionHeader {
    fn from_bytes(source: &mut Peekable<Iter<u8>>) -> ReadResult<Self>
    where
        Self: Sized,
    {
        let name_idx = u32::from_bytes(source)
            .map_err(|_| ReadError::SectionHeaderConstantReadError("name index"))?
            as usize;
        let sh_kind = SectionKind::from_bytes(source)?;
        let size = u32::from_bytes(source)
            .map_err(|_| ReadError::SectionHeaderConstantReadError("size"))?;

        Ok(SectionHeader {
            name_idx,
            sh_kind,
            size,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kofile::symbols::{SymBind, SymType};
    use crate::KOSValue;

    #[test]
    fn strtab_insert() {
        let mut strtab = StringTable::new(8, 0);

        let index = strtab.add(".text");

        assert_eq!(index, 1);
        assert_eq!(strtab.get(index).unwrap(), ".text");
    }

    #[test]
    fn strtab_get_null() {
        let strtab = StringTable::new(8, 0);

        assert_eq!(strtab.get(0).unwrap(), "");
    }

    #[test]
    fn strtab_get_str() {
        let mut strtab = StringTable::new(8, 0);

        strtab.add("Hello");

        assert_eq!(strtab.get(1).unwrap(), "Hello");
    }

    #[test]
    fn strtab_get_str_2() {
        let mut strtab = StringTable::new(16, 0);

        strtab.add("Hello");

        let index = strtab.add("world");

        assert_eq!(index, 2);
        assert_eq!(strtab.get(index).unwrap(), "world");
    }

    #[test]
    fn strtab_strings() {
        let mut strtab = StringTable::new(16, 0);

        strtab.add("Hello");
        strtab.add("rust");
        strtab.add("world");

        let mut strs = strtab.strings();

        assert_eq!(strs.next().unwrap(), "");
        assert_eq!(strs.next().unwrap(), "Hello");
        assert_eq!(strs.next().unwrap(), "rust");
        assert_eq!(strs.next().unwrap(), "world");
    }

    #[test]
    fn symtab_insert() {
        use crate::kofile::symbols::KOSymbol;

        let mut strtab = StringTable::new(8, 0);
        let mut symtab = SymbolTable::new(1, 0);

        let s_index = strtab.add("fn_add");

        let sym = KOSymbol::new(s_index, 0, 0, SymBind::Local, SymType::Func, 3);

        let sym_index = symtab.add(sym);

        assert_eq!(sym_index, 0);
    }

    #[test]
    fn symtab_get() {
        use crate::kofile::symbols::KOSymbol;

        let mut strtab = StringTable::new(8, 0);
        let mut symtab = SymbolTable::new(1, 0);

        let s_index = strtab.add("fn_add");

        let sym = KOSymbol::new(s_index, 0, 0, SymBind::Local, SymType::Func, 3);

        let sym_index = symtab.add(sym);

        assert_eq!(
            strtab
                .get(symtab.get(sym_index).unwrap().name_idx())
                .unwrap(),
            "fn_add"
        );
    }

    #[test]
    fn data_insert() {
        let mut data_section = DataSection::new(1, 0);

        let index = data_section.add(KOSValue::Int32(365));

        assert_eq!(index, 0);
    }

    #[test]
    fn data_insert_2() {
        let mut data_section = DataSection::new(2, 0);

        data_section.add(KOSValue::ArgMarker);

        let index = data_section.add(KOSValue::Bool(true));

        assert_eq!(index, 1);
    }

    #[test]
    fn data_get() {
        let mut data_section = DataSection::new(1, 0);

        let index = data_section.add(KOSValue::Int16(657));

        assert_eq!(data_section.get(index).unwrap(), &KOSValue::Int16(657));
    }

    #[test]
    fn write_header() {
        let mut sh = SectionHeader::new(0, SectionKind::SymTab);
        sh.set_size(42);

        let mut buf = Vec::new();

        sh.to_bytes(&mut buf);

        assert_eq!(
            buf,
            vec![0x00, 0x00, 0x00, 0x00, 0x01, 0x2a, 0x00, 0x00, 0x00]
        );
    }
}
