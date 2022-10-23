//! A module describing an argument section in a KSM file

use crate::ksm::{fewest_bytes_to_hold, read_var_int, write_var_int, IntSize};
use crate::{ArgumentSectionParseError, BufferIterator, FromBytes, KOSValue, ToBytes};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::slice::Iter;

/// A wrapper type that represents an index into the argument section of a KSM file.
///
/// This type implements From<usize> and usize implements From<ArgIndex>, but this is provided
/// so that it takes 1 extra step to convert raw integers into ArgIndexes which could stop potential
/// logical bugs.
///
/// This is a kOS-governed type that is an index into the *bytes* of an argument section.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ArgIndex(usize);

impl ArgIndex {
    /// Tries to parse an ArgIndex from the byte source provided, and the NumArgIndexBytes
    /// from the argument section header.
    ///
    /// Returns either the ArgIndex, or an Err(()), which can only happen if we ran out of bytes.
    ///
    pub fn parse(source: &mut BufferIterator, index_bytes: IntSize) -> Result<Self, ()> {
        read_var_int(source, index_bytes)
            .map_err(|_| ())
            .map(|v| v.into())
    }

    /// Writes an ArgIndex into the provided buffer, using the NumArgIndexBytes, which
    /// is required to know how many bytes to use to write the index.
    pub fn write(&self, buf: &mut Vec<u8>, index_bytes: IntSize) {
        write_var_int(self.0 as u32, buf, index_bytes);
    }
}

impl From<usize> for ArgIndex {
    fn from(i: usize) -> Self {
        Self(i)
    }
}

impl From<u8> for ArgIndex {
    fn from(i: u8) -> Self {
        Self(i as usize)
    }
}

impl From<u16> for ArgIndex {
    fn from(i: u16) -> Self {
        Self(i as usize)
    }
}

impl From<u32> for ArgIndex {
    fn from(i: u32) -> Self {
        Self(i as usize)
    }
}

impl From<u64> for ArgIndex {
    fn from(i: u64) -> Self {
        Self(i as usize)
    }
}

impl From<ArgIndex> for usize {
    fn from(arg_idx: ArgIndex) -> Self {
        arg_idx.0
    }
}

/// An argument section within a KSM file.
///
/// You can create a new ArgumentSection using new() and then add items using add():
///
/// This section stores all operands of every
/// instruction contained within the code sections of a KSM file, which all index into the
/// file's argument section.
///
/// ```
/// use kerbalobjects::KOSValue;
/// use kerbalobjects::ksm::sections::ArgumentSection;
///
/// let mut arg_section = ArgumentSection::new();
///
/// let index = arg_section.add(KOSValue::Int16(2));
/// ```
///
/// See the [file format docs](https://github.com/newcomb-luke/kerbalobjects.rs/blob/main/docs/KSM-file-format.md#argument-section) for more details.
#[derive(Debug)]
pub struct ArgumentSection {
    num_index_bytes: IntSize,
    hashes: HashMap<u64, ArgIndex>,
    arguments: Vec<KOSValue>,
    value_index_map: HashMap<ArgIndex, usize>,
    size_bytes: usize,
}

impl ArgumentSection {
    // 2 for the %A that goes before the section, and 1 for the NumArgIndexBytes
    const BEGIN_SIZE: usize = 3;

    /// Creates a new empty ArgumentSection
    pub fn new() -> Self {
        Self {
            num_index_bytes: IntSize::One,
            hashes: HashMap::new(),
            arguments: Vec::new(),
            value_index_map: HashMap::new(),
            size_bytes: Self::BEGIN_SIZE,
        }
    }

    /// Creates a new empty ArgumentSection, with the provided pre-allocated size
    pub fn with_capacity(amount: usize) -> Self {
        Self {
            num_index_bytes: IntSize::One,
            hashes: HashMap::with_capacity(amount),
            arguments: Vec::with_capacity(amount),
            value_index_map: HashMap::with_capacity(amount),
            size_bytes: Self::BEGIN_SIZE,
        }
    }

    /// A builder-style method that takes an iterator of KOSValues that should be added
    /// to this ArgumentSection
    ///
    /// This variant performs checking as the items are added
    ///
    /// This also returns a Vec<ArgIndex> that represent all of the indices that every element
    /// inserted resides at, and will contain duplicates if duplicate elements were added.
    ///
    pub fn with_arguments_checked(
        mut self,
        iter: impl IntoIterator<Item = KOSValue>,
    ) -> (Self, Vec<ArgIndex>) {
        let iter = iter.into_iter();
        let mut indices = Vec::with_capacity(iter.size_hint().0);

        for item in iter {
            indices.push(self.add_checked(item));
        }

        (self, indices)
    }

