use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::iter::Peekable;
use std::slice::Iter;

use crate::{FromBytes, KOSValue, ToBytes};

use crate::errors::{ReadError, ReadResult};

use super::{instructions::Instr, symbols::KOSymbol, symbols::ReldEntry, SectionFromBytes};
use std::mem;

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
    fn from_bytes(source: &mut Peekable<Iter<u8>>, _debug: bool) -> ReadResult<Self>
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

impl SectionIndex for SymbolTable {
    fn section_index(&self) -> usize {
        self.section_index
    }
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

    pub fn find_has_name(&self, name_idx: usize) -> Option<&KOSymbol> {
        self.symbols.iter().find(|sym| sym.name_idx() == name_idx)
    }

    pub fn find(&self, symbol: &KOSymbol) -> Option<usize> {
        self.symbols.iter().position(|sym| sym == symbol)
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
}

impl SectionFromBytes for SymbolTable {
    fn from_bytes(
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
    hashes: Vec<u64>,
    contents: Vec<String>,
    section_index: usize,
    size: usize,
}

impl SectionIndex for StringTable {
    fn section_index(&self) -> usize {
        self.section_index
    }
}

impl StringTable {
    pub fn new(amount: usize, section_index: usize) -> Self {
        let mut contents = Vec::with_capacity(amount);
        contents.push(String::new());

        StringTable {
            hashes: Vec::new(),
            contents,
            section_index,
            size: 1,
        }
    }

    pub fn get(&self, index: usize) -> Option<&String> {
        self.contents.get(index)
    }

    pub fn find(&self, s: &str) -> Option<usize> {
        let mut hasher = DefaultHasher::new();
        hasher.write(s.as_bytes());
        let hash = hasher.finish();

        self.hashes.iter().position(|item| **item == *hash)
    }

    pub fn strings(&self) -> Iter<String> {
        self.contents.iter()
    }

    pub fn add_checked(&mut self, new_str: &str) -> usize {
        match self.find(new_str) {
            Some(index) => index,
            None => self.add(new_str),
        }
    }

    pub fn add(&mut self, new_str: &str) -> usize {
        let mut hasher = DefaultHasher::new();
        hasher.write(new_str.as_bytes());
        let hash = hasher.finish();

        self.hashes.push(hash);
        self.size += new_str.len() + 1;
        self.contents.push(String::from(new_str));

        self.contents.len() - 1
    }

    pub fn size(&self) -> u32 {
        self.size as u32
    }
}

impl SectionFromBytes for StringTable {
    fn from_bytes(
        source: &mut Peekable<Iter<u8>>,
        debug: bool,
        size: usize,
        section_index: usize,
    ) -> ReadResult<Self> {
        let mut contents = Vec::new();
        let mut hashes = Vec::new();
        let mut read = 0;
        let mut hasher = DefaultHasher::new();

        while read < size {
            let mut b = Vec::new();

            while read < size {
                let c = *source.next().ok_or(ReadError::StringTableReadError)?;
                read += 1;

                if c == b'\0' {
                    break;
                }

                b.push(c);
            }

            let s = String::from_utf8(b).map_err(|_| ReadError::StringTableReadError)?;
            s.hash(&mut hasher);
            let hash = hasher.finish();

            hashes.push(hash);
            contents.push(s);
        }

        if debug {
            println!("String table strings: ");

            for s in contents.iter() {
                println!("\t{}", s);
            }
        }

        Ok(StringTable {
            hashes,
            contents,
            section_index,
            size,
        })
    }
}

impl ToBytes for StringTable {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        for string in self.contents.iter() {
            buf.extend_from_slice(string.as_bytes());
            buf.push(0);
        }
    }
}

pub struct DataSection {
    hashes: Vec<u64>,
    data: Vec<KOSValue>,
    size: usize,
    section_index: usize,
}

impl SectionIndex for DataSection {
    fn section_index(&self) -> usize {
        self.section_index
    }
}

impl DataSection {
    pub fn new(amount: usize, section_index: usize) -> Self {
        DataSection {
            hashes: Vec::with_capacity(amount),
            data: Vec::with_capacity(amount),
            size: 0,
            section_index,
        }
    }

