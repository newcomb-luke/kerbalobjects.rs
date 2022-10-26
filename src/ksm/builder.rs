//! A module containing code related to the KSMFileBuilder type, which allows for the creation of a
//! KSMFile with a builder-like interface.
//!
//! # Example:
//!
//! ```
//! use std::io::Write;
//! use kerbalobjects::ksm::sections::{ArgumentSection, CodeSection, CodeType, DebugEntry, DebugRange, DebugSection};
//! use kerbalobjects::ksm::{Instr, KSMFileBuilder};
//! use kerbalobjects::{Opcode, KOSValue, ToBytes};
//!
//! let builder = KSMFileBuilder::new();
//!
//! let mut arg_section = ArgumentSection::new();
//! let mut main_code = CodeSection::new(CodeType::Main);
//!
//! let one = arg_section.add_checked(KOSValue::Int16(1));
//!
//! // Corresponds to the KerbalScript code:
//! // PRINT("Hello, world!").
//!
//! main_code.add(Instr::OneOp(Opcode::Push, arg_section.add_checked(KOSValue::String("@0001".into()))));
//! main_code.add(Instr::TwoOp(Opcode::Bscp, one, arg_section.add_checked(KOSValue::Int16(0))));
//! main_code.add(Instr::ZeroOp(Opcode::Argb));
//! main_code.add(Instr::OneOp(Opcode::Push, arg_section.add_checked(KOSValue::ArgMarker)));
//! main_code.add(Instr::OneOp(Opcode::Push, arg_section.add_checked(KOSValue::StringValue("Hello, world!".into()))));
//! main_code.add(Instr::TwoOp(Opcode::Call, arg_section.add_checked(KOSValue::String("".into())), arg_section.add_checked(KOSValue::String("print()".into()))));
//! main_code.add(Instr::ZeroOp(Opcode::Pop));
//! main_code.add(Instr::OneOp(Opcode::Escp, one));
//!
//! let code_sections = vec![
//!     CodeSection::new(CodeType::Function),
//!     CodeSection::new(CodeType::Initialization),
//!     main_code
//! ];
//!
//! let builder = builder.with_arg_section(arg_section).with_code_sections(code_sections);
//!
//! // A completely wrong and useless debug section, but we NEED to have one
//! let mut debug_entry =  DebugEntry::new(1).with_range(DebugRange::new(0x06, 0x13));
//!
//! let debug_section = DebugSection::new(debug_entry);
//!
//! let ksm_file = builder.with_debug_section(debug_section).finish();
//!
//! let mut file_buffer = Vec::with_capacity(2048);
//!
//! ksm_file.write(&mut file_buffer);
//!
//! let mut file = std::fs::File::create("hello.ksm").expect("Couldn't open output file");
//!
//! file.write_all(file_buffer.as_slice()).expect("Failed to write to output file");
//! ```
//!
use crate::ksm::sections::{ArgumentSection, CodeSection, DebugSection};
use crate::ksm::KSMFile;

use std::marker::PhantomData;

#[allow(missing_docs)]
#[allow(missing_debug_implementations)]
pub mod completeness {
    #[non_exhaustive]
    pub struct Empty;

    #[non_exhaustive]
    pub struct WithArgSection;

    #[non_exhaustive]
    pub struct WithCodeSections;

    #[non_exhaustive]
    pub struct WithDebugSection;

    #[non_exhaustive]
    pub struct WithArgAndCode;

    #[non_exhaustive]
    pub struct WithArgAndDebug;

    #[non_exhaustive]
    pub struct WithCodeAndDebug;

    #[non_exhaustive]
    pub struct Complete;
}

macro_rules! with_arg_section {
    ($transition: ty) => {
        /// Adds the required argument section to this KSMFileBuilder
        pub fn with_arg_section(self, arg_section: ArgumentSection) -> KSMFileBuilder<$transition> {
            KSMFileBuilder {
                argument_section: Some(arg_section),
                code_sections: self.code_sections,
                debug_section: self.debug_section,
                completeness: PhantomData,
            }
        }
    };
}

