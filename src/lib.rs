//!
//! # KerbalObjects
//!
//! The `kerbalobjects` crate provides an interface for creating object files for
//! [Kerbal Operating System](https://ksp-kos.github.io/KOS/) (kOS), It supports reading and writing
//! Kerbal Object files, and Kerbal Machine Code (kOS executable) files.
//!
//! Documentation for both file formats can be found here:
//! * For [Kerbal Object](https://github.com/newcomb-luke/kerbalobjects.rs/blob/main/docs/KO-file-format.md) files.
//! * For [Kerbal Machine code](https://github.com/newcomb-luke/kerbalobjects.rs/blob/main/docs/KSM-file-format.md) files.
//!
//! Kerbal Object files are used by the Kerbal Assembler and Kerbal Linker toolchain, and eventually are linked into
//! Kerbal Machine Code files, which are loadable and executable by kOS.
//!
//! # Example for Kerbal Machine Code hello world program
//!
//! ```
//! # #[cfg(feature = "ksm")] {
//! use std::io::Write;
//! use kerbalobjects::ksm::sections::{ArgumentSection, CodeSection, CodeType, DebugEntry, DebugRange, DebugSection};
//! use kerbalobjects::ksm::{Instr, KSMFile};
//! use kerbalobjects::{Opcode, KOSValue, ToBytes};
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
//! // A completely wrong and useless debug section, but we NEED to have one
//! let mut debug_entry =  DebugEntry::new(1).with_range(DebugRange::new(0x06, 0x13));
//!
//! let debug_section = DebugSection::new(debug_entry);
//!
//! let mut file_buffer = Vec::with_capacity(2048);
//!
//! let ksm_file = KSMFile::new_from_parts(arg_section, code_sections, debug_section);
//!
//! ksm_file.write(&mut file_buffer);
//!
//! let mut file = std::fs::File::create("hello.ksm").expect("Couldn't open output file");
//!
//! file.write_all(file_buffer.as_slice()).expect("Failed to write to output file");
//! # }
//! ```
//!
//! # Example for Kerbal Object hello world program
//!
//! ```
//! #[cfg(feature = "ko")] {
//! use kerbalobjects::ko::symbols::{KOSymbol, SymBind, SymType};
//! use kerbalobjects::ko::{Instr, KOFile};
//! use kerbalobjects::{KOSValue, Opcode};
//! use kerbalobjects::ko::SectionIdx;
//! use kerbalobjects::ko::sections::DataIdx;
//! use std::io::Write;
//! use std::path::PathBuf;
//!
//! let mut ko = KOFile::new();
//!
//! let mut data_section = ko.new_data_section(".data");
//! let mut start = ko.new_func_section("_start");
//! let mut symtab = ko.new_symtab(".symtab");
//! let mut symstrtab = ko.new_strtab(".symstrtab");
//!
//! // Set up the main code function section
//! let one = data_section.add_checked(KOSValue::Int16(1));
//!
//! start.add(Instr::TwoOp(
//!     Opcode::Bscp,
//!     one,
//!     data_section.add_checked(KOSValue::Int16(0)),
//! ));
//! start.add(Instr::ZeroOp(Opcode::Argb));
//! start.add(Instr::OneOp(
//!     Opcode::Push,
//!     data_section.add_checked(KOSValue::ArgMarker),
//! ));
//! start.add(Instr::OneOp(
//!     Opcode::Push,
//!     data_section.add_checked(KOSValue::StringValue("Hello, world!".into())),
//! ));
//! start.add(Instr::TwoOp(
//!     Opcode::Call,
//!     data_section.add_checked(KOSValue::String("".into())),
//!     data_section.add_checked(KOSValue::String("print()".into())),
//! ));
//! start.add(Instr::ZeroOp(Opcode::Pop));
//! start.add(Instr::OneOp(Opcode::Escp, one));
//!
//! // Set up our symbols
//! let file_symbol = KOSymbol::new(
//!     symstrtab.add("test.kasm"),
//!     DataIdx::PLACEHOLDER,
//!     0,
//!     SymBind::Global,
//!     SymType::File,
//!     SectionIdx::NULL,
//! );
//! let start_symbol = KOSymbol::new(
//!     symstrtab.add("_start"),
//!     DataIdx::PLACEHOLDER,
//!     start.size() as u16,
//!     SymBind::Global,
//!     SymType::Func,
//!     start.section_index(),
//! );
//!
//! symtab.add(file_symbol);
//! symtab.add(start_symbol);
//!
//! ko.add_data_section(data_section);
//! ko.add_func_section(start);
//! ko.add_str_tab(symstrtab);
//! ko.add_sym_tab(symtab);
//!
//! // Write the file out to disk
//! let mut file_buffer = Vec::with_capacity(2048);
//!
//! let ko = ko.validate().expect("Could not update KO headers properly");
//! ko.write(&mut file_buffer);
//!
//! let file_path = PathBuf::from("test.ko");
//! let mut file =
//!     std::fs::File::create(file_path).expect("Output file could not be created: test.ko");
//!
//! file.write_all(file_buffer.as_slice())
//!     .expect("File test.ko could not be written to.");
//! # }
//! ```
//!

#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![allow(clippy::result_unit_err)]

mod common;
pub use common::*;

pub mod errors;
pub use errors::*;

#[cfg(feature = "ko")]
pub mod ko;
#[cfg(feature = "ksm")]
pub mod ksm;
