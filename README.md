# KerbalObjects

[<img src="https://img.shields.io/badge/github-newcomb--luke%2Fkerbalobjects.rs-8da0cb?style=for-the-badge&logo=github&labelColor=555555" alt="github" height="20">](https://github.com/newcomb-luke/kerbalobjects.rs)
[<img src="https://img.shields.io/crates/v/kerbalobjects?color=fc8d62&logo=rust&style=for-the-badge" alt="github" height="20">](https://crates.io/crates/kerbalobjects)
[<img src="https://img.shields.io/badge/docs.rs-kerbalobjects-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" alt="github" height="20">](https://docs.rs/kerbalobjects/latest/kerbalobjects/)
[<img alt="License" src="https://img.shields.io/github/license/newcomb-luke/kerbalobjects.rs?style=for-the-badge" height="20">]()
[<img alt="Crates.io Downloads" src="https://img.shields.io/crates/d/kerbalobjects?style=for-the-badge" height="20">]()

[<img alt="GitHub Workflow Status" src="https://img.shields.io/github/actions/workflow/status/newcomb-luke/kerbalobjects.rs/main.yml?style=for-the-badge" height="20">]()
[<img alt="Libraries.io dependency status for GitHub repo" src="https://img.shields.io/librariesio/github/newcomb-luke/kerbalobjects.rs?style=for-the-badge" height="20">](https://deps.rs/repo/github/newcomb-luke/kerbalobjects.rs)

A Rust crate that allows anyone to read or write a Kerbal Machine Code file or Kerbal Object file.

```toml
[dependencies]
kerbalobjects = "4.0"
```

## Examples

### Example for Kerbal Machine Code hello world program

```rust
use std::io::Write;
use kerbalobjects::ksm::sections::{ArgumentSection, CodeSection, CodeType, DebugEntry, DebugRange, DebugSection};
use kerbalobjects::ksm::{Instr, KSMFile};
use kerbalobjects::{Opcode, KOSValue, ToBytes};

let mut arg_section = ArgumentSection::new();
let mut main_code = CodeSection::new(CodeType::Main);

let one = arg_section.add_checked(KOSValue::Int16(1));

// Corresponds to the KerbalScript code:
// PRINT("Hello, world!").

main_code.add(Instr::OneOp(Opcode::Push, arg_section.add_checked(KOSValue::String("@0001".into()))));
main_code.add(Instr::TwoOp(Opcode::Bscp, one, arg_section.add_checked(KOSValue::Int16(0))));
main_code.add(Instr::ZeroOp(Opcode::Argb));
main_code.add(Instr::OneOp(Opcode::Push, arg_section.add_checked(KOSValue::ArgMarker)));
main_code.add(Instr::OneOp(Opcode::Push, arg_section.add_checked(KOSValue::StringValue("Hello, world!".into()))));
main_code.add(Instr::TwoOp(Opcode::Call, arg_section.add_checked(KOSValue::String("".into())), arg_section.add_checked(KOSValue::String("print()".into()))));
main_code.add(Instr::ZeroOp(Opcode::Pop));
main_code.add(Instr::OneOp(Opcode::Escp, one));

let code_sections = vec![
    CodeSection::new(CodeType::Function),
    CodeSection::new(CodeType::Initialization),
    main_code
];

// A completely wrong and useless debug section, but we NEED to have one
let mut debug_entry =  DebugEntry::new(1).with_range(DebugRange::new(0x06, 0x13));

let debug_section = DebugSection::new(debug_entry);

let mut file_buffer = Vec::with_capacity(2048);

let ksm_file = KSMFile::new_from_parts(arg_section, code_sections, debug_section);

ksm_file.write(&mut file_buffer);

let mut file = std::fs::File::create("hello.ksm").expect("Couldn't open output file");

file.write_all(file_buffer.as_slice()).expect("Failed to write to output file");
```

### Example for Kerbal Object hello world program

```rust
use kerbalobjects::ko::symbols::{KOSymbol, SymBind, SymType};
use kerbalobjects::ko::{Instr, KOFile};
use kerbalobjects::{KOSValue, Opcode};
use kerbalobjects::ko::SectionIdx;
use kerbalobjects::ko::sections::DataIdx;
use std::io::Write;
use std::path::PathBuf;

let mut ko = KOFile::new();

let mut data_section = ko.new_data_section(".data");
let mut start = ko.new_func_section("_start");
let mut symtab = ko.new_symtab(".symtab");
let mut symstrtab = ko.new_strtab(".symstrtab");

// Set up the main code function section
let one = data_section.add_checked(KOSValue::Int16(1));

start.add(Instr::TwoOp(
    Opcode::Bscp,
    one,
    data_section.add_checked(KOSValue::Int16(0)),
));
start.add(Instr::ZeroOp(Opcode::Argb));
start.add(Instr::OneOp(
    Opcode::Push,
    data_section.add_checked(KOSValue::ArgMarker),
));
start.add(Instr::OneOp(
    Opcode::Push,
    data_section.add_checked(KOSValue::StringValue("Hello, world!".into())),
));
start.add(Instr::TwoOp(
    Opcode::Call,
    data_section.add_checked(KOSValue::String("".into())),
    data_section.add_checked(KOSValue::String("print()".into())),
));
start.add(Instr::ZeroOp(Opcode::Pop));
start.add(Instr::OneOp(Opcode::Escp, one));

// Set up our symbols
let file_symbol = KOSymbol::new(
    symstrtab.add("test.kasm"),
    DataIdx::PLACEHOLDER,
    0,
    SymBind::Global,
    SymType::File,
    SectionIdx::NULL,
);
let start_symbol = KOSymbol::new(
    symstrtab.add("_start"),
    DataIdx::PLACEHOLDER,
    start.size() as u16,
    SymBind::Global,
    SymType::Func,
    start.section_index(),
);

symtab.add(file_symbol);
symtab.add(start_symbol);

ko.add_data_section(data_section);
ko.add_func_section(start);
ko.add_str_tab(symstrtab);
ko.add_sym_tab(symtab);

// Write the file out to disk
let mut file_buffer = Vec::with_capacity(2048);

let ko = ko.validate().expect("Could not update KO headers properly");
ko.write(&mut file_buffer);

let file_path = PathBuf::from("test.ko");
let mut file =
    std::fs::File::create(file_path).expect("Output file could not be created: test.ko");

file.write_all(file_buffer.as_slice())
    .expect("File test.ko could not be written to.");
```

## Documentation

See the kerbalobjects [docs.rs](https://docs.rs/kerbalobjects/latest/kerbalobjects/) for information on how to use this library.

Documentation on kOS [instructions](https://github.com/newcomb-luke/kerbalobjects.rs/blob/main/docs/Instruction-docs.md) and what they do. 

## Support

Besides the documentation, support can be found by the library author in this [Discord server](https://discord.gg/APETM2ceVZ).

## If this doesn't fit your use case

If this library doesn't implement a specific feature that your program needs, then please create a new issue or contact the developer.

If that cannot be resolved, see docs/ for examples and explanations of the [KSM](https://github.com/newcomb-luke/kerbalobjects.rs/blob/main/docs/KSM-file-format.md) and [KO](https://github.com/newcomb-luke/kerbalobjects.rs/blob/main/docs/KO-file-format.md) file formats and how to create or read them.
