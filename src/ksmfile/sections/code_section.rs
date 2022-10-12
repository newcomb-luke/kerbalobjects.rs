use crate::ksmfile::Instr;
use crate::{FromBytes, ReadError, ReadResult, ToBytes};
use std::iter::Peekable;
use std::slice::Iter;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CodeType {
    Function,
    Initialization,
    Main,
    Unknown,
}

impl From<u8> for CodeType {
    fn from(byte: u8) -> Self {
        match byte {
            b'F' => CodeType::Function,
            b'I' => CodeType::Initialization,
            b'M' => CodeType::Main,
            _ => CodeType::Unknown,
        }
    }
}

impl From<CodeType> for u8 {
    fn from(c: CodeType) -> Self {
        match c {
            CodeType::Function => b'F',
            CodeType::Initialization => b'I',
            CodeType::Main => b'M',
            CodeType::Unknown => b'U',
        }
    }
}

impl ToBytes for CodeType {
    fn to_bytes(&self, buf: &mut Vec<u8>) {
        buf.push((*self).into());
    }
}

impl FromBytes for CodeType {
    fn from_bytes(source: &mut Peekable<Iter<u8>>) -> ReadResult<Self>
    where
        Self: Sized,
    {
        let value = *source.next().ok_or(ReadError::CodeTypeReadError)?;
        let code_type = CodeType::from(value);

        match code_type {
            CodeType::Unknown => Err(ReadError::UnknownCodeTypeReadError(value as char)),
            _ => Ok(code_type),
        }
    }
}

#[derive(Debug)]
pub struct CodeSection {
    section_type: CodeType,
    instructions: Vec<Instr>,
}

impl CodeSection {
    pub fn new(section_type: CodeType) -> Self {
        CodeSection {
            section_type,
            instructions: Vec::new(),
        }
    }

    pub fn instructions(&self) -> Iter<Instr> {
        self.instructions.iter()
    }

    pub fn add(&mut self, instr: Instr) {
        self.instructions.push(instr);
    }

    pub fn section_type(&self) -> CodeType {
        self.section_type
    }

    pub fn to_bytes(&self, buf: &mut Vec<u8>, num_index_bytes: usize) {
        buf.push(b'%');
        self.section_type.to_bytes(buf);

        for instr in self.instructions.iter() {
            instr.to_bytes(buf, num_index_bytes);
        }
    }

    pub fn from_bytes(source: &mut Peekable<Iter<u8>>, num_index_bytes: usize) -> ReadResult<Self> {
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
                if **next == b'%' {
                    break;
                }

                let instr = Instr::from_bytes(source, num_index_bytes)?;

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
