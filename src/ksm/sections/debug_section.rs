//! A module describing a debug section in a KSM file
use crate::{BufferIterator, DebugEntryParseError, DebugSectionParseError, FromBytes, ToBytes};

use crate::ksm::{fewest_bytes_to_hold, read_var_int, write_var_int, IntSize};
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

    /// Returns the size of a debug range in bytes, using the provided range size
    pub fn size_bytes(&self, range_size: IntSize) -> usize {
        2 * range_size as usize
    }

    /// Converts this range to bytes and writes it to the provided buffer.
    ///
    /// This requires that the range size in bytes is specified, which describes how
    /// many bytes are required to describe a range start or end.
    ///
    /// The range size is based off of the length of the file, and is the debug
    /// section counterpart to NumArgIndexBytes
    pub fn write(&self, buf: &mut Vec<u8>, range_size: IntSize) {
        write_var_int(self.start as u32, buf, range_size);
        write_var_int(self.end as u32, buf, range_size);
    }

    /// Parses a debug range, using the provided range size
    ///
    /// The only reason that this can fail is if we run out of bytes
    pub fn parse(source: &mut BufferIterator, range_size: IntSize) -> Result<Self, ()> {
        let start = read_var_int(source, range_size)? as usize;
        let end = read_var_int(source, range_size)? as usize;

        Ok(Self { start, end })
    }
}

/// An entry into the debug section
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugEntry {
    /// The line number this entry refers to in the source code
    pub line_number: isize,
    ranges: Vec<DebugRange>,
}

impl DebugEntry {
    /// Creates a new debug entry with the provided line number
    pub fn new(line_number: isize) -> Self {
        Self {
            line_number,
            ranges: Vec::new(),
        }
    }

    /// A builder-style method for creating a new DebugEntry while adding a debug range
    pub fn with_range(mut self, range: DebugRange) -> Self {
        self.ranges.push(range);

        self
    }

