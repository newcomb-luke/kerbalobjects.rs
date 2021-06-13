use std::iter::Peekable;
use std::slice::Iter;

use crate::{FromBytes, KOSValue, ToBytes};

use crate::errors::{ReadError, ReadResult};

use super::{instructions::Instr, symbols::KOSymbol};
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

impl ToBytes for SectionKind {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        buf.push((*self).into());
    }
}

impl FromBytes for SectionKind {
    fn from_bytes(source: &mut Peekable<Iter<u8>>, debug: bool) -> ReadResult<Self>
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
    fn from_bytes(source: &mut Peekable<Iter<u8>>, debug: bool) -> ReadResult<Self>
    where
        Self: Sized,
    {
        let name_idx = u32::from_bytes(source, debug)
            .map_err(|_| ReadError::SectionHeaderConstantReadError("name index"))?
            as usize;
        let sh_kind = SectionKind::from_bytes(source, debug)?;
        let size = u32::from_bytes(source, debug)
            .map_err(|_| ReadError::SectionHeaderConstantReadError("size"))?;

        Ok(SectionHeader {
            name_idx,
            sh_kind,
            size,
        })
    }
}

pub struct SymbolTable {
    symbols: Vec<KOSymbol>,
    size: usize,
    section_index: usize,
}

impl SymbolTable {
    pub fn new(amount: usize, section_index: usize) -> Self {
        SymbolTable {
            symbols: Vec::with_capacity(amount * mem::size_of::<KOSymbol>()),
            size: 0,
            section_index,
        }
    }

    pub fn get(&self, index: usize) -> Option<&KOSymbol> {
        self.symbols.get(index)
    }

    pub fn add(&mut self, symbol: KOSymbol) -> usize {
        self.size += KOSymbol::size_bytes() as usize;
        self.symbols.push(symbol);
        self.symbols.len() - 1
    }

    pub fn size(&self) -> u32 {
        self.size as u32
    }

    pub fn symbols(&self) -> Iter<KOSymbol> {
        self.symbols.iter()
    }

    pub fn section_index(&self) -> usize {
        self.sction_index
    }

    pub fn from_bytes(
        source: &mut Peekable<Iter<u8>>,
        debug: bool,
        size: usize,
        section_index: usize,
    ) -> ReadResult<Self> {
        let num_symbols = size / KOSymbol::size_bytes() as usize;
        let mut read_symbols = 0;

        let mut sym_tab = SymbolTable::new(num_symbols as usize, section_index);

        while (read_symbols * KOSymbol::size_bytes() as usize) < size {
            let symbol = KOSymbol::from_bytes(source, debug)?;
            read_symbols += 1;

            sym_tab.add(symbol);
        }

        Ok(sym_tab)
    }
}

impl ToBytes for SymbolTable {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        for symbol in self.symbols.iter() {
            symbol.to_bytes(buf);
        }
    }
}

pub struct StringTable {
    contents: String,
    section_index: usize,
}

impl StringTable {
    pub fn new(size: usize, section_index: usize) -> Self {
        let mut contents = String::with_capacity(size);
        contents.push('\0');

        StringTable {
            contents,
            section_index,
        }
    }

    pub fn get(&self, index: usize) -> Option<&str> {
        let mut end = index;

        let mut contents_iter = self.contents.chars().skip(index);

        loop {
            if let Some(c) = contents_iter.next() {
                if c == '\0' {
                    break;
                }

                end += c.len_utf8();
            } else {
                break;
            }
        }

        Some(&self.contents[index..end])
    }

    pub fn strings(&self) -> Vec<&str> {
        let mut strs = Vec::new();
        let mut index = 1;
        let mut end = index;
        let mut contents_iter = self.contents.chars();

        while contents_iter.next().is_some() {
            loop {
                if let Some(c) = contents_iter.next() {
                    if c == '\0' {
                        break;
                    }

                    end += c.len_utf8();
                } else {
                    break;
                }
            }

            strs.push(&self.contents[index..end]);

            index += end - index + 1;
            end = index;
        }

        strs
    }

    pub fn add(&mut self, new_str: &str) -> usize {
        let index = self.contents.len();

        self.contents.push_str(new_str);
        self.contents.push('\0');

        index
    }

    pub fn size(&self) -> u32 {
        self.contents.len() as u32
    }

    pub fn section_index(&self) -> usize {
        self.section_index
    }

