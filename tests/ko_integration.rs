use std::io::{Read, Write};

use kerbalobjects::{
    kofile::{instructions::Instr, KOFile},
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

    let print_value = KOSValue::String(String::from("print()"));
    let print_value_index = data_section.add(print_value);

    let empty_value = KOSValue::String(String::from(""));
    let empty_value_index = data_section.add(empty_value);

    let two_value = KOSValue::ScalarInt(2);
    let two_value_index = data_section.add(two_value);

    let marker_value = KOSValue::ArgMarker;
    let marker_value_index = data_section.add(marker_value);

    let push_two_instr = Instr::OneOp(Opcode::Push, two_value_index);
    let add_instr = Instr::ZeroOp(Opcode::Add);
    let push_marker = Instr::OneOp(Opcode::Push, marker_value_index);
    let call_print = Instr::TwoOp(Opcode::Call, empty_value_index, print_value_index);

    start.add(push_marker);
    start.add(push_two_instr);
    start.add(push_two_instr);
    start.add(add_instr);
    start.add(call_print);

    ko.add_data_section(data_section);
    ko.add_func_section(start);

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