macro_rules! with_code_section {
    ($transition: ty) => {
        /// Adds the provided code section to this KSMFileBuilder. At least 3 code sections
        /// are required to have a well-formed KSM file.
        pub fn with_code_section(
            mut self,
            code_section: CodeSection,
        ) -> KSMFileBuilder<$transition> {
            self.code_sections.push(code_section);

            KSMFileBuilder {
                argument_section: self.argument_section,
                code_sections: self.code_sections,
                debug_section: self.debug_section,
                completeness: PhantomData,
            }
        }
    };
}

macro_rules! with_code_sections {
    ($transition: ty) => {
        /// Adds all of the provided code sections to this KSMFileBuilder. At least 3 code sections
        /// are required to have a well-formed KSM file.
        pub fn with_code_sections(
            mut self,
            code_sections: Vec<CodeSection>,
        ) -> KSMFileBuilder<$transition> {
            self.code_sections.extend(code_sections);

            KSMFileBuilder {
                argument_section: self.argument_section,
                code_sections: self.code_sections,
                debug_section: self.debug_section,
                completeness: PhantomData,
            }
        }
    };
}

macro_rules! with_debug_section {
    ($transition: ty) => {
        /// Adds the required debug section to this KSMFileBuilder
        pub fn with_debug_section(
            self,
            debug_section: DebugSection,
        ) -> KSMFileBuilder<$transition> {
            KSMFileBuilder {
                argument_section: self.argument_section,
                code_sections: self.code_sections,
                debug_section: Some(debug_section),
                completeness: PhantomData,
            }
        }
    };
}

/// A struct designed to allow the creation of a KSMFile to be performed step-by-step in a builder
/// fashion. In order to call the .finish() method, which returns the completed KSMFile,
/// at least one argument section and one debug section must be added. Likewise, at least one code section
/// must be added, although remember:
///
/// KSM files to be correctly loaded by kOS without a cryptic error, **must** contain
/// at least one entry in the debug section, and a Function code section **first**, followed
/// by an Initialization code section, followed by a Main code section with at least **one**
/// instruction in it.
///
#[must_use]
#[derive(Debug, Clone)]
pub struct KSMFileBuilder<Completeness = completeness::Empty> {
    argument_section: Option<ArgumentSection>,
    code_sections: Vec<CodeSection>,
    debug_section: Option<DebugSection>,
    completeness: PhantomData<Completeness>,
}

impl Default for KSMFileBuilder<completeness::Empty> {
    fn default() -> Self {
        KSMFileBuilder::new()
    }
}

impl KSMFileBuilder {
    /// Creates a new empty KSMFileBuilder
    pub const fn new() -> Self {
        Self {
            argument_section: None,
            code_sections: Vec::new(),
            debug_section: None,
            completeness: PhantomData,
        }
    }
}

impl KSMFileBuilder<completeness::Empty> {
    with_arg_section!(completeness::WithArgSection);

    with_code_section!(completeness::WithCodeSections);

    with_code_sections!(completeness::WithCodeSections);

    with_debug_section!(completeness::WithDebugSection);
}

impl KSMFileBuilder<completeness::WithArgSection> {
    with_code_section!(completeness::WithArgAndCode);

    with_code_sections!(completeness::WithArgAndCode);

    with_debug_section!(completeness::WithArgAndDebug);
}

impl KSMFileBuilder<completeness::WithCodeSections> {
    with_arg_section!(completeness::WithArgAndCode);

    with_code_section!(completeness::WithCodeSections);

    with_code_sections!(completeness::WithCodeSections);

    with_debug_section!(completeness::WithCodeAndDebug);
}

impl KSMFileBuilder<completeness::WithDebugSection> {
    with_arg_section!(completeness::WithArgAndDebug);

    with_code_section!(completeness::WithCodeAndDebug);

    with_code_sections!(completeness::WithCodeAndDebug);
}

