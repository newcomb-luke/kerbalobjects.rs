//! A collection of the sections that can be contained within a KO file
use crate::{BufferIterator, FromBytes, ToBytes, WritableBuffer};

mod data_section;
mod func_section;
mod reld_section;
mod string_table;
mod symbol_table;

use crate::ko::errors::SectionHeaderParseError;
pub use data_section::*;
pub use func_section::*;
pub use reld_section::*;
pub use string_table::*;
pub use symbol_table::*;

/// The kind of section of an entry in the section header table
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
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

/// A Kerbal Object file section header entry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SectionHeader {
    /// The index into the KO file's .shstrtab (section header string table) which stores
    /// the name of this section.
    pub name_idx: StringIdx,
    /// The section's kind
    pub section_kind: SectionKind,
    /// The size of this *section* in bytes
    pub size: u32,
}

impl SectionHeader {
    /// A constant for the null section header
    pub const NULL: SectionHeader = SectionHeader::null();

    /// Creates a new null section header entry, which **must** be present as the first entry into
    /// the section header table
    pub const fn null() -> Self {
        Self {
            name_idx: StringIdx::EMPTY,
            section_kind: SectionKind::Null,
            size: 0,
        }
    }

    /// Creates a new section header entry
    pub fn new(name_idx: StringIdx, section_kind: SectionKind) -> Self {
        Self {
            name_idx,
            section_kind,
            size: 0,
        }
    }

    /// Parses a SectionHeader from the provided byte buffer
    pub fn parse(source: &mut BufferIterator) -> Result<Self, SectionHeaderParseError> {
        let name_idx = StringIdx::from(
            u32::from_bytes(source).map_err(|_| SectionHeaderParseError::MissingNameIdxError)?,
        );

        let section_kind_raw = source.next().ok_or_else(|| {
            SectionHeaderParseError::MissingSectionKindError(source.current_index())
        })?;
        let section_kind = SectionKind::try_from(section_kind_raw).map_err(|_| {
            SectionHeaderParseError::InvalidSectionKindError(
                source.current_index(),
                section_kind_raw,
            )
        })?;
        let size = u32::from_bytes(source)
            .map_err(|_| SectionHeaderParseError::MissingSizeError(source.current_index()))?;

        Ok(Self {
            name_idx,
            section_kind,
            size,
        })
    }