    /// A builder-style method that takes an iterator of KOSValues that should be added
    /// to this ArgumentSection
    ///
    /// This variant does *not* perform checking as the items are added
    ///
    /// This also returns a Vec<ArgIndex> that represent all of the indices that every element
    /// inserted resides at, and will *not* contain duplicates even if duplicate elements were added.
    ///
    pub fn with_arguments_unchecked(
        mut self,
        iter: impl IntoIterator<Item = KOSValue>,
    ) -> (Self, Vec<ArgIndex>) {
        let iter = iter.into_iter();
        let mut indices = Vec::with_capacity(iter.size_hint().0);

        for item in iter {
            indices.push(self.add(item));
        }

        (self, indices)
    }

    /// Returns the NumArgIndexBytes that this argument section currently requires.
    ///
    /// This represents the current size range of this argument section, because this is the number
    /// of bytes that are required to reference an item within the argument section.
    pub fn num_index_bytes(&self) -> IntSize {
        self.num_index_bytes
    }

    /// Returns the ArgIndex into this argument section that a KOSValue resides at, or None
    /// if no such value is in this section.
    pub fn find(&self, value: &KOSValue) -> Option<ArgIndex> {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        let hash = hasher.finish();

        self.hashes.get(&hash).copied()
    }

    /// Add a new KOSValue to this argument section, checking if it is a duplicate, and
    /// returning the ArgIndex of the value. If it already exists, the ArgIndex of that value is
    /// returned, if it does not, then it is added, and the new ArgIndex is returned.
    pub fn add_checked(&mut self, value: KOSValue) -> ArgIndex {
        match self.find(&value) {
            Some(index) => index,
            None => self.add(value),
        }
    }

    /// Adds a new KOSValue to this argument section.
    ///
    /// This does not do any sort of checking for duplication and will simply add it.
    ///
    /// This function returns the ArgIndex that can be used to refer to the inserted value,
    /// for example when trying to reference it in an instruction.
    pub fn add(&mut self, argument: KOSValue) -> ArgIndex {
        let size = argument.size_bytes();
        let index = self.arguments.len();
        let arg_index = ArgIndex(self.size_bytes);

        let mut hasher = DefaultHasher::new();
        argument.hash(&mut hasher);
        let hash = hasher.finish();

        self.hashes.entry(hash).or_insert(arg_index);

        self.arguments.push(argument);
        self.value_index_map.insert(arg_index, index);
        self.size_bytes += size;

        // This may be a little slow, but it saves us from having any state that a user has to deal with
        self.recalculate_index_bytes();

        arg_index
    }

    /// Gets a reference to a particular KOSValue in this argument section.
    ///
    /// This is done using the ArgIndex that is returned when the value was added.
    ///
    /// Returns None of the ArgIndex doesn't refer to a valid value.
    pub fn get(&self, index: ArgIndex) -> Option<&KOSValue> {
        let vec_index = *self.value_index_map.get(&index)?;

        self.arguments.get(vec_index)
    }

    /// Returns an iterator over all of the KOSValues that are stored in this section.
    pub fn arguments(&self) -> Iter<KOSValue> {
        self.arguments.iter()
    }

    /// Returns the size in bytes that this section would take up in total in the final binary file.
    pub fn size_bytes(&self) -> usize {
        self.size_bytes
    }

    // Recalculates the number of bytes required to reference any value within this section.
    fn recalculate_index_bytes(&mut self) {
        self.num_index_bytes = fewest_bytes_to_hold(self.size_bytes as u32);
    }

    /// Attempts to parse an argument section from the current buffer iterator.
    ///
    /// This can fail if the buffer runs out of bytes, or if the argument section is malformed.
    ///
    pub fn parse(source: &mut BufferIterator) -> Result<Self, ArgumentSectionParseError> {
        #[cfg(feature = "print_debug")]
        {
            println!("Reading ArgumentSection");
        }

        let header =
            u16::from_bytes(source).map_err(|_| ArgumentSectionParseError::MissingHeader)?;

        // %A in hex, little-endian
        if header != 0x4125 {
            return Err(ArgumentSectionParseError::InvalidHeader(header));
        }

        let raw_num_index_bytes = u8::from_bytes(source)
            .map_err(|_| ArgumentSectionParseError::MissingNumArgIndexBytes)?;

        let num_index_bytes: IntSize = raw_num_index_bytes
            .try_into()
            .map_err(ArgumentSectionParseError::InvalidNumArgIndexBytes)?;

        #[cfg(feature = "print_debug")]
        {
            println!("\tNumber of index bytes: {}", raw_num_index_bytes);
        }

        let mut arg_section = Self {
            num_index_bytes,
            hashes: HashMap::new(),
            arguments: Vec::new(),
            value_index_map: HashMap::new(),
            size_bytes: Self::BEGIN_SIZE,
        };

        loop {
            if let Some(next) = source.peek() {
                if next == b'%' {
                    break;
                } else {
                    let argument = KOSValue::from_bytes(source).map_err(|e| {
                        ArgumentSectionParseError::KOSValueParseError(source.current_index(), e)
                    })?;

                    #[cfg(feature = "print_debug")]
                    {
                        println!("\tRead argument: {:?}", argument);
                    }

                    arg_section.add(argument);
                }
            } else {
                return Err(ArgumentSectionParseError::EOF);
            }
        }

        Ok(arg_section)
    }

