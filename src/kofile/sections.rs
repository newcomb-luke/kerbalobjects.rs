use crate::{KOSValue, ToBytes};

use super::symbols::KOSymbol;
use std::mem;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SectionKind {
    Null,
    SymTab,
    StrTab,
    Rel,
    Data,
    Debug,
    Unknown,
}

impl From<u8> for SectionKind {
    fn from(byte: u8) -> Self {
        match byte {
            0 => Self::Null,
            1 => Self::SymTab,
            2 => Self::StrTab,
            3 => Self::Rel,
            4 => Self::Data,
            5 => Self::Debug,
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
            SectionKind::Rel => 3,
            SectionKind::Data => 4,
            SectionKind::Debug => 5,
            SectionKind::Unknown => 255,
        }
    }
}

pub struct SectionHeader<'name> {
    name: &'name str,
    name_idx: usize,
    sh_kind: SectionKind,
    size: u32,
    offset: u32,
}

impl<'a> SectionHeader<'a> {
    pub fn null() -> Self {
        SectionHeader {
            name: "",
            name_idx: 0,
            sh_kind: SectionKind::Null,
            size: 0,
            offset: 0,
        }
    }

    pub fn new(name: &'a str, name_idx: usize, sh_kind: SectionKind) -> Self {
        SectionHeader {
            name,
            name_idx,
            sh_kind,
            size: 0,
            offset: 0,
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

    pub fn set_size(&mut self, size: u32) {
        self.size = size;
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn set_offset(&mut self, offset: u32) {
        self.offset = offset;
    }

    pub fn offset(&self) -> u32 {
        self.offset
    }
}

impl<'a> ToBytes for SectionHeader<'a> {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        crate::push_32!(self.name_idx => buf);
        buf.push(self.sh_kind.into());
        crate::push_32!(self.size => buf);
        crate::push_32!(self.offset => buf);
    }
}

pub struct SymbolTable<'a> {
    symbols: Vec<KOSymbol<'a>>,
}

impl<'a> SymbolTable<'a> {
    pub fn new(amount: usize) -> Self {
        SymbolTable {
            symbols: Vec::with_capacity(amount * mem::size_of::<KOSymbol>()),
        }
    }

    pub fn get(&self, index: usize) -> Option<&KOSymbol> {
        self.symbols.get(index)
    }

    pub fn add(&mut self, symbol: KOSymbol<'a>) -> usize {
        self.symbols.push(symbol);
        self.symbols.len() - 1
    }
}

impl<'a> ToBytes for SymbolTable<'a> {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        for symbol in self.symbols.iter() {
            symbol.to_bytes(buf);
        }
    }
}

pub struct StringTable {
    contents: String,
}

impl StringTable {
    pub fn new(size: usize) -> Self {
        let mut contents = String::with_capacity(size);
        contents.push('\0');

        StringTable { contents }
    }

    pub fn get(&self, index: usize) -> Option<&str> {
        let mut end = index;

        let mut contents_iter = self.contents.chars().skip(index);

        loop {
            let c = contents_iter.next().unwrap();

            if c == '\0' {
                break;
            }

            end += c.len_utf8();
        }

        Some(&self.contents[index..end])
    }

    pub fn add(&mut self, new_str: &str) -> (usize, &str) {
        let index = self.contents.len();

        self.contents.push_str(new_str);
        self.contents.push('\0');

        (index, &self.contents[index..index + new_str.len()])
    }
}

impl ToBytes for StringTable {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self.contents.as_bytes());
    }
}

pub struct DataSection<'a> {
    data: Vec<KOSValue<'a>>,
}

impl<'a> DataSection<'a> {
    pub fn new(amount: usize) -> Self {
        DataSection {
            data: Vec::with_capacity(amount * mem::size_of::<KOSValue>()),
        }
    }

    pub fn add(&mut self, value: KOSValue<'a>) -> usize {
        let index = self.data.len();

        self.data.push(value);

        index
    }

    pub fn get(&self, index: usize) -> Option<&KOSValue> {
        self.data.get(index)
    }
}

impl<'a> ToBytes for DataSection<'a> {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        for value in self.data.iter() {
            value.to_bytes(buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kofile::symbols::{SymBind, SymType};

    #[test]
    fn strtab_insert() {
        let mut strtab = StringTable::new(8);

        let (index, s) = strtab.add(".text");

        assert_eq!(index, 1);
        assert_eq!(s, ".text");
    }

    #[test]
    fn strtab_get_null() {
        let strtab = StringTable::new(8);

        assert_eq!(strtab.get(0).unwrap(), "");
    }

    #[test]
    fn strtab_get_str() {
        let mut strtab = StringTable::new(8);

        strtab.add("Hello");

        assert_eq!(strtab.get(1).unwrap(), "Hello");
    }

    #[test]
    fn strtab_get_str_2() {
        let mut strtab = StringTable::new(16);

        strtab.add("Hello");

        let (index, s) = strtab.add("world");

        assert_eq!(index, 7);
        assert_eq!(strtab.get(index).unwrap(), "world");
    }

    #[test]
    fn symtab_insert() {
        let mut strtab = StringTable::new(8);
        let mut symtab = SymbolTable::new(1);

        let mut sym = KOSymbol::new(0, 0, SymBind::Local, SymType::NoType, 3);

        let (s_index, s) = strtab.add("fn_add");
        sym.set_name(s_index, s);

        let sym_index = symtab.add(sym);

        assert_eq!(sym_index, 0);
    }

    #[test]
    fn symtab_get() {
        let mut strtab = StringTable::new(8);
        let mut symtab = SymbolTable::new(1);

        let mut sym = KOSymbol::new(0, 0, SymBind::Local, SymType::NoType, 3);

        let (s_index, s) = strtab.add("fn_add");
        sym.set_name(s_index, s);

        let sym_index = symtab.add(sym);

        assert_eq!(symtab.get(sym_index).unwrap().name(), "fn_add");
    }

    #[test]
    fn data_insert() {
        let mut data_section = DataSection::new(1);

        let index = data_section.add(KOSValue::Int32(365));

        assert_eq!(index, 0);
    }

    #[test]
    fn data_insert_2() {
        let mut data_section = DataSection::new(2);

        data_section.add(KOSValue::ArgMarker);

        let index = data_section.add(KOSValue::Bool(true));

        assert_eq!(index, 1);
    }

    #[test]
    fn data_get() {
        let mut data_section = DataSection::new(1);

        let index = data_section.add(KOSValue::Int16(657));

        assert_eq!(data_section.get(index).unwrap(), &KOSValue::Int16(657));
    }
}