impl KSMFileBuilder<completeness::WithArgAndCode> {
    with_code_section!(completeness::WithArgAndCode);

    with_code_sections!(completeness::WithArgAndCode);

    with_debug_section!(completeness::Complete);
}

impl KSMFileBuilder<completeness::WithArgAndDebug> {
    with_code_section!(completeness::Complete);

    with_code_sections!(completeness::Complete);
}

impl KSMFileBuilder<completeness::WithCodeAndDebug> {
    with_arg_section!(completeness::Complete);

    with_code_section!(completeness::WithCodeAndDebug);

    with_code_sections!(completeness::WithCodeAndDebug);
}

impl KSMFileBuilder<completeness::Complete> {
    with_code_section!(completeness::Complete);

    with_code_sections!(completeness::Complete);

    /// Returns the completed KSMFile that this builder has successfully created.
    ///
    /// KSM files to be correctly loaded by kOS without a cryptic error, **must** contain
    /// at least one entry in the debug section, and a Function code section **first**, followed
    /// by an Initialization code section, followed by a Main code section with at least **one**
    /// instruction in it. This cannot be statically guaranteed like mandatory argument and debug sections.
    ///
    pub fn finish(self) -> KSMFile {
        self.into()
    }
}

impl From<KSMFileBuilder<completeness::Complete>> for KSMFile {
    fn from(builder: KSMFileBuilder<completeness::Complete>) -> Self {
        Self::new_from_parts(
            builder.argument_section.unwrap(),
            builder.code_sections,
            builder.debug_section.unwrap(),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::ksm::sections::{
        ArgumentSection, CodeSection, CodeType, DebugEntry, DebugRange, DebugSection,
    };
    use crate::ksm::{Instr, KSMFile, KSMFileBuilder};
    use crate::{KOSValue, Opcode};

    #[test]
    fn nominal() {
        let arg_section = ArgumentSection::new()
            .with_arguments_unchecked(vec![
                KOSValue::Null,
                KOSValue::ArgMarker,
                KOSValue::ScalarInt(4),
            ])
            .0;

        let code_sections = vec![CodeSection::new(CodeType::Function)
            .with_instructions(vec![Instr::ZeroOp(Opcode::Add)])];

        let debug_section = DebugSection::new_empty()
            .with_entries(vec![DebugEntry::new(2).with_range(DebugRange::new(2, 10))]);

        let ksm1 = KSMFile::new_from_parts(
            arg_section.clone(),
            code_sections.clone(),
            debug_section.clone(),
        );

        let ksm2 = KSMFileBuilder::new()
            .with_arg_section(arg_section)
            .with_code_sections(code_sections)
            .with_debug_section(debug_section)
            .finish();

        assert_eq!(ksm1, ksm2);
    }

    #[test]
    fn multiple_code_additions() {
        let arg_section = ArgumentSection::new()
            .with_arguments_unchecked(vec![
                KOSValue::Null,
                KOSValue::ArgMarker,
                KOSValue::ScalarInt(4),
            ])
            .0;

        let code_section_1 = CodeSection::new(CodeType::Function)
            .with_instructions(vec![Instr::ZeroOp(Opcode::Add)]);

        let code_section_2 = CodeSection::new(CodeType::Initialization)
            .with_instructions(vec![Instr::ZeroOp(Opcode::Pop)]);

        let code_sections = vec![code_section_1.clone(), code_section_2.clone()];

        let debug_section = DebugSection::new_empty()
            .with_entries(vec![DebugEntry::new(2).with_range(DebugRange::new(2, 10))]);

        let ksm1 =
            KSMFile::new_from_parts(arg_section.clone(), code_sections, debug_section.clone());

        let ksm2 = KSMFileBuilder::new()
            .with_arg_section(arg_section)
            .with_code_section(code_section_1)
            .with_code_section(code_section_2)
            .with_debug_section(debug_section)
            .finish();

        assert_eq!(ksm1, ksm2);
    }
}