    pub fn find(&self, value: &KOSValue) -> Option<usize> {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        let hash = hasher.finish();

        self.hashes.iter().position(|item| **item == *hash)
    }

    pub fn add_checked(&mut self, value: KOSValue) -> usize {
        match self.find(&value) {
            Some(index) => index,
            None => self.add(value),
        }
    }

    pub fn add(&mut self, value: KOSValue) -> usize {
        self.size += value.size_bytes();

        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        let hash = hasher.finish();

        self.hashes.push(hash);
        self.data.push(value);

        self.data.len() - 1
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
}

impl SectionFromBytes for DataSection {
    fn from_bytes(
        source: &mut Peekable<Iter<u8>>,
        debug: bool,
        size: usize,
        section_index: usize,
    ) -> ReadResult<Self> {
        let mut read = 0;
        let mut data = Vec::new();
        let mut hashes = Vec::new();

        let mut hasher = DefaultHasher::new();

        while read < size {
            let kos_value = KOSValue::from_bytes(source, debug)?;
            read += kos_value.size_bytes();

            kos_value.hash(&mut hasher);
            let hash = hasher.finish();

            hashes.push(hash);
            data.push(kos_value);
        }

        if debug {
            println!("Data section:");

            for d in data.iter() {
                println!("\t{:?}", d);
            }
        }

        Ok(DataSection {
            hashes,
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

pub struct FuncSection {
    instructions: Vec<Instr>,
    size: usize,
    section_index: usize,
}

impl SectionIndex for FuncSection {
    fn section_index(&self) -> usize {
        self.section_index
    }
}

impl FuncSection {
    pub fn new(amount: usize, section_index: usize) -> Self {
        FuncSection {
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
}

impl SectionFromBytes for FuncSection {
    fn from_bytes(
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

        Ok(FuncSection {
            instructions,
            size,
            section_index,
        })
    }
}

impl ToBytes for FuncSection {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        for instr in self.instructions.iter() {
            instr.to_bytes(buf);
        }
    }
}

pub struct ReldSection {
    entries: Vec<ReldEntry>,
    size: u32,
    section_index: usize,
}

impl SectionIndex for ReldSection {
    fn section_index(&self) -> usize {
        self.section_index
    }
}

impl ReldSection {
    pub fn new(amount: usize, section_index: usize) -> Self {
        ReldSection {
            entries: Vec::with_capacity(amount),
            size: 0,
            section_index,
        }
    }

    pub fn add(&mut self, entry: ReldEntry) -> usize {
        self.size += entry.size_bytes();
        self.entries.push(entry);
        self.entries.len() - 1
    }

    pub fn get(&self, index: usize) -> Option<&ReldEntry> {
        self.entries.get(index)
    }

    pub fn entries(&self) -> Iter<ReldEntry> {
        self.entries.iter()
    }

    pub fn size(&self) -> u32 {
        self.size
    }
}

impl ToBytes for ReldSection {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        for reld_entry in self.entries.iter() {
            reld_entry.to_bytes(buf);
        }
    }
}

impl SectionFromBytes for ReldSection {
    fn from_bytes(
        source: &mut Peekable<Iter<u8>>,
        debug: bool,
        size: usize,
        section_index: usize,
    ) -> ReadResult<Self> {
        let mut read = 0;
        let mut entries = Vec::new();

        if debug {
            println!("Reld section:");
        }

        while read < size {
            let entry = ReldEntry::from_bytes(source, debug)?;
            read += entry.size_bytes() as usize;

            if debug {
                println!("\t{:?}", entry);
            }

            entries.push(entry);
        }

        Ok(ReldSection {
            entries,
            size: size as u32,
            section_index,
        })
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
        let mut strtab = StringTable::new(8, 0);
        let mut symtab = SymbolTable::new(1, 0);

        let s_index = strtab.add("fn_add");

        let sym = KOSymbol::new(s_index, 0, 0, SymBind::Local, SymType::Func, 3);

        let sym_index = symtab.add(sym);

        assert_eq!(sym_index, 0);
    }

    #[test]
    fn symtab_get() {
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
