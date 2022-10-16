//! A module describing a debug section in a KSM file
use crate::{FileIterator, FromBytes, ReadError, ReadResult, ToBytes};

use std::slice::Iter;

/// Represents a range of bytes in the KSM file, from the beginning of the code sections, that store
/// the opcodes for the DebugEntry which will contain this range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DebugRange {
    /// The start of this range in bytes
    pub start: usize,
    /// The end of this range in bytes
    pub end: usize,
}

impl DebugRange {
    /// Creates a new debug range with the specified start and end
    pub fn new(start: usize, end: usize) -> Self {
        DebugRange { start, end }
    }

    /// Converts this range to bytes and adds it to the provided buffer.
    ///
    /// This requires that the range size in bytes is specified, which describes how
    /// many bytes are required to describe a range start or end.
    ///
    /// The range size is based off of the length of the file, and is the debug
    /// section counterpart to NumArgIndexBytes
    pub fn to_bytes(&self, buf: &mut Vec<u8>, range_size: usize) {
        DebugRange::write_variable(buf, self.start, range_size);
        DebugRange::write_variable(buf, self.end, range_size);
    }

    // Writes a single range piece/index using the provided size.
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

    /// Parses a debug range, using the provided range size
    pub fn from_bytes(source: &mut FileIterator, range_size: usize) -> ReadResult<Self> {
        let start = DebugRange::read_variable(source, range_size)?;
        let end = DebugRange::read_variable(source, range_size)?;

        Ok(DebugRange { start, end })
    }

    // Reads a single range piece/index using the provided size.
    fn read_variable(source: &mut FileIterator, range_size: usize) -> ReadResult<usize> {
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

/// An entry into the debug section
#[derive(Debug, Clone)]
pub struct DebugEntry {
    /// The line number this entry refers to in the source code
    pub line_number: isize,
    ranges: Vec<DebugRange>,
}

impl DebugEntry {
    /// Creates a new debug entry with the provided line number
    pub fn new(line_number: isize) -> Self {
        DebugEntry {
            line_number,
            ranges: Vec::new(),
        }
    }

    /// Adds a debug range to this entry
    pub fn add(&mut self, range: DebugRange) {
        self.ranges.push(range);
    }

    /// The number of DebugRanges that this entry has
    pub fn number_ranges(&self) -> usize {
        self.ranges.len()
    }

    /// Returns an iterator over each debug range in this entry
    pub fn ranges(&self) -> Iter<DebugRange> {
        self.ranges.iter()
    }

    /// Returns the debug range from the index into this entry's list of debug ranges.
    ///
    /// Returns None if there is no DebugRange at the provided index.
    pub fn get_range(&self, index: usize) -> Option<&DebugRange> {
        self.ranges.get(index)
    }

    /// Converts this debug entry into bytes and writes it into the provided buffer.
    ///
    /// Uses the range size specified in the debug section header
    pub fn to_bytes(&self, buf: &mut Vec<u8>, range_size: usize) {
        (self.line_number as i16).to_bytes(buf);
        (self.number_ranges() as u8).to_bytes(buf);

        for range in self.ranges.iter() {
            range.to_bytes(buf, range_size);
        }
    }

    /// Parses this debug entry from bytes
    ///
    /// Uses the range size specified in the debug section header
    pub fn from_bytes(source: &mut FileIterator, range_size: usize) -> ReadResult<Self> {
        let line_number =
            i16::from_bytes(source).map_err(|_| ReadError::DebugEntryReadError)? as isize;
        let number_ranges =
            u8::from_bytes(source).map_err(|_| ReadError::DebugEntryReadError)? as usize;
        let mut ranges = Vec::new();

        for _ in 0..number_ranges {
            let range = DebugRange::from_bytes(source, range_size)?;

            ranges.push(range);
        }

        Ok(Self {
            line_number,
            ranges,
        })
    }
}

/// A debug section in a KSM file
///
/// The section of a KSM file that is dedicated to presenting (somewhat) better error messages
/// to the user of the program. This section maps source code lines to ranges of bytes in the file.
///
#[derive(Debug)]
pub struct DebugSection {
    /// Specified in the debug section's header, the size, in bytes, of a debug range
    /// in all debug entries. This corresponds to the argument section's NumArgIndexBytes.
    /// This is needed to know how many bytes are required to represent an index into this KSM file's
    /// code sections.
    pub range_size: usize,
    debug_entries: Vec<DebugEntry>,
}

impl DebugSection {
    /// Creates a new debug section using the default range size of 1.
    ///
    /// Unfortunately, this debug section has no way to know what the range size
    /// should be, as that depends on all of the previous code sections. So in order
    /// for this debug section to work, we must update the range size after the rest of the file
    /// has been generated.
    ///
    pub fn new() -> Self {
        DebugSection {
            range_size: 1,
            debug_entries: Vec::new(),
        }
    }

    /// Adds a new debug entry to this debug section
    pub fn add(&mut self, entry: DebugEntry) {
        self.debug_entries.push(entry);
    }

    /// Returns an iterator over all of the debug entries contained within this section
    pub fn debug_entries(&self) -> Iter<DebugEntry> {
        self.debug_entries.iter()
    }
}

impl Default for DebugSection {
    fn default() -> Self {
        Self::new()
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
    fn from_bytes(source: &mut FileIterator) -> ReadResult<Self>
    where
        Self: Sized,
    {
        if let Some(next) = source.next() {
            if next != b'D' {
                Err(ReadError::ExpectedDebugSectionError(next))
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
