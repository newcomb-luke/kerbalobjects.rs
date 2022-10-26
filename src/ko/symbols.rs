//! Describes symbols contained within a KO file's symbol table
use crate::{BufferIterator, FromBytes, ToBytes, WritableBuffer};

use crate::ko::errors::{ReldEntryParseError, SymbolParseError};
use crate::ko::sections::{DataIdx, InstrIdx, StringIdx, SymbolIdx};
use crate::ko::SectionIdx;

/// Represents the "binding" of a symbol in the symbol table.
///
/// This is akin to the visibility level of the symbol. Is it a "local" symbol, where it can
/// only be recognized within this KO file, or is it global, and can be linked against from
/// another file?
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
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

/// This represents the type of symbol.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[repr(u8)]
pub enum SymType {
    /// No type, used for any type of data, such as KOSValues in a data section.
    NoType = 0,
    /// An object. Specified for forward-compatibility, currently unused.
    Object = 1,
    /// A function defined elsewhere in the file.
    Func = 2,
    /// A reference to a section of this object file.
    Section = 3,
    /// The filename of the file that produced the object file.
    /// Only one of these can be specified in each object file.
    File = 4,
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

/// This represents the operand index of an operand to replace from a relocation data section,
/// because instructions can have 0, 1, or 2 operands, this value can only be 1 or 2, because those
/// are the only values that make sense. This provides a certain degree of type safety.
///
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[repr(u8)]
pub enum OperandIndex {
    /// The first operand
    One = 1,
    /// The second operand
    Two = 2,
}

impl TryFrom<u8> for OperandIndex {
    type Error = ();

    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        match byte {
            1 => Ok(Self::One),
            2 => Ok(Self::Two),
            _ => Err(()),
        }
    }
}

impl From<OperandIndex> for u8 {
    fn from(index: OperandIndex) -> u8 {
        match index {
            OperandIndex::One => 1,
            OperandIndex::Two => 2,
        }
    }
}

/// Represents a symbol in a symbol table in a Kerbal Object file
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KOSymbol {
    /// The index into the .symstrtab that is the name of this symbol
    pub name_idx: StringIdx,
    /// The index into the .data section that this symbol refers to
    pub value_idx: DataIdx,
    /// The size of the data this symbol refers to in bytes
    pub size: u16,
    /// The symbol binding of this symbol
    pub sym_bind: SymBind,
    /// The symbol type
    pub sym_type: SymType,
    /// The section header index this symbol refers to
    pub sh_idx: SectionIdx,
}

impl KOSymbol {
    // The size of a KO symbol in bytes
    const SYMBOL_SIZE: u32 = 14;

    /// Creates a new KOSymbol
    pub const fn new(
        name_idx: StringIdx,
        value_idx: DataIdx,
        size: u16,
        sym_bind: SymBind,
        sym_type: SymType,
        sh_idx: SectionIdx,
    ) -> Self {
        Self {
            name_idx,
            value_idx,
            size,
            sym_bind,
            sym_type,
            sh_idx,
        }
    }

    /// Returns the size of a symbol table entry in bytes
    pub const fn size_bytes() -> u32 {
        Self::SYMBOL_SIZE
    }

