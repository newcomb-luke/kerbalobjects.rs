use std::io::{Read, Write};

use kerbalobjects::{
    kofile::{
        instructions::Instr,
        symbols::{KOSymbol, ReldEntry},
        KOFile,
    },
    FromBytes, KOSValue, Opcode, ToBytes,
};

#[test]
fn write_and_read_ko() {
    write_kofile();
    read_kofile();
}

fn write_kofile() {
    let mut ko = KOFile::new();

    let mut data_section = ko.new_datasection(".data");
    let mut start = ko.new_funcsection("_start");
    let mut symtab = ko.new_symtab(".symtab");
    let mut symstrtab = ko.new_strtab(".symstrtab");
    let mut reld_section = ko.new_reldsection(".reld");

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
        kerbalobjects::kofile::symbols::SymBind::Global,
        kerbalobjects::kofile::symbols::SymType::NoType,
        2,
    );
    let marker_symbol_index = symtab.add(marker_symbol);

    let reld_entry = ReldEntry::new(3, 0, 0, marker_symbol_index);

    reld_section.add(reld_entry);

    let push_two_instr = Instr::OneOp(Opcode::Push, two_value_index);
    let add_instr = Instr::ZeroOp(Opcode::Add);
    let push_marker = Instr::OneOp(Opcode::Push, marker_symbol_index);
    let call_print = Instr::TwoOp(Opcode::Call, empty_value_index, print_value_index);

    start.add(push_marker);
    start.add(push_two_instr);
    start.add(push_two_instr);
    start.add(add_instr);
    start.add(call_print);

    let start_symbol_name_idx = symstrtab.add("_start");
    let start_symbol = KOSymbol::new(
        start_symbol_name_idx,
        0,
        start.size() as u16,
        kerbalobjects::kofile::symbols::SymBind::Global,
        kerbalobjects::kofile::symbols::SymType::Func,
        3,
    );

    let file_symbol_name_idx = symstrtab.add("test.ko");
    let file_symbol = KOSymbol::new(
        file_symbol_name_idx,
        0,
        0,
        kerbalobjects::kofile::symbols::SymBind::Global,
        kerbalobjects::kofile::symbols::SymType::File,
        0,
    );

    symtab.add(file_symbol);
    symtab.add(start_symbol);

    ko.add_data_section(data_section);
    ko.add_func_section(start);
    ko.add_str_tab(symstrtab);
    ko.add_sym_tab(symtab);
    ko.add_reld_section(reld_section);

    let mut file_buffer = Vec::with_capacity(2048);

    ko.update_headers()
        .expect("Could not update KO headers properly");
    ko.to_bytes(&mut file_buffer);

    let mut file =
        std::fs::File::create("test.ko").expect("Output file could not be created: test.ko");

    file.write_all(file_buffer.as_slice())
        .expect("File test.ko could not be written to.");
}

fn read_kofile() {
    let mut buffer = Vec::with_capacity(2048);
    let mut file = std::fs::File::open("test.ko").expect("Error opening test.ko");

    file.read_to_end(&mut buffer)
        .expect("Error reading test.ko");

    let mut buffer_iter = buffer.iter().peekable();

    let _ko = KOFile::from_bytes(&mut buffer_iter, true).expect("Error reading KO file");
}
