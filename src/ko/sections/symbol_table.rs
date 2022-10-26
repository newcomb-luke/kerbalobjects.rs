//! A module describing a symbol table section in a Kerbal Object file. This section stores
//! a list of all symbols that this object file stores and references.
use crate::ko::errors::SymbolTableParseError;
use crate::ko::sections::StringIdx;
use crate::ko::symbols::KOSymbol;
use crate::ko::SectionIdx;
use crate::{BufferIterator, WritableBuffer};
use std::collections::HashMap;
use std::slice::Iter;

/// A wrapper type that represents an index into a symbol table of a KO file.
///
/// This type implements From<usize> and usize implements From<SymbolIdx>, but this is provided
/// so that it takes 1 extra step to convert raw integers into SymbolIdx which could stop potential
/// logical bugs.
///
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct SymbolIdx(u32);

impl From<usize> for SymbolIdx {
    fn from(i: usize) -> Self {
        Self(i as u32)
    }
}

impl From<u8> for SymbolIdx {
    fn from(i: u8) -> Self {
        Self(i as u32)
    }
}

impl From<u16> for SymbolIdx {
    fn from(i: u16) -> Self {
        Self(i as u32)
    }
}

impl From<u32> for SymbolIdx {
    fn from(i: u32) -> Self {
        Self(i)
    }
}

impl From<SymbolIdx> for u32 {
    fn from(sym_idx: SymbolIdx) -> Self {
        sym_idx.0
    }
}

impl From<SymbolIdx> for usize {
    fn from(sym_idx: SymbolIdx) -> Self {
        sym_idx.0 as usize
    }
}

/// A symbol table section in a KerbalObject file.
///
/// This section stores a list of all symbols that this object file stores and references.
///
#[derive(Debug)]
pub struct SymbolTable {
    symbols: Vec<KOSymbol>,
    name_map: HashMap<StringIdx, usize>,
    size: u32,
    section_index: SectionIdx,
}

impl SymbolTable {
    /// Creates a new symbol table section with the provided section index
    pub fn new(section_index: SectionIdx) -> Self {
        Self {
            symbols: Vec::new(),
            name_map: HashMap::new(),
            size: 0,
            section_index,
        }
    }

    /// Creates a new symbol table section with the provided section index, with
    /// internal data structures pre-allocated for the provided amount of symbols
    pub fn with_capacity(amount: usize, section_index: SectionIdx) -> Self {
        Self {
            symbols: Vec::with_capacity(amount),
            name_map: HashMap::with_capacity(amount),
            size: 0,
            section_index,
        }
    }

    /// Gets the symbol at the provided index into this symbol table, or None if
    /// it doesn't exist.
    pub fn get(&self, index: SymbolIdx) -> Option<&KOSymbol> {
        self.symbols.get(usize::from(index))
    }

    /// Returns the KOSymbol by the provided name index (from the .symstrtab), or None
    /// if it doesn't exist
    pub fn find_by_name(&self, name_idx: StringIdx) -> Option<&KOSymbol> {
        self.symbols.get(*self.name_map.get(&name_idx)?)
    }

    /// Returns the position/index in this symbol table of the symbol of the provided
    /// name index (from the .symstrtab), or None if it doesn't exist
    pub fn position_by_name(&self, name_idx: StringIdx) -> Option<SymbolIdx> {
        self.name_map
            .get(&name_idx)
            .map(|i| SymbolIdx::from(*i as u32))
    }

    /// Returns the position/index in this symbol table of the provided
    /// symbol, or None if it doesn't exist
    pub fn find(&self, symbol: &KOSymbol) -> Option<SymbolIdx> {
        self.symbols
            .iter()
            .position(|sym| sym == symbol)
            .map(|i| SymbolIdx::from(i))
    }

    /// Adds a new symbol to this symbol table, returning the index of the symbol
    /// into this section. Doesn't do anything like checking if a symbol with that name already
    /// exists in the table
    pub fn add(&mut self, symbol: KOSymbol) -> SymbolIdx {
        self.size += KOSymbol::size_bytes();
        self.name_map.insert(symbol.name_idx, self.symbols.len());
        self.symbols.push(symbol);
        SymbolIdx::from(self.symbols.len() - 1)
    }

    /// Adds a new symbol to this symbol table, returning the index of the symbol
    /// into this section. Checks if a symbol with the same name is already in the table,
    /// and in that case just returns the index of that symbol
    pub fn add_checked(&mut self, symbol: KOSymbol) -> SymbolIdx {
        if let Some(i) = self.name_map.get(&symbol.name_idx) {
            SymbolIdx::from(*i)
        } else {
            self.add(symbol)
        }
    }

    /// The size of this symbol table section in bytes
    pub fn size(&self) -> u32 {
        self.size
    }

    /// Returns an iterator over all symbols in this symbol table
    pub fn symbols(&self) -> Iter<KOSymbol> {
        self.symbols.iter()
    }

    /// The index of this section's section header
    pub fn section_index(&self) -> SectionIdx {
        self.section_index
    }

    /// Parses a symbol table from the provided byte buffer
    pub fn parse(
        source: &mut BufferIterator,
        size: u32,
        section_index: SectionIdx,
    ) -> Result<Self, SymbolTableParseError> {
        let num_symbols = size / KOSymbol::size_bytes();
        let mut num_read_symbols = 0;

        let mut sym_tab = SymbolTable::with_capacity(num_symbols as usize, section_index);

        while num_read_symbols * KOSymbol::size_bytes() < size {
            let symbol = KOSymbol::parse(source).map_err(|e| {
                SymbolTableParseError::SymbolParseError(
                    num_read_symbols as usize,
                    source.current_index(),
                    e,
                )
            })?;
            num_read_symbols += 1;

            sym_tab.add(symbol);
        }

        Ok(sym_tab)
    }

    /// Converts this symbol table to its binary representation and appends it to the provided buffer
    pub fn write(&self, buf: &mut impl WritableBuffer) {
        for symbol in self.symbols.iter() {
            symbol.write(buf);
        }
    }
}
