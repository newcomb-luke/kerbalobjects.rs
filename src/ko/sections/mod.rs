//! A collection of the sections that can be contained within a KO file
use crate::errors::{ReadError, ReadResult};
use crate::{FileIterator, FromBytes, ToBytes};

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

/// An object that is a section, which can provide the index of itself in the section
/// header table.
pub trait SectionIndex {
    /// Returns the index into the section header table for this section
    fn section_index(&self) -> usize;
}

/// The kind of section of an entry in the section header table
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum SectionKind {
    /// A null section. There is always a Null section entry in the section header table
    /// which other parts of the object file can use to reference a section that does not exist.
    Null = 0,
    /// A symbol table
    SymTab = 1,
    /// A string table
    StrTab = 2,
    /// A function table
    Func = 3,
    /// A data section
    Data = 4,
    /// A debug section
    Debug = 5,
    /// A relocation data section
    Reld = 6,
}

impl TryFrom<u8> for SectionKind {
    type Error = ();

    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        match byte {
            0 => Ok(Self::Null),
            1 => Ok(Self::SymTab),
            2 => Ok(Self::StrTab),
            3 => Ok(Self::Func),
            4 => Ok(Self::Data),
            5 => Ok(Self::Debug),
            6 => Ok(Self::Reld),
            _ => Err(()),
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
        }
    }
}

impl ToBytes for SectionKind {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        buf.push((*self).into());
    }
}

impl FromBytes for SectionKind {
    fn from_bytes(source: &mut FileIterator) -> ReadResult<Self>
    where
        Self: Sized,
    {
        let value = source.next().ok_or(ReadError::SectionKindReadError)?;

        SectionKind::try_from(value).map_err(|_| ReadError::UnknownSectionKindReadError(value))
    }
}

/// A Kerbal Object file section header entry
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SectionHeader {
    /// The index into the KO file's .shstrtab (section header string table) which stores
    /// the name of this section.
    pub name_idx: usize,
    /// The section's kind
    pub section_kind: SectionKind,
    /// The size of this section in bytes
    pub size: u32,
}

impl SectionHeader {
    /// Creates a new null section header entry, which **must** be present as the first entry into
    /// the section header table
    pub fn null() -> Self {
        SectionHeader {
            name_idx: 0,
            section_kind: SectionKind::Null,
            size: 0,
        }
    }

    /// Creates a new section header entry
    pub fn new(name_idx: usize, section_kind: SectionKind) -> Self {
        Self {
            name_idx,
            section_kind,
            size: 0,
        }
    }
}

impl ToBytes for SectionHeader {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        (self.name_idx as u32).to_bytes(buf);
        self.section_kind.to_bytes(buf);
        self.size.to_bytes(buf);
    }
}

impl FromBytes for SectionHeader {
    fn from_bytes(source: &mut FileIterator) -> ReadResult<Self>
    where
        Self: Sized,
    {
        let name_idx = u32::from_bytes(source)
            .map_err(|_| ReadError::SectionHeaderConstantReadError("name index"))?
            as usize;
        let section_kind = SectionKind::from_bytes(source)?;
        let size = u32::from_bytes(source)
            .map_err(|_| ReadError::SectionHeaderConstantReadError("size"))?;

        Ok(Self {
            name_idx,
            section_kind,
            size,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ko::symbols::{SymBind, SymType};
    use crate::KOSValue;

    #[test]
    fn strtab_insert() {
        let mut strtab = StringTable::with_capacity(8, 0);

        let index = strtab.add(".text");

        assert_eq!(index, 1);
        assert_eq!(strtab.get(index).unwrap(), ".text");
    }

    #[test]
    fn strtab_get_null() {
        let strtab = StringTable::with_capacity(8, 0);

        assert_eq!(strtab.get(0).unwrap(), "");
    }

    #[test]
    fn strtab_get_str() {
        let mut strtab = StringTable::with_capacity(8, 0);

        strtab.add("Hello");

        assert_eq!(strtab.get(1).unwrap(), "Hello");
    }

    #[test]
    fn strtab_get_str_2() {
        let mut strtab = StringTable::with_capacity(16, 0);

        strtab.add("Hello");

        let index = strtab.add("world");

        assert_eq!(index, 2);
        assert_eq!(strtab.get(index).unwrap(), "world");
    }

    #[test]
    fn strtab_strings() {
        let mut strtab = StringTable::with_capacity(16, 0);

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
        use crate::ko::symbols::KOSymbol;

        let mut strtab = StringTable::with_capacity(8, 0);
        let mut symtab = SymbolTable::with_capacity(1, 0);

        let s_index = strtab.add("fn_add");

        let sym = KOSymbol::new(s_index, 0, 0, SymBind::Local, SymType::Func, 3);

        let sym_index = symtab.add(sym);

        assert_eq!(sym_index, 0);
    }

    #[test]
    fn symtab_get() {
        use crate::ko::symbols::KOSymbol;

        let mut strtab = StringTable::with_capacity(8, 0);
        let mut symtab = SymbolTable::with_capacity(1, 0);

        let s_index = strtab.add("fn_add");

        let sym = KOSymbol::new(s_index, 0, 0, SymBind::Local, SymType::Func, 3);

        let sym_index = symtab.add(sym);

        assert_eq!(
            strtab.get(symtab.get(sym_index).unwrap().name_idx).unwrap(),
            "fn_add"
        );
    }

    #[test]
    fn data_insert() {
        let mut data_section = DataSection::with_capacity(1, 0);

        let index = data_section.add(KOSValue::Int32(365));

        assert_eq!(index, 0);
    }

    #[test]
    fn data_insert_2() {
        let mut data_section = DataSection::with_capacity(2, 0);

        data_section.add(KOSValue::ArgMarker);

        let index = data_section.add(KOSValue::Bool(true));

        assert_eq!(index, 1);
    }

    #[test]
    fn data_get() {
        let mut data_section = DataSection::with_capacity(1, 0);

        let index = data_section.add(KOSValue::Int16(657));

        assert_eq!(data_section.get(index).unwrap(), &KOSValue::Int16(657));
    }

    #[test]
    fn write_header() {
        let mut sh = SectionHeader::new(0, SectionKind::SymTab);
        sh.size = 42;

        let mut buf = Vec::new();

        sh.to_bytes(&mut buf);

        assert_eq!(
            buf,
            vec![0x00, 0x00, 0x00, 0x00, 0x01, 0x2a, 0x00, 0x00, 0x00]
        );
    }
}
