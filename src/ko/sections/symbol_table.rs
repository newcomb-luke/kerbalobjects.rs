//! A module describing a symbol table section in a Kerbal Object file. This section stores
//! a list of all symbols that this object file stores and references.
use crate::ko::sections::SectionIndex;
use crate::ko::symbols::KOSymbol;
use crate::ko::SectionFromBytes;
use crate::{FileIterator, FromBytes, ReadResult, ToBytes};
use std::slice::Iter;

/// A symbol table section in a KerbalObject file.
///
/// This section stores a list of all symbols that this object file stores and references.
///
#[derive(Debug)]
pub struct SymbolTable {
    symbols: Vec<KOSymbol>,
    size: usize,
    /// The index of this section's section header
    pub section_index: usize,
}

impl SectionIndex for SymbolTable {
    fn section_index(&self) -> usize {
        self.section_index
    }
}

impl SymbolTable {
    /// Creates a new symbol table section with the provided section index
    pub fn new(section_index: usize) -> Self {
        Self {
            symbols: Vec::new(),
            size: 0,
            section_index,
        }
    }

    /// Creates a new symbol table section with the provided section index, with
    /// internal data structures pre-allocated for the provided amount of items
    pub fn with_capacity(amount: usize, section_index: usize) -> Self {
        Self {
            symbols: Vec::with_capacity(amount),
            size: 0,
            section_index,
        }
    }

    /// Gets the symbol at the provided index into this symbol table, or None if
    /// it doesn't exist.
    pub fn get(&self, index: usize) -> Option<&KOSymbol> {
        self.symbols.get(index)
    }

    /// Returns the KOSymbol by the provided name index (from the .symstrtab), or None
    /// if it doesn't exist
    pub fn find_has_name(&self, name_idx: usize) -> Option<&KOSymbol> {
        self.symbols.iter().find(|sym| sym.name_idx == name_idx)
    }

    /// Returns the position/index in this symbol table of the symbol of the provided
    /// name index (from the .symstrtab), or None if it doesn't exist
    pub fn position_by_name(&self, name_idx: usize) -> Option<usize> {
        self.symbols.iter().position(|sym| sym.name_idx == name_idx)
    }

    /// Returns the position/index in this symbol table of the provided
    /// symbol, or None if it doesn't exist
    pub fn find(&self, symbol: &KOSymbol) -> Option<usize> {
        self.symbols.iter().position(|sym| sym == symbol)
    }

    /// Adds a new symbol to this symbol table, returning the index of the symbol
    /// into this section.
    pub fn add(&mut self, symbol: KOSymbol) -> usize {
        self.size += KOSymbol::size_bytes() as usize;
        self.symbols.push(symbol);
        self.symbols.len() - 1
    }

    /// The size of this symbol table section in bytes
    pub fn size(&self) -> u32 {
        self.size as u32
    }

    /// Returns an iterator over all symbols in this symbol table
    pub fn symbols(&self) -> Iter<KOSymbol> {
        self.symbols.iter()
    }
}

impl SectionFromBytes for SymbolTable {
    fn from_bytes(
        source: &mut FileIterator,
        size: usize,
        section_index: usize,
    ) -> ReadResult<Self> {
        let num_symbols = size / KOSymbol::size_bytes() as usize;
        let mut read_symbols = 0;

        let mut sym_tab = SymbolTable::with_capacity(num_symbols as usize, section_index);

        while (read_symbols * KOSymbol::size_bytes() as usize) < size {
            let symbol = KOSymbol::from_bytes(source)?;
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
