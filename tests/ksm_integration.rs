use std::io::{Read, Write};

use kerbalobjects::{
    ksmfile::{
        sections::{CodeSection, CodeType, DebugEntry, DebugRange},
        Instr, KSMFile,
    },
    FromBytes, KOSValue, Opcode, ToBytes,
};

#[test]
fn read_kos_ksm() {
    let mut buffer = Vec::with_capacity(2048);
    let mut file = std::fs::File::open("example.ksm").expect("Error opening KSM file");

    file.read_to_end(&mut buffer)
        .expect("Error reading example.ksm");

    let mut buffer_iter = buffer.iter().peekable();

    let _ksm = KSMFile::from_bytes(&mut buffer_iter).expect("Error reading KSM file");
}

#[test]
fn read_write_ksm() {
    write_ksm();
    read_ksm();
}

fn write_ksm() {
    let mut ksm = KSMFile::new();

    let print = KOSValue::String(String::from("print()"));

    let empty = KOSValue::String(String::from(""));

    let two = KOSValue::ScalarInt(2);

    let marker = KOSValue::ArgMarker;

    let first = KOSValue::String(String::from("@0001"));

    let one = KOSValue::Int16(1);

    let zero = KOSValue::Int16(0);

    let arg_section = ksm.arg_section_mut();

    let print_index = arg_section.add(print);
    let empty_index = arg_section.add(empty);
    let two_index = arg_section.add(two);
    let marker_index = arg_section.add(marker);
    let first_index = arg_section.add(first);
    let one_index = arg_section.add(one);
    let zero_index = arg_section.add(zero);

    let push_marker = Instr::OneOp(Opcode::Push, marker_index);
    let add_instr = Instr::ZeroOp(Opcode::Add);
    let push_two_instr = Instr::OneOp(Opcode::Push, two_index);
    let call_print = Instr::TwoOp(Opcode::Call, empty_index, print_index);
    let lbrt_instr = Instr::OneOp(Opcode::Lbrt, first_index);
    let begin_scope = Instr::TwoOp(Opcode::Bscp, one_index, zero_index);
    let pop_instr = Instr::ZeroOp(Opcode::Pop);
    let end_scope = Instr::OneOp(Opcode::Escp, one_index);
    let argb_instr = Instr::ZeroOp(Opcode::Argb);

    let mut main_section = CodeSection::new(CodeType::Main);
    let init_section = CodeSection::new(CodeType::Initialization);
    let func_section = CodeSection::new(CodeType::Function);

    main_section.add(lbrt_instr);
    main_section.add(begin_scope);
    main_section.add(argb_instr);
    main_section.add(push_marker);
    main_section.add(push_two_instr);
    main_section.add(push_two_instr);
    main_section.add(add_instr);
    main_section.add(call_print);
    main_section.add(pop_instr);
    main_section.add(end_scope);

    ksm.add_code_section(func_section);
    ksm.add_code_section(init_section);
    ksm.add_code_section(main_section);

    let mut debug_entry = DebugEntry::new(1);
    debug_entry.add(DebugRange::new(6, 0x18));

    ksm.debug_section_mut().add(debug_entry);

    let mut file_buffer = Vec::with_capacity(2048);

    ksm.to_bytes(&mut file_buffer);

    let mut file = std::fs::File::create("test.ksm").expect("Error opening test.ksm");

    file.write_all(file_buffer.as_slice())
        .expect("test.ksm could not be written to.");
}

fn read_ksm() {
    let mut buffer = Vec::with_capacity(2048);
    let mut file = std::fs::File::open("test.ksm").expect("Error opening test.ksm");

    file.read_to_end(&mut buffer)
        .expect("Error reading test.ksm");

    let mut buffer_iter = buffer.iter().peekable();

    let _ksm = KSMFile::from_bytes(&mut buffer_iter);
}
