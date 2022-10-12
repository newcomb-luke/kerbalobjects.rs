use crate::{FromBytes, ReadError, ReadResult, ToBytes};
use std::iter::Peekable;
use std::slice::Iter;

#[derive(Debug, Clone, Copy)]
pub struct DebugRange {
    start: usize,
    end: usize,
}

impl DebugRange {
    pub fn new(start: usize, end: usize) -> Self {
        DebugRange { start, end }
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }

    pub fn to_bytes(&self, buf: &mut Vec<u8>, range_size: usize) {
        DebugRange::write_variable(buf, self.start, range_size);
        DebugRange::write_variable(buf, self.end, range_size);
    }

    fn write_variable(buf: &mut Vec<u8>, data: usize, size: usize) {
        match size {
            1 => {
                (data as u8).to_bytes(buf);
            }
            2 => {
                (data as u16).to_bytes(buf);
            }
            3 => {
                let converted = data as u32;
                let upper = (data >> 16) as u8;
                let lower = (converted & 0x0000ffff) as u16;

                upper.to_bytes(buf);
                lower.to_bytes(buf);
            }
            4 => {
                (data as u32).to_bytes(buf);
            }
            _ => unreachable!(),
        }
    }

    pub fn from_bytes(source: &mut Peekable<Iter<u8>>, range_size: usize) -> ReadResult<Self> {
        let start = DebugRange::read_variable(source, range_size)?;
        let end = DebugRange::read_variable(source, range_size)?;

        Ok(DebugRange { start, end })
    }

    fn read_variable(source: &mut Peekable<Iter<u8>>, range_size: usize) -> ReadResult<usize> {
        Ok(match range_size {
            1 => (u8::from_bytes(source).map_err(|_| ReadError::DebugRangeReadError))? as usize,
            2 => (u16::from_bytes(source).map_err(|_| ReadError::DebugRangeReadError))? as usize,
            3 => {
                let mut upper =
                    u8::from_bytes(source).map_err(|_| ReadError::DebugRangeReadError)? as u32;
                upper <<= 24;
                let lower =
                    u16::from_bytes(source).map_err(|_| ReadError::DebugRangeReadError)? as u32;

                (upper | lower) as usize
            }
            4 => (u32::from_bytes(source).map_err(|_| ReadError::DebugRangeReadError))? as usize,
            _ => unreachable!(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct DebugEntry {
    line_number: isize,
    number_ranges: usize,
    ranges: Vec<DebugRange>,
}

impl DebugEntry {
    pub fn new(line_number: isize) -> Self {
        DebugEntry {
            line_number,
            number_ranges: 0,
            ranges: Vec::new(),
        }
    }

    pub fn add(&mut self, range: DebugRange) {
        self.ranges.push(range);
        self.number_ranges += 1;
    }

    pub fn line_number(&self) -> isize {
        self.line_number
    }

    pub fn number_ranges(&self) -> usize {
        self.number_ranges
    }

    pub fn ranges(&self) -> Iter<DebugRange> {
        self.ranges.iter()
    }

    pub fn get(&self, index: usize) -> Option<&DebugRange> {
        self.ranges.get(index)
    }

    pub fn to_bytes(&self, buf: &mut Vec<u8>, range_size: usize) {
        (self.line_number as i16).to_bytes(buf);
        (self.number_ranges as u8).to_bytes(buf);

        for range in self.ranges.iter() {
            range.to_bytes(buf, range_size);
        }
    }

    pub fn from_bytes(source: &mut Peekable<Iter<u8>>, range_size: usize) -> ReadResult<Self> {
        let line_number =
            i16::from_bytes(source).map_err(|_| ReadError::DebugEntryReadError)? as isize;
        let number_ranges =
            u8::from_bytes(source).map_err(|_| ReadError::DebugEntryReadError)? as usize;
        let mut ranges = Vec::new();

        for _ in 0..number_ranges {
            let range = DebugRange::from_bytes(source, range_size)?;

            ranges.push(range);
        }

        Ok(DebugEntry {
            line_number,
            number_ranges,
            ranges,
        })
    }
}

#[derive(Debug)]
pub struct DebugSection {
    range_size: usize,
    debug_entries: Vec<DebugEntry>,
}

impl DebugSection {
    pub fn new(range_size: usize) -> Self {
        DebugSection {
            range_size,
            debug_entries: Vec::new(),
        }
    }

    pub fn add(&mut self, entry: DebugEntry) {
        self.debug_entries.push(entry);
    }

    pub fn debug_entries(&self) -> Iter<DebugEntry> {
        self.debug_entries.iter()
    }

    pub fn range_size(&self) -> usize {
        self.range_size
    }

    pub fn set_range_size(&mut self, range_size: usize) {
        self.range_size = range_size
    }
}

impl ToBytes for DebugSection {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        b'%'.to_bytes(buf);
        b'D'.to_bytes(buf);

        (self.range_size as u8).to_bytes(buf);

        for entry in self.debug_entries.iter() {
            entry.to_bytes(buf, self.range_size);
        }
    }
}

impl FromBytes for DebugSection {
    fn from_bytes(source: &mut Peekable<Iter<u8>>) -> ReadResult<Self>
    where
        Self: Sized,
    {
        if let Some(next) = source.next() {
            if *next != b'D' {
                Err(ReadError::ExpectedDebugSectionError(*next))
            } else {
                let range_size =
                    u8::from_bytes(source).map_err(|_| ReadError::DebugSectionReadError)? as usize;
                let mut debug_entries = Vec::new();

                while source.peek().is_some() {
                    let debug_entry = DebugEntry::from_bytes(source, range_size)?;

                    debug_entries.push(debug_entry);
                }

                Ok(DebugSection {
                    range_size,
                    debug_entries,
                })
            }
        } else {
            Err(ReadError::MissingDebugSectionError)
        }
    }
}
