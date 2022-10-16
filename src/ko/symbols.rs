//! Describes symbols contained within a KO file's symbol table
use crate::{FileIterator, FromBytes, ToBytes};

use crate::errors::{ReadError, ReadResult};

/// Represents the "binding" of a symbol in the symbol table.
///
/// This is akin to the visibility level of the symbol. Is it a "local" symbol, where it can
/// only be recognized within this KO file, or is it global, and can be linked against from
/// another file?
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum SymBind {
    /// This symbol is only visible within the current object file.
    Local = 0,
    /// This symbol is visible to the linker from other object files.
    Global = 1,
    /// This symbol doesn't exist in the current object file, it is external.
    /// This should be matched with a corresponding symbol of the same name marked as Global
    Extern = 2,
}

impl TryFrom<u8> for SymBind {
    type Error = ();

    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        match byte {
            0 => Ok(Self::Local),
            1 => Ok(Self::Global),
            2 => Ok(Self::Extern),
            _ => Err(()),
        }
    }
}

impl From<SymBind> for u8 {
    fn from(bind: SymBind) -> u8 {
        match bind {
            SymBind::Local => 0,
            SymBind::Global => 1,
            SymBind::Extern => 2,
        }
    }
}

impl ToBytes for SymBind {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        buf.push((*self).into());
    }
}

impl FromBytes for SymBind {
    fn from_bytes(source: &mut FileIterator) -> ReadResult<Self>
    where
        Self: Sized,
    {
        let value = source.next().ok_or(ReadError::SymBindReadError)?;

        SymBind::try_from(value).map_err(|_| ReadError::UnknownSimBindReadError(value))
    }
}

/// This represents the type of symbol.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum SymType {
    /// No type, used for any type of data, such as KOSValues in a data section.
    NoType,
    /// An object. Specified for forward-compatibility, currently unused.
    Object,
    /// A function defined elsewhere in the file.
    Func,
    /// A reference to a section of this object file.
    Section,
    /// The filename of the file that produced the object file.
    /// Only one of these can be specified in each object file.
    File,
}

impl TryFrom<u8> for SymType {
    type Error = ();

    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        match byte {
            0 => Ok(Self::NoType),
            1 => Ok(Self::Object),
            2 => Ok(Self::Func),
            3 => Ok(Self::Section),
            4 => Ok(Self::File),
            _ => Err(()),
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
        }
    }
}

impl ToBytes for SymType {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        buf.push((*self).into());
    }
}

impl FromBytes for SymType {
    fn from_bytes(source: &mut FileIterator) -> ReadResult<Self>
    where
        Self: Sized,
    {
        let value = source.next().ok_or(ReadError::SymTypeReadError)?;

        SymType::try_from(value).map_err(|_| ReadError::UnknownSimTypeReadError(value))
    }
}

/// Represents a symbol in a symbol table in a Kerbal Object file
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KOSymbol {
    /// The index into the .symstrtab that is the name of this symbol
    pub name_idx: usize,
    /// The index into the .data section that this symbol refers to
    pub value_idx: usize,
    /// The size of the data this symbol refers to in bytes
    pub size: u16,
    /// The symbol binding of this symbol
    pub sym_bind: SymBind,
    /// The symbol type
    pub sym_type: SymType,
    /// The section header index this symbol refers to
    pub sh_idx: u16,
}

impl KOSymbol {
    /// Creates a new KOSymbol
    pub fn new(
        name_idx: usize,
        value_idx: usize,
        size: u16,
        sym_bind: SymBind,
        sym_type: SymType,
        sh_idx: u16,
    ) -> Self {
        KOSymbol {
            name_idx,
            value_idx,
            size,
            sym_bind,
            sym_type,
            sh_idx,
        }
    }

    /// Returns the size of a symbol table entry in bytes
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
    fn from_bytes(source: &mut FileIterator) -> ReadResult<Self>
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

/// An entry in a relocation data section
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReldEntry {
    /// The function section which this relocation applies to
    section_index: u32,
    /// The index of the instruction whose operand needs to be "relocated"
    instr_index: u32,
    /// The operand of the instruction which needs to be "relocated"
    operand_index: u8,
    /// The index into the symbol table which should replace this operand
    symbol_index: u32,
}

impl ReldEntry {
    /// Creates a new relocation data entry
    pub fn new(section_index: u32, instr_index: u32, operand_index: u8, symbol_index: u32) -> Self {
        ReldEntry {
            section_index,
            instr_index,
            operand_index,
            symbol_index,
        }
    }

    /// The size of this relocation data entry in bytes
    pub fn size_bytes(&self) -> u32 {
        4 * 4
    }
}

impl ToBytes for ReldEntry {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        self.section_index.to_bytes(buf);
        self.instr_index.to_bytes(buf);
        self.operand_index.to_bytes(buf);
        self.symbol_index.to_bytes(buf);
    }
}

// TODO: Docs but also rewrite
impl FromBytes for ReldEntry {
    fn from_bytes(source: &mut FileIterator) -> ReadResult<Self> {
        let section_index = u32::from_bytes(source).map_err(|_| ReadError::ReldReadError)?;
        let instr_index = u32::from_bytes(source).map_err(|_| ReadError::ReldReadError)?;
        let operand_index = u8::from_bytes(source).map_err(|_| ReadError::ReldReadError)?;
        let symbol_index = u32::from_bytes(source).map_err(|_| ReadError::ReldReadError)?;

        Ok(ReldEntry {
            section_index,
            instr_index,
            operand_index,
            symbol_index,
        })
    }
}
