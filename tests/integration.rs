use std::io::{Read, Write};

use kerbalobjects::{
    kofile::{
        self,
        instructions::{Instr, Opcode},
        sections::{DataSection, RelSection, SectionHeader, StringTable, SymbolTable},
        symbols::{KOSymbol, SymBind, SymType},
        KOFile,
    },
    FromBytes, KOSValue, ToBytes,
};

#[test]
fn write_and_read() {
    write_kofile();
    read_kofile();
}

fn write_kofile() {
    let mut ko = KOFile::new();

    let symstrtab_str_index = ko.add_shstr(".symstrtab");
    let sh_symstrtab =
        SectionHeader::new(symstrtab_str_index, kofile::sections::SectionKind::StrTab);
    let sh_symstrtab_index = ko.add_header(sh_symstrtab);

    let symtab_str_index = ko.add_shstr(".symtab");
    let sh_symtab = SectionHeader::new(symtab_str_index, kofile::sections::SectionKind::SymTab);
    let sh_symtab_index = ko.add_header(sh_symtab);

    let datatab_str_index = ko.add_shstr(".data");
    let sh_datatab = SectionHeader::new(datatab_str_index, kofile::sections::SectionKind::Data);
    let sh_datatab_index = ko.add_header(sh_datatab);

    let text_str_index = ko.add_shstr(".text");
    let sh_text = SectionHeader::new(text_str_index, kofile::sections::SectionKind::Rel);
    let sh_text_index = ko.add_header(sh_text);

    let mut symstrtab = StringTable::new(2, sh_symstrtab_index);
    let mut symtab = SymbolTable::new(2, sh_symtab_index);
    let mut datatab = DataSection::new(2, sh_datatab_index);
    let mut text = RelSection::new(2, sh_text_index);

    let file_sym_str = symstrtab.add("test.kasm");
    let mut file_sym = KOSymbol::new(
        0,
        0,
        kofile::symbols::SymBind::Local,
        kofile::symbols::SymType::File,
        0,
    );
    file_sym.set_name_idx(file_sym_str);
    symtab.add(file_sym);

    let print_value = KOSValue::String(String::from("print()"));
    let print_value_size = print_value.size_bytes();
    let print_value_index = datatab.add(print_value);
    let print_sym = KOSymbol::new(
        print_value_index,
        print_value_size as u16,
        kofile::symbols::SymBind::Local,
        kofile::symbols::SymType::NoType,
        sh_datatab_index as u16,
    );
    let print_sym_index = symtab.add(print_sym);

    let empty_value = KOSValue::String(String::from(""));
    let empty_value_size = empty_value.size_bytes();
    let empty_value_index = datatab.add(empty_value);
    let empty_sym = KOSymbol::new(
        empty_value_index,
        empty_value_size as u16,
        kofile::symbols::SymBind::Local,
        kofile::symbols::SymType::NoType,
        sh_datatab_index as u16,
    );
    let empty_sym_index = symtab.add(empty_sym);

    let two_value = KOSValue::ScalarInt(2);
    let two_value_size = two_value.size_bytes();
    let two_value_index = datatab.add(two_value);
    let two_sym = KOSymbol::new(
        two_value_index,
        two_value_size as u16,
        kofile::symbols::SymBind::Local,
        kofile::symbols::SymType::NoType,
        sh_datatab_index as u16,
    );
    let two_sym_index = symtab.add(two_sym);

    let marker_value = KOSValue::ArgMarker;
    let marker_value_size = marker_value.size_bytes();
    let marker_value_index = datatab.add(marker_value);
    let marker_sym = KOSymbol::new(
        marker_value_index,
        marker_value_size as u16,
        SymBind::Local,
        SymType::NoType,
        sh_datatab_index as u16,
    );
    let marker_sym_index = symtab.add(marker_sym);

    let push_two_instr = Instr::OneOp(Opcode::Push, two_sym_index);
    let add_instr = Instr::ZeroOp(Opcode::Add);
    let push_marker = Instr::OneOp(Opcode::Push, marker_sym_index);
    let call_print = Instr::TwoOp(Opcode::Call, empty_sym_index, print_sym_index);

    text.add(push_marker);
    text.add(push_two_instr);
    text.add(push_two_instr);
    text.add(add_instr);
    text.add(call_print);

    ko.add_str_tab(symstrtab);
    ko.add_sym_tab(symtab);
    ko.add_data_section(datatab);
    ko.add_rel_section(text);

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

    let mut buffer_iter = buffer.iter();

    let _ko = KOFile::from_bytes(&mut buffer_iter).expect("Error reading KO file");
}
