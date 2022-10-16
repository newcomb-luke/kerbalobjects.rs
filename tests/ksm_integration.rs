use std::io::{Read, Write};
use std::path::PathBuf;

use kerbalobjects::{
    ksm::{
        sections::{CodeSection, CodeType, DebugEntry, DebugRange},
        Instr, KSMFile,
    },
    FileIterator, FromBytes, KOSValue, Opcode, ToBytes,
};

/// This test is simply a large file that kerbalobjects should be able to read
/// correctly. It doesn't yet test the 3 byte argument index size, but it is somewhat
/// a stress test. It is provided by the [kosos repository](https://gitlab.com/thexa4/kosos),
/// which is a collection of kOS scripts making up "kOS operating system" and a self-hosting compiler.
#[test]
fn read_kosos_kash() {
    let mut buffer = Vec::with_capacity(2048);
    let file_path = PathBuf::from("tests").join("kash.ksm");
    let mut file = std::fs::File::open(file_path).expect("Error opening KSM file");

    file.read_to_end(&mut buffer)
        .expect("Error reading kash.ksm");

    let mut buffer_iter = FileIterator::new(&buffer);

    let expected_args = vec![
        KOSValue::String("@0001".into()),
        KOSValue::Int16(1),
        KOSValue::Int16(0),
        KOSValue::StringValue("kpp 1.1".into()),
        KOSValue::StringValue("https://gitlab.com/thexa4/kos-kpp".into()),
        KOSValue::String("$class".into()),
        KOSValue::String("__new".into()),
        KOSValue::ArgMarker,
        KOSValue::String("@0481".into()),
        KOSValue::Int16(2),
        KOSValue::String("$r".into()),
        KOSValue::String("@0063".into()),
        KOSValue::Int16(3),
    ];

    let ksm = KSMFile::from_bytes(&mut buffer_iter).expect("Error reading KSM file");

    let mut arg_section_args = ksm.arg_section.arguments();

    for arg in expected_args {
        assert_eq!(&arg, arg_section_args.next().unwrap());
    }

    let mut code_sections = ksm.code_sections();

    code_sections.next().unwrap();
    code_sections.next().unwrap();

    let main_code = code_sections.next().unwrap();

    assert!(code_sections.next().is_none());

    let expected_instrs = vec![
        Opcode::Lbrt,
        Opcode::Bscp,
        Opcode::Argb,
        Opcode::Push,
        Opcode::Pop,
        Opcode::Push,
        Opcode::Pop,
        Opcode::Push,
        Opcode::Gmet,
        Opcode::Push,
        Opcode::Push,
        Opcode::Jmp,
        Opcode::Bscp,
        Opcode::Stol,
        Opcode::Argb,
        Opcode::Push,
    ];

    let mut instructions = main_code.instructions();

    for opcode in expected_instrs {
        assert_eq!(opcode, instructions.next().unwrap().opcode());
    }
}

#[test]
fn read_kos_ksm() {
    let mut buffer = Vec::with_capacity(2048);
    let file_path = PathBuf::from("tests").join("example.ksm");
    let mut file = std::fs::File::open(file_path).expect("Error opening KSM file");

    file.read_to_end(&mut buffer)
        .expect("Error reading example.ksm");

    let mut buffer_iter = FileIterator::new(&buffer);

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

    let arg_section = &mut ksm.arg_section;

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

    ksm.debug_section.add(debug_entry);

    let mut file_buffer = Vec::with_capacity(2048);

    ksm.to_bytes(&mut file_buffer);

    let file_path = PathBuf::from("tests").join("test.ksm");
    let mut file = std::fs::File::create(file_path).expect("Error opening test.ksm");

    file.write_all(file_buffer.as_slice())
        .expect("test.ksm could not be written to.");
}

fn read_ksm() {
    let mut buffer = Vec::with_capacity(2048);
    let file_path = PathBuf::from("tests").join("test.ksm");
    let mut file = std::fs::File::open(file_path).expect("Error opening test.ksm");

    file.read_to_end(&mut buffer)
        .expect("Error reading test.ksm");

    let mut buffer_iter = FileIterator::new(&buffer);

    let _ksm = KSMFile::from_bytes(&mut buffer_iter);
}