    /// Appends the byte representation of this argument section to a buffer of bytes
    pub fn write(&self, buf: &mut Vec<u8>) {
        // Write the section header
        b'%'.to_bytes(buf);
        b'A'.to_bytes(buf);
        // Store the NumArgIndexBytes
        (self.num_index_bytes as u8).to_bytes(buf);

        // Simply write out each KOSValue
        for argument in self.arguments.iter() {
            argument.to_bytes(buf);
        }
    }
}

impl Default for ArgumentSection {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::ksm::sections::{ArgIndex, ArgumentSection};
    use crate::ksm::IntSize;
    use crate::{BufferIterator, KOSValue};

    #[test]
    fn arg_section_checked() {
        let mut arg_section = ArgumentSection::new();

        let index = arg_section.add_checked(KOSValue::ArgMarker);
        println!("{:?}", index);
        arg_section.add_checked(KOSValue::ArgMarker);
        arg_section.add_checked(KOSValue::ArgMarker);

        let mut args = arg_section.arguments();

        assert_eq!(KOSValue::ArgMarker, *args.next().unwrap());
        assert!(args.next().is_none());

        arg_section.add(KOSValue::ArgMarker);

        let mut args = arg_section.arguments();

        assert_eq!(KOSValue::ArgMarker, *args.next().unwrap());
        assert_eq!(KOSValue::ArgMarker, *args.next().unwrap());
        assert!(args.next().is_none());

        let other_index = arg_section.add_checked(KOSValue::ArgMarker);

        assert_eq!(index, other_index);
    }

    #[test]
    fn size() {
        let mut arg_section = ArgumentSection::new();

        arg_section.add(KOSValue::ArgMarker);
        arg_section.add(KOSValue::Bool(false));
        arg_section.add(KOSValue::String("kOS".into()));

        let mut buffer = Vec::with_capacity(128);

        arg_section.write(&mut buffer);

        assert_eq!(buffer.len(), arg_section.size_bytes());
    }

    #[test]
    fn arg_index_read() {
        let data = vec![0x5, 0x4, 0x3];
        let mut source = BufferIterator::new(&data);

        let arg_index = ArgIndex::parse(&mut source, IntSize::Three).unwrap();
        assert_eq!(arg_index, ArgIndex::from(0x00050403u32));

        let data = vec![0x1, 0x5, 0x4, 0x3];
        let mut source = BufferIterator::new(&data);

        let arg_index = ArgIndex::parse(&mut source, IntSize::Four).unwrap();
        assert_eq!(arg_index, ArgIndex::from(0x01050403u32));

        let data = vec![0x4, 0x3];
        let mut source = BufferIterator::new(&data);

        let arg_index = ArgIndex::parse(&mut source, IntSize::Two).unwrap();
        assert_eq!(arg_index, ArgIndex::from(0x0403u16));

        let data = vec![0x3];
        let mut source = BufferIterator::new(&data);

        let arg_index = ArgIndex::parse(&mut source, IntSize::One).unwrap();
        assert_eq!(arg_index, ArgIndex::from(0x03u8));
    }

    #[test]
    fn arg_index_write() {
        let arg_index = ArgIndex::from(0xffu8);
        let mut data = Vec::new();

        arg_index.write(&mut data, IntSize::One);
        assert_eq!(data, Vec::from([0xff]));

        let arg_index = ArgIndex::from(0x03ffu16);
        let mut data = Vec::new();

        arg_index.write(&mut data, IntSize::Two);
        assert_eq!(data, Vec::from([0x03, 0xff]));

        let arg_index = ArgIndex::from(0x0503ffu32);
        let mut data = Vec::new();

        arg_index.write(&mut data, IntSize::Three);
        assert_eq!(data, Vec::from([0x05, 0x03, 0xff]));

        let arg_index = ArgIndex::from(0x05ffefffu32);
        let mut data = Vec::new();

        arg_index.write(&mut data, IntSize::Four);
        assert_eq!(data, Vec::from([0x05, 0xff, 0xef, 0xff]));
    }
}