    /// A builder-style method for creating a new DebugEntry which takes an iterator of DebugRanges
    /// that should be added to this
    pub fn with_ranges(mut self, iter: impl IntoIterator<Item = DebugRange>) -> Self {
        self.ranges.extend(iter);

        self
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

    /// Returns the size of this debug entry in bytes using the provided range size
    pub fn size_bytes(&self, range_size: IntSize) -> usize {
        // Two for the line number, one for the number of ranges
        3 + self
            .ranges
            .iter()
            .map(|r| r.size_bytes(range_size))
            .sum::<usize>()
    }

    /// Converts this debug entry into bytes and writes it into the provided buffer.
    ///
    /// Uses the range size specified in the debug section header
    pub fn write(&self, buf: &mut Vec<u8>, range_size: IntSize) {
        (self.line_number as i16).to_bytes(buf);
        (self.number_ranges() as u8).to_bytes(buf);

        for range in self.ranges.iter() {
            range.write(buf, range_size);
        }
    }

    /// Parses this debug entry from bytes
    ///
    /// Uses the range size specified in the debug section header
    ///
    /// The only reason that this can fail is if we run out of bytes
    pub fn parse(
        source: &mut BufferIterator,
        range_size: IntSize,
    ) -> Result<Self, DebugEntryParseError> {
        let line_number =
            i16::from_bytes(source).map_err(|_| DebugEntryParseError::MissingLineNumber)? as isize;
        let number_ranges =
            u8::from_bytes(source).map_err(|_| DebugEntryParseError::MissingNumRanges)? as usize;
        let mut ranges = Vec::new();

        for i in 0..number_ranges {
            let range = DebugRange::parse(source, range_size)
                .map_err(|_| DebugEntryParseError::MissingRange(i))?;

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
    range_size: IntSize,
    debug_entries: Vec<DebugEntry>,
}

impl DebugSection {
    // 2 for the %D that goes before the section, 1 for the range size
    const BEGIN_SIZE: usize = 3;

    /// Creates a new empty debug section, defaulting to a debug range size of 1 byte
    pub fn new() -> Self {
        Self {
            range_size: IntSize::One,
            debug_entries: Vec::new(),
        }
    }

    /// A builder-style method that takes an iterator of DebugRanges that should be
    /// added to this DebugSection
    pub fn with_entries(mut self, iter: impl IntoIterator<Item = DebugEntry>) -> Self {
        for item in iter {
            self.add(item);
        }

        self
    }

    /// Adds a new debug entry to this debug section
    pub fn add(&mut self, entry: DebugEntry) {
        // Check to see if we need to alter our range size bytes
        for range in entry.ranges() {
            let size = fewest_bytes_to_hold(range.start.max(range.end) as u32);
            if size > self.range_size {
                self.range_size = size;
            }
        }

        self.debug_entries.push(entry);
    }

    /// Returns an iterator over all of the debug entries contained within this section
    pub fn debug_entries(&self) -> Iter<DebugEntry> {
        self.debug_entries.iter()
    }

    /// Specified in the debug section's header, the size, in bytes, of a debug range
    /// in all debug entries. This corresponds to the argument section's NumArgIndexBytes.
    /// This is needed to know how many bytes are required to represent an index into this KSM file's
    /// code sections.
    pub fn range_size(&self) -> IntSize {
        self.range_size
    }

    /// Returns the current size of this debug section in bytes
    pub fn size_bytes(&self) -> usize {
        Self::BEGIN_SIZE
            + self
                .debug_entries
                .iter()
                .map(|e| e.size_bytes(self.range_size))
                .sum::<usize>()
    }

    /// Converts this debug section into bytes and writes it to the provided buffer.
    pub fn write(&self, buf: &mut Vec<u8>) {
        b'%'.to_bytes(buf);
        b'D'.to_bytes(buf);

        (self.range_size as u8).to_bytes(buf);

        for entry in self.debug_entries.iter() {
            entry.write(buf, self.range_size);
        }
    }

    /// Parses a debug section from bytes
    pub fn parse(source: &mut BufferIterator) -> Result<Self, DebugSectionParseError> {
        // This really shouldn't be possible to fail if we wrote this correctly, we wouldn't even be in here
        assert_eq!(source.next().unwrap(), b'D');

        let raw_range_size =
            u8::from_bytes(source).map_err(|_| DebugSectionParseError::MissingDebugRangeSize)?;
        let range_size = IntSize::try_from(raw_range_size)
            .map_err(|_| DebugSectionParseError::InvalidDebugRangeSize(raw_range_size))?;
        let mut debug_entries = Vec::new();

        while source.peek().is_some() {
            let debug_entry = DebugEntry::parse(source, range_size).map_err(|e| {
                DebugSectionParseError::DebugEntryParseError(source.current_index(), e)
            })?;

            debug_entries.push(debug_entry);
        }

        Ok(Self {
            range_size,
            debug_entries,
        })
    }
}

impl Default for DebugSection {
    fn default() -> Self {
        Self::new()
    }
}

impl FromIterator<DebugEntry> for DebugSection {
    fn from_iter<T: IntoIterator<Item = DebugEntry>>(iter: T) -> Self {
        Self::new().with_entries(iter)
    }
}

#[cfg(test)]
mod tests {
    use crate::ksm::sections::{DebugEntry, DebugRange, DebugSection};

    #[test]
    fn size() {
        let mut debug_section = DebugSection::new();

        let mut entry_1 = DebugEntry::new(1);
        entry_1.add(DebugRange::new(0x04, 0x16));

        let mut entry_2 = DebugEntry::new(2);
        entry_2.add(DebugRange::new(0x17, 0x19));
        entry_2.add(DebugRange::new(0x1a, 0x011f));

        debug_section.add(entry_1);
        debug_section.add(entry_2);

        let mut buffer = Vec::with_capacity(128);
        debug_section.write(&mut buffer);

        assert_eq!(debug_section.size_bytes(), buffer.len());
    }
}
