use crate::kofile::sections::SectionIndex;
use crate::kofile::symbols::KOSymbol;
use crate::kofile::SectionFromBytes;
use crate::{FromBytes, ReadResult, ToBytes};
use std::iter::Peekable;
use std::slice::Iter;

#[derive(Debug)]
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
            symbols: Vec::with_capacity(amount),
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

    pub fn position_by_name(&self, name_idx: usize) -> Option<usize> {
        self.symbols
            .iter()
            .position(|sym| sym.name_idx() == name_idx)
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
        size: usize,
        section_index: usize,
    ) -> ReadResult<Self> {
        let num_symbols = size / KOSymbol::size_bytes() as usize;
        let mut read_symbols = 0;

        let mut sym_tab = SymbolTable::new(num_symbols as usize, section_index);

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
