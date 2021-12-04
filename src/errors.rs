pub type ReadResult<T> = Result<T, ReadError>;

#[derive(Debug)]
pub enum ReadError {
    ConstantReadError(ConstantReadError),
    OpcodeReadError,
    BogusOpcodeReadError(u8),
    OperandReadError,
    SymBindReadError,
    UnknownSimBindReadError(u8),
    SymTypeReadError,
    UnknownSimTypeReadError(u8),
    KOSymbolConstantReadError(&'static str),
    KOSValueTypeReadError(u8),
    KOSValueReadError,
    SectionKindReadError,
    UnknownSectionKindReadError(u8),
    SectionHeaderConstantReadError(&'static str),
    StringTableReadError,
    KOHeaderReadError(&'static str),
    KSMHeaderReadError(&'static str),
    VersionMismatchError(u8),
    DebugSectionUnsupportedError,
    MissingSectionError(&'static str),
    InvalidKOFileMagicError,
    InvalidKSMFileMagicError,
    KSMDecompressionError(std::io::Error),
    MissingArgumentSectionError,
    ExpectedArgumentSectionError(u16),
    NumIndexBytesReadError,
    InvalidNumIndexBytesError(usize),
    ArgumentSectionReadError,
    CodeTypeReadError,
    UnknownCodeTypeReadError(char),
    MissingCodeSectionError,
    ExpectedCodeSectionError(u8),
    ReldReadError,
    CodeSectionReadError,
    DebugRangeReadError,
    DebugEntryReadError,
    MissingDebugSectionError,
    ExpectedDebugSectionError(u8),
    DebugSectionReadError,
}

#[derive(Debug)]
pub enum ConstantReadError {
    BoolReadError,
    U8ReadError,
    I8ReadError,
    U16ReadError,
    I16ReadError,
    U32ReadError,
    I32ReadError,
    F32ReadError,
    F64ReadError,
    StringReadError,
}

impl std::error::Error for ReadError {}
impl std::error::Error for ConstantReadError {}

impl std::fmt::Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadError::ConstantReadError(e) => {
                write!(f, "Error reading constant of type {}, ran out of bytes.", e)
            }
            ReadError::OpcodeReadError => {
                write!(f, "Error reading opcode for instruction, ran out of bytes.")
            }
            ReadError::BogusOpcodeReadError(v) => {
                write!(
                    f,
                    "Error reading opcode for instruction, value {:x} is not a valid opcode",
                    v
                )
            }
            ReadError::OperandReadError => {
                write!(
                    f,
                    "Error reading operand for instruction, ran out of bytes."
                )
            }
            ReadError::SymBindReadError => {
                write!(f, "Error reading symbol binding, ran out of bytes.")
            }
            ReadError::UnknownSimBindReadError(v) => {
                write!(
                    f,
                    "Error reading symbol binding, unknown binding with value {:x}",
                    v
                )
            }
            ReadError::SymTypeReadError => {
                write!(f, "Error reading symbol type, ran out of bytes.")
            }
            ReadError::UnknownSimTypeReadError(v) => {
                write!(
                    f,
                    "Error reading symbol type, unknown type with value {:x}",
                    v
                )
            }
            ReadError::KOSymbolConstantReadError(s) => {
                write!(
                    f,
                    "Error reading symbol while parsing constant for {}, ran out of bytes.",
                    s
                )
            }
            ReadError::KOSValueTypeReadError(v) => {
                write!(
                    f,
                    "Error reading kOS value, unknown type with value {:x}",
                    v
                )
            }
            ReadError::KOSValueReadError => {
                write!(f, "Error reading kOS value, ran out of bytes.")
            }
            ReadError::SectionKindReadError => {
                write!(f, "Error reading section kind, ran out of bytes.")
            }
            ReadError::UnknownSectionKindReadError(v) => {
                write!(
                    f,
                    "Error reading section kind, unknown kind with value {:x}",
                    v
                )
            }
            ReadError::SectionHeaderConstantReadError(s) => {
                write!(
                    f,
                    "Error reading section header while parsing constant {}, ran out of bytes.",
                    s
                )
            }
            ReadError::StringTableReadError => {
                write!(f, "Error reading string table, ran out of bytes.")
            }
            ReadError::KOHeaderReadError(s) => {
                write!(
                    f,
                    "Error reading kerbal object file header, while parsing constant {}, ran out of bytes.",
                    s
                )
            }
            ReadError::KSMHeaderReadError(s) => {
                write!(f, "Error reading kerbal machine code file, while parsing constant {}, ran out of bytes.", s)
            }
            ReadError::VersionMismatchError(v) => {
                write!(
                    f,
                    "Error reading KerbalObject file, unsupported KO file version: {}",
                    v
                )
            }
            ReadError::DebugSectionUnsupportedError => {
                write!(
                    f,
                    "Error reading KerbalObject file, debug sections unsupported in this version."
                )
            }
            &ReadError::MissingSectionError(s) => {
                write!(
                    f,
                    "Error reading KerbalObject file, expected section: {}",
                    s
                )
            }
            ReadError::InvalidKOFileMagicError => {
                write!(
                    f,
                    "Error reading KerbalObject file, input file does not appear to be a KO file"
                )
            }
            ReadError::InvalidKSMFileMagicError => {
                write!(f, "Error reading Kerbal Machine Code file, input file does not appear to be a KSM file")
            }
            ReadError::KSMDecompressionError(e) => {
                write!(
                    f,
                    "Error while trying to decompress Kerbal Machine File: {}",
                    e
                )
            }
            ReadError::MissingArgumentSectionError => {
                write!(f, "Error reading Kerbal Machine Code file, expected argument section, ran out of bytes.")
            }
            ReadError::ExpectedArgumentSectionError(v) => {
                write!(f, "Error reading Kerbal Machine Code file, expected argument section, instead found {:x}", v)
            }
            ReadError::NumIndexBytesReadError => {
                write!(f, "Error reading Kerbal Machine Code argument section num index bytes, ran out of bytes.")
            }
            ReadError::InvalidNumIndexBytesError(n) => {
                write!(f, "Error reading Kerbal Machine Code argument section num index bytes, invalid number: {}, only values 1-4 permitted.", n)
            }
            ReadError::ArgumentSectionReadError => {
                write!(f, "Error reading Kerbal Machine Code argument section, expected argument, ran out of bytes.")
            }
            ReadError::CodeTypeReadError => {
                write!(f, "Error reading KSM section type, ran out of bytes")
            }
            ReadError::UnknownCodeTypeReadError(c) => {
                write!(
                    f,
                    "Error reading KSM section type, unknown type with value: {}",
                    c
                )
            }
            ReadError::MissingCodeSectionError => {
                write!(f, "Error reading code section, ran out of bytes")
            }
            ReadError::ExpectedCodeSectionError(b) => {
                write!(
                    f,
                    "Error reading KSM file, expected code section, found {:?}",
                    b
                )
            }
            ReadError::ReldReadError => {
                write!(f, "Error reading relocation data, ran out of bytes")
            }
            ReadError::CodeSectionReadError => {
                write!(f, "Error reading code section, ran out of bytes")
            }
            ReadError::DebugRangeReadError => {
                write!(f, "Error reading debug section, ran out of bytes")
            }
            ReadError::DebugEntryReadError => {
                write!(f, "Error reading debug entry, ran out of bytes")
            }
            ReadError::MissingDebugSectionError => {
                write!(f, "Error reading KSM file debug section, expected debug section, ran out of bytes")
            }
            ReadError::ExpectedDebugSectionError(b) => {
                write!(
                    f,
                    "Error reading KSM file, expected debug section header, found {:?}",
                    b
                )
            }
            ReadError::DebugSectionReadError => {
                write!(f, "Error reading KSM file debug section, ran out of bytes")
            }
        }
    }
}

impl std::fmt::Display for ConstantReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            &ConstantReadError::BoolReadError => {
                write!(f, "boolean")
            }
            &ConstantReadError::U8ReadError => {
                write!(f, "unsigned byte")
            }
            &ConstantReadError::I8ReadError => {
                write!(f, "signed byte")
            }
            &ConstantReadError::U16ReadError => {
                write!(f, "unsigned 16-bit integer")
            }
            &ConstantReadError::I16ReadError => {
                write!(f, "signed 16-bit integer")
            }
            &ConstantReadError::U32ReadError => {
                write!(f, "unsigned 32-bit integer")
            }
            &ConstantReadError::I32ReadError => {
                write!(f, "signed 32-bit integer")
            }
            &ConstantReadError::F32ReadError => {
                write!(f, "float")
            }
            &ConstantReadError::F64ReadError => {
                write!(f, "double float")
            }
            &ConstantReadError::StringReadError => {
                write!(f, "string")
            }
        }
    }
}