    pub fn from_bytes(
        source: &mut Peekable<Iter<u8>>,
        debug: bool,
        size: usize,
        section_index: usize,
    ) -> ReadResult<Self> {
        let mut s_vec = Vec::with_capacity(size);

        for _ in 0..size {
            let b = *source.next().ok_or(ReadError::StringTableReadError)?;
            s_vec.push(b);
        }

        let contents = String::from_utf8(s_vec).map_err(|_| ReadError::StringTableReadError)?;

        Ok(StringTable {
            contents,
            section_index,
        })
    }
}

impl ToBytes for StringTable {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self.contents.as_bytes());
    }
}

pub struct DataSection {
    data: Vec<KOSValue>,
    size: usize,
    section_index: usize,
}

impl DataSection {
    pub fn new(amount: usize, section_index: usize) -> Self {
        DataSection {
            data: Vec::with_capacity(amount),
            size: 0,
            section_index,
        }
    }

    pub fn add(&mut self, value: KOSValue) -> usize {
        let index = self.data.len();

        self.size += value.size_bytes();
        self.data.push(value);

        index
    }

    pub fn get(&self, index: usize) -> Option<&KOSValue> {
        self.data.get(index)
    }

    pub fn data(&self) -> Iter<KOSValue> {
        self.data.iter()
    }

    pub fn size(&self) -> u32 {
        self.size as u32
    }

    pub fn section_index(&self) -> usize {
        self.section_index
    }

    pub fn from_bytes(
        source: &mut Peekable<Iter<u8>>,
        debug: bool,
        size: usize,
        section_index: usize,
    ) -> ReadResult<Self> {
        let mut new_size = 0;
        let mut data = Vec::new();

        while new_size < size {
            let kos_value = KOSValue::from_bytes(source, debug)?;
            new_size += kos_value.size_bytes();

            data.push(kos_value);
        }

        Ok(DataSection {
            data,
            size,
            section_index,
        })
    }
}

impl ToBytes for DataSection {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        for value in self.data.iter() {
            value.to_bytes(buf);
        }
    }
}

pub struct RelSection {
    instructions: Vec<Instr>,
    size: usize,
    section_index: usize,
}

impl RelSection {
    pub fn new(amount: usize, section_index: usize) -> Self {
        RelSection {
            instructions: Vec::with_capacity(amount),
            size: 0,
            section_index,
        }
    }

    pub fn add(&mut self, instr: Instr) -> usize {
        let index = self.instructions.len();

        self.size += instr.size_bytes();
        self.instructions.push(instr);

        index
    }

    pub fn get(&self, index: usize) -> Option<&Instr> {
        self.instructions.get(index)
    }

    pub fn size(&self) -> u32 {
        self.size as u32
    }

    pub fn instructions(&self) -> Iter<Instr> {
        self.instructions.iter()
    }

    pub fn section_index(&self) -> usize {
        self.section_index
    }

    pub fn from_bytes(
        source: &mut Peekable<Iter<u8>>,
        debug: bool,
        size: usize,
        section_index: usize,
    ) -> ReadResult<Self> {
        let mut new_size = 0;
        let mut instructions = Vec::new();

        while new_size < size {
            let instr = Instr::from_bytes(source, debug)?;
            new_size += instr.size_bytes();

            instructions.push(instr);
        }

        Ok(RelSection {
            instructions,
            size,
            section_index,
        })
    }
}

impl ToBytes for RelSection {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        for instr in self.instructions.iter() {
            instr.to_bytes(buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kofile::symbols::{SymBind, SymType};

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

        assert_eq!(index, 7);
        assert_eq!(strtab.get(index).unwrap(), "world");
    }

    #[test]
    fn symtab_insert() {
        let mut strtab = StringTable::new(8, 0);
        let mut symtab = SymbolTable::new(1, 0);

        let mut sym = KOSymbol::new(0, 0, SymBind::Local, SymType::NoType, 3);

        let s_index = strtab.add("fn_add");
        sym.set_name_idx(s_index);

        let sym_index = symtab.add(sym);

        assert_eq!(sym_index, 0);
    }

    #[test]
    fn symtab_get() {
        let mut strtab = StringTable::new(8, 0);
        let mut symtab = SymbolTable::new(1, 0);

        let mut sym = KOSymbol::new(0, 0, SymBind::Local, SymType::NoType, 3);

        let s_index = strtab.add("fn_add");
        sym.set_name_idx(s_index);

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
