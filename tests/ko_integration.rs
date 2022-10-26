#![cfg(feature = "ko")]
use std::io::{Read, Write};
use std::path::PathBuf;

use kerbalobjects::ko::sections::{DataIdx, InstrIdx};
use kerbalobjects::ko::symbols::OperandIndex;
use kerbalobjects::ko::SectionIdx;
use kerbalobjects::{
    ko::{
        instructions::Instr,
        symbols::{KOSymbol, ReldEntry},
        KOFile,
    },
    BufferIterator, KOSValue, Opcode, WritableBuffer,
};

#[test]
fn write_and_read_ko() {
    write_kofile();
    read_kofile();
}

#[test]
fn doc_test_real() {
    use kerbalobjects::ko::symbols::{KOSymbol, SymBind, SymType};
    use kerbalobjects::ko::{Instr, KOFile};
    use kerbalobjects::{KOSValue, Opcode};
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
}

fn write_kofile() {
    let mut ko = KOFile::new();

    let mut data_section = ko.new_data_section(".data");
    let mut start = ko.new_func_section("_start");
    let mut symtab = ko.new_symtab(".symtab");
    let mut symstrtab = ko.new_strtab(".symstrtab");
    let mut reld_section = ko.new_reld_section(".reld");

    let print_value = KOSValue::String(String::from("print()"));
    let print_value_index = data_section.add(print_value);

    let empty_value = KOSValue::String(String::from(""));
    let empty_value_index = data_section.add(empty_value);

    let two_value = KOSValue::ScalarInt(2);
    let two_value_index = data_section.add(two_value);

    let marker_value = KOSValue::ArgMarker;
    let marker_value_size = marker_value.size_bytes();
    let marker_value_index = data_section.add(marker_value);

    let marker_symbol_name_idx = symstrtab.add("marker");

    let marker_symbol = KOSymbol::new(
        marker_symbol_name_idx,
        marker_value_index,
        marker_value_size as u16,
        kerbalobjects::ko::symbols::SymBind::Global,
        kerbalobjects::ko::symbols::SymType::NoType,
        SectionIdx::from(2u16),
    );
    let marker_symbol_index = symtab.add(marker_symbol);

    let reld_entry = ReldEntry::new(
        SectionIdx::from(3u16),
        InstrIdx::from(0u32),
        OperandIndex::One,
        marker_symbol_index,
    );

    reld_section.add(reld_entry);

    let push_two_instr = Instr::OneOp(Opcode::Push, two_value_index);
    let add_instr = Instr::ZeroOp(Opcode::Add);
    let push_marker = Instr::OneOp(Opcode::Push, DataIdx::PLACEHOLDER);
    let call_print = Instr::TwoOp(Opcode::Call, empty_value_index, print_value_index);

    start.add(push_marker);
    start.add(push_two_instr);
    start.add(push_two_instr);
    start.add(add_instr);
    start.add(call_print);

    let start_symbol_name_idx = symstrtab.add("_start");
    let start_symbol = KOSymbol::new(
        start_symbol_name_idx,
        DataIdx::from(0u32),
        start.size() as u16,
        kerbalobjects::ko::symbols::SymBind::Global,
        kerbalobjects::ko::symbols::SymType::Func,
        SectionIdx::from(3u16),
    );

    let file_symbol_name_idx = symstrtab.add("test.ko");
    let file_symbol = KOSymbol::new(
        file_symbol_name_idx,
        DataIdx::from(0u32),
        0,
        kerbalobjects::ko::symbols::SymBind::Global,
        kerbalobjects::ko::symbols::SymType::File,
        SectionIdx::from(0u16),
    );

    symtab.add(file_symbol);
    symtab.add(start_symbol);

    ko.add_data_section(data_section);
    ko.add_func_section(start);
    ko.add_str_tab(symstrtab);
    ko.add_sym_tab(symtab);
    ko.add_reld_section(reld_section);

    // This is just a proof that all of WritableKOFile.write() is generic over impl WritableInterface, and proof that
    // you can implement WritableInterface.
    struct VecWrapper {
        vec: Vec<u8>,
    }

    impl VecWrapper {
        fn with_capacity(capacity: usize) -> Self {
            Self {
                vec: Vec::with_capacity(capacity),
            }
        }
    }

    impl WritableBuffer for VecWrapper {
        fn allocate_more(&mut self, amount: usize) {
            self.vec.reserve(amount);
        }

        fn write(&mut self, val: u8) {
            self.vec.push(val);
        }

        fn write_bytes(&mut self, val: &[u8]) {
            self.vec.extend_from_slice(val);
        }
    }

    let mut file_buffer = VecWrapper::with_capacity(2048);

    let ko = match ko.validate() {
        Ok(ko) => ko,
        Err(e) => {
            panic!("Could not update KO headers properly: {0} {0:?}", e.1);
        }
    };
    ko.write(&mut file_buffer);

    let file_path = PathBuf::from("tests").join("test.ko");
    let mut file =
        std::fs::File::create(file_path).expect("Output file could not be created: test.ko");

    file.write_all(file_buffer.vec.as_slice())
        .expect("File test.ko could not be written to.");
}

fn read_kofile() {
    let mut buffer = Vec::with_capacity(2048);
    let file_path = PathBuf::from("tests").join("test.ko");
    let mut file = std::fs::File::open(file_path).expect("Error opening test.ko");

    file.read_to_end(&mut buffer)
        .expect("Error reading test.ko");

    let mut buffer_iter = BufferIterator::new(&buffer);

    let _ko = KOFile::parse(&mut buffer_iter).expect("Error reading KO file");
}