    /// Converts this section header to its binary representation and appends it to the provided buffer
    pub fn write(&self, buf: &mut impl WritableBuffer) {
        (usize::from(self.name_idx) as u32).to_bytes(buf);
        buf.write(self.section_kind as u8);
        self.size.to_bytes(buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ko::symbols::{SymBind, SymType};
    use crate::ko::SectionIdx;
    use crate::KOSValue;

    #[test]
    fn strtab_insert() {
        let mut strtab = StringTable::with_capacity(8, SectionIdx::NULL);

        let index = strtab.add(".text");

        assert_eq!(index, StringIdx::from(1usize));
        assert_eq!(strtab.get(index).unwrap(), ".text");
    }

    #[test]
    fn strtab_get_null() {
        let strtab = StringTable::with_capacity(8, SectionIdx::NULL);

        assert_eq!(strtab.get(StringIdx::EMPTY).unwrap(), "");
    }

    #[test]
    fn strtab_get_str() {
        let mut strtab = StringTable::with_capacity(8, SectionIdx::NULL);

        strtab.add("Hello");

        assert_eq!(strtab.get(StringIdx::from(1usize)).unwrap(), "Hello");
    }

    #[test]
    fn strtab_get_str_2() {
        let mut strtab = StringTable::with_capacity(16, SectionIdx::NULL);

        strtab.add("Hello");

        let index = strtab.add("world");

        assert_eq!(index, StringIdx::from(2usize));
        assert_eq!(strtab.get(index).unwrap(), "world");
    }

    #[test]
    fn strtab_strings() {
        let mut strtab = StringTable::with_capacity(16, SectionIdx::NULL);

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
    fn strtab_read_write() {
        let mut strtab = StringTable::with_capacity(2, SectionIdx::NULL);

        strtab.add("strings");
        strtab.add("in");
        strtab.add("rust");

        let mut buffer = Vec::new();

        strtab.write(&mut buffer);

        let mut iterator = BufferIterator::new(&buffer);

        let read_strtab =
            StringTable::parse(&mut iterator, strtab.size(), strtab.section_index()).unwrap();

        for (s1, s2) in strtab.strings().zip(read_strtab.strings()) {
            assert_eq!(s1, s2);
        }
    }

    #[test]
    fn symtab_insert() {
        use crate::ko::symbols::KOSymbol;

        let mut strtab = StringTable::with_capacity(8, SectionIdx::NULL);
        let mut symtab = SymbolTable::with_capacity(1, SectionIdx::NULL);

        let s_index = strtab.add("fn_add");

        let sym = KOSymbol::new(
            s_index,
            DataIdx::from(0u32),
            0,
            SymBind::Local,
            SymType::Func,
            SectionIdx::from(3u16),
        );

        let sym_index = symtab.add(sym);

        assert_eq!(sym_index, SymbolIdx::from(0u32));
    }

    #[test]
    fn symtab_get() {
        use crate::ko::symbols::KOSymbol;

        let mut strtab = StringTable::with_capacity(8, SectionIdx::NULL);
        let mut symtab = SymbolTable::with_capacity(1, SectionIdx::NULL);

        let s_index = strtab.add("fn_add");

        let sym = KOSymbol::new(
            s_index,
            DataIdx::from(0u32),
            0,
            SymBind::Local,
            SymType::Func,
            SectionIdx::from(3u16),
        );

        let sym_index = symtab.add(sym);

        assert_eq!(
            strtab.get(symtab.get(sym_index).unwrap().name_idx).unwrap(),
            "fn_add"
        );
    }

    #[test]
    fn data_insert() {
        let mut data_section = DataSection::with_capacity(1, SectionIdx::NULL);

        let index = data_section.add(KOSValue::Int32(365));

        assert_eq!(index, DataIdx::from(0u32));
    }

    #[test]
    fn data_insert_2() {
        let mut data_section = DataSection::with_capacity(2, SectionIdx::NULL);

        data_section.add(KOSValue::ArgMarker);

        let index = data_section.add(KOSValue::Bool(true));

        assert_eq!(index, DataIdx::from(1u32));
    }

    #[test]
    fn read_write_data() {
        let mut buffer = Vec::new();

        let mut data_section = DataSection::new(SectionIdx::from(2u16));

        data_section.add_checked(KOSValue::ArgMarker);
        data_section.add_checked(KOSValue::Null);
        data_section.add_checked(KOSValue::ScalarInt(2));
        data_section.add_checked(KOSValue::ScalarInt(4));
        data_section.add_checked(KOSValue::String(String::from("test")));

        data_section.write(&mut buffer);

        let mut iter = BufferIterator::new(&buffer);

        let read =
            DataSection::parse(&mut iter, data_section.size(), SectionIdx::from(2u16)).unwrap();

        assert_eq!(read, data_section);
    }

    #[test]
    fn data_get() {
        let mut data_section = DataSection::with_capacity(1, SectionIdx::NULL);

        let index = data_section.add(KOSValue::Int16(657));

        assert_eq!(data_section.get(index).unwrap(), &KOSValue::Int16(657));
    }

    #[test]
    fn write_header() {
        let mut sh = SectionHeader::new(StringIdx::from(0usize), SectionKind::SymTab);
        sh.size = 42;

        let mut buf = Vec::new();

        sh.write(&mut buf);

        assert_eq!(
            buf,
            vec![0x00, 0x00, 0x00, 0x00, 0x01, 0x2a, 0x00, 0x00, 0x00]
        );
    }
}