    /// Parses a KOSymbol from the provided buffer
    pub fn parse(source: &mut BufferIterator) -> Result<Self, SymbolParseError> {
        let name_idx = StringIdx::from(
            u32::from_bytes(source).map_err(|_| SymbolParseError::MissingNameIndexError)?,
        );
        let value_idx = DataIdx::from(
            u32::from_bytes(source).map_err(|_| SymbolParseError::MissingValueIndexError)?,
        );
        let size = u16::from_bytes(source).map_err(|_| SymbolParseError::MissingSymbolSizeError)?;
        let raw_sym_bind =
            u8::from_bytes(source).map_err(|_| SymbolParseError::MissingSymbolBindingError)?;
        let sym_bind = SymBind::try_from(raw_sym_bind)
            .map_err(|_| SymbolParseError::InvalidSymbolBindingError(raw_sym_bind))?;
        let raw_sym_type =
            u8::from_bytes(source).map_err(|_| SymbolParseError::MissingSymbolTypeError)?;
        let sym_type = SymType::try_from(raw_sym_type)
            .map_err(|_| SymbolParseError::InvalidSymbolTypeError(raw_sym_type))?;
        let sh_idx = SectionIdx::from(
            u16::from_bytes(source).map_err(|_| SymbolParseError::MissingSectionIndexError)?,
        );

        Ok(Self {
            name_idx,
            value_idx,
            size,
            sym_bind,
            sym_type,
            sh_idx,
        })
    }

    /// Converts this KOSymbol to its binary representation and appends it to the provided buffer
    pub fn write(&self, buf: &mut impl WritableBuffer) {
        (usize::from(self.name_idx) as u32).to_bytes(buf);
        u32::from(self.value_idx).to_bytes(buf);
        self.size.to_bytes(buf);
        buf.write(self.sym_bind as u8);
        buf.write(self.sym_type as u8);
        u16::from(self.sh_idx).to_bytes(buf);
    }
}

/// An entry in a relocation data section
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReldEntry {
    /// The function section which this relocation applies to
    section_index: SectionIdx,
    /// The index of the instruction whose operand needs to be "relocated"
    instr_index: InstrIdx,
    /// The operand of the instruction which needs to be "relocated"
    operand_index: OperandIndex,
    /// The index into the symbol table which should replace this operand
    symbol_index: SymbolIdx,
}

impl ReldEntry {
    /// The size of a reld entry in bytes
    const RELD_ENTRY_SIZE: u32 = (std::mem::size_of::<u16>()
        + std::mem::size_of::<u32>()
        + std::mem::size_of::<u8>()
        + std::mem::size_of::<u32>()) as u32;

    /// Creates a new relocation data entry
    pub const fn new(
        section_index: SectionIdx,
        instr_index: InstrIdx,
        operand_index: OperandIndex,
        symbol_index: SymbolIdx,
    ) -> Self {
        Self {
            section_index,
            instr_index,
            operand_index,
            symbol_index,
        }
    }

    /// The size of this relocation data entry in bytes
    pub const fn size_bytes(&self) -> u32 {
        Self::RELD_ENTRY_SIZE
    }

    /// Parses a ReldEntry from the provided byte buffer
    pub fn parse(source: &mut BufferIterator) -> Result<Self, ReldEntryParseError> {
        let section_index = SectionIdx::from(
            u16::from_bytes(source).map_err(|_| ReldEntryParseError::MissingSectionIndexError)?,
        );
        let instr_index = InstrIdx::from(
            u32::from_bytes(source)
                .map_err(|_| ReldEntryParseError::MissingInstructionIndexError)?,
        );
        let raw_operand_index =
            u8::from_bytes(source).map_err(|_| ReldEntryParseError::MissingOperandIndexError)?;
        let operand_index = OperandIndex::try_from(raw_operand_index)
            .map_err(|_| ReldEntryParseError::InvalidOperandIndexError(raw_operand_index))?;
        let symbol_index = SymbolIdx::from(
            u32::from_bytes(source).map_err(|_| ReldEntryParseError::MissingSymbolIndexError)?,
        );

        Ok(Self {
            section_index,
            instr_index,
            operand_index,
            symbol_index,
        })
    }

    /// Converts this ReldEntry to its binary representation and appends it to the provided buffer
    pub fn write(&self, buf: &mut impl WritableBuffer) {
        u16::from(self.section_index).to_bytes(buf);
        u32::from(self.instr_index).to_bytes(buf);
        u8::from(self.operand_index).to_bytes(buf);
        u32::from(self.symbol_index).to_bytes(buf);
    }
}
