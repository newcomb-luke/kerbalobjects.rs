//! A module describing a code section in a KSM file
use crate::ksm::{Instr, IntSize};
use crate::{FileIterator, FromBytes, ReadError, ReadResult, ToBytes};
use std::slice::Iter;

/// The type of code that a code section is
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CodeType {
    /// A user-defined function
    Function,
    /// Code that is run before the code in the Main section, usually setting up of
    /// delegates or triggers.
    Initialization,
    /// The code that gets run when the file gets loaded
    Main,
}

impl TryFrom<u8> for CodeType {
    type Error = ();

    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        match byte {
            b'F' => Ok(CodeType::Function),
            b'I' => Ok(CodeType::Initialization),
            b'M' => Ok(CodeType::Main),
            _ => Err(()),
        }
    }
}

impl From<CodeType> for u8 {
    fn from(c: CodeType) -> Self {
        match c {
            CodeType::Function => b'F',
            CodeType::Initialization => b'I',
            CodeType::Main => b'M',
        }
    }
}

impl ToBytes for CodeType {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        buf.push((*self).into());
    }
}

impl FromBytes for CodeType {
    fn from_bytes(source: &mut FileIterator) -> ReadResult<Self>
    where
        Self: Sized,
    {
        let value = source.next().ok_or(ReadError::CodeTypeReadError)?;

        CodeType::try_from(value).map_err(|_| ReadError::UnknownCodeTypeReadError(value as char))
    }
}

/// A code section within a KSM file.
///
/// A KSM file can have as many
/// of these as required, as each one can represent a user-defined function, or other types of code.
///
/// There should be a minimum of three of them in each KSM file in order for it to be properly loaded
/// into kOS.
///
/// All of the function code sections should come first, followed by a required, but possibly
/// empty Initialization section, followed by a required, but possibly empty Main section.
///
#[derive(Debug)]
pub struct CodeSection {
    /// The type of code section that this is
    pub section_type: CodeType,
    instructions: Vec<Instr>,
}

impl CodeSection {
    // 2 for the %F/I/M that goes before the section
    const BEGIN_SIZE: usize = 2;

    /// Creates a new code section of the provided type
    pub fn new(section_type: CodeType) -> Self {
        CodeSection {
            section_type,
            instructions: Vec::new(),
        }
    }

    /// Returns an iterator over all of the instructions in this code section
    pub fn instructions(&self) -> Iter<Instr> {
        self.instructions.iter()
    }

    /// Adds a new instruction to this code section
    pub fn add(&mut self, instr: Instr) {
        self.instructions.push(instr);
    }

    /// Returns how large this section will be if it is written with the provided
    /// number of argument index bytes
    pub fn size_bytes(&self, index_bytes: IntSize) -> usize {
        Self::BEGIN_SIZE
            + self
                .instructions
                .iter()
                .map(|i| i.size_bytes(index_bytes))
                .sum::<usize>()
    }

    /// Converts this code section into bytes and appends it to the provided buffer.
    ///
    /// This requires the number of bytes required to index into the argument section
    /// so that instruction operands can be written using the correct byte width.
    pub fn to_bytes(&self, buf: &mut Vec<u8>, index_bytes: IntSize) {
        buf.push(b'%');
        self.section_type.to_bytes(buf);

        for instr in self.instructions.iter() {
            instr.to_bytes(buf, index_bytes);
        }
    }

    /// Parses a code section from bytes.
    ///
    /// This requires the number of bytes required to index into the argument section
    /// so that instruction operands can be read using the correct byte width.
    pub fn from_bytes(source: &mut FileIterator, index_bytes: IntSize) -> ReadResult<Self> {
        #[cfg(feature = "print_debug")]
        {
            print!("Reading code section, ");
        }

        let section_type = CodeType::from_bytes(source)?;

        #[cfg(feature = "print_debug")]
        {
            println!("{:?}", section_type);
        }

        let mut instructions = Vec::new();

        loop {
            if let Some(next) = source.peek() {
                if next == b'%' {
                    break;
                }

                let instr = Instr::from_bytes(source, index_bytes)?;

                #[cfg(feature = "print_debug")]
                {
                    println!("\tRead instruction {:?}", instr);
                }

                instructions.push(instr);
            } else {
                return Err(ReadError::CodeSectionReadError);
            }
        }

        Ok(CodeSection {
            section_type,
            instructions,
        })
    }
}
