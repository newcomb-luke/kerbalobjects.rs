use std::{error::Error, process, fs};

use kerbalobjects::{KOFile, KOFileWriter, KOFileReader, RelSection, RelInstruction, Symbol, KOSValue, SymbolInfo, SymbolType};

fn main() {

    if let Err(e) = run() {
        eprintln!("Application error: {}", e);

        process::exit(1);
    }

}

fn run() -> Result<(), Box<dyn Error>> {

    println!("Writing object file");

    let mut writer = KOFileWriter::new("test.ko");

    let mut kofile = KOFile::new();

    let first_label = kofile.add_symbol(Symbol::new("", KOSValue::STRING(String::from("@0001")), 6, SymbolInfo::LOCAL, SymbolType::NOTYPE, 2));
    let _start_func = kofile.add_symbol(Symbol::new("_start", KOSValue::NULL, 0, SymbolInfo::GLOBAL, SymbolType::FUNC, 4));
    let main_scope = kofile.add_symbol(Symbol::new("", KOSValue::INT16(1), 0, SymbolInfo::LOCAL, SymbolType::NOTYPE, 2));
    let outer_scope = kofile.add_symbol(Symbol::new("", KOSValue::INT16(0), 0, SymbolInfo::LOCAL, SymbolType::NOTYPE, 2));
    let arg_marker = kofile.add_symbol(Symbol::new("", KOSValue::ARGMARKER, 0, SymbolInfo::LOCAL, SymbolType::NOTYPE, 2));
    let b = kofile.add_symbol(Symbol::new("", KOSValue::BOOL(false), 0, SymbolInfo::LOCAL, SymbolType::NOTYPE, 2));
    let lib_name = kofile.add_symbol(Symbol::new("", KOSValue::STRINGVALUE(String::from("loaded")), 0, SymbolInfo::LOCAL, SymbolType::NOTYPE, 2));
    let loader_label = kofile.add_symbol(Symbol::new("", KOSValue::STRING(String::from("@LR00")), 0, SymbolInfo::LOCAL, SymbolType::NOTYPE, 2));
    let n = kofile.add_symbol(Symbol::new("", KOSValue::NULL, 0, SymbolInfo::LOCAL, SymbolType::NOTYPE, 2));
    let msg = kofile.add_symbol(Symbol::new("", KOSValue::STRINGVALUE(String::from("Loaded.")), 0, SymbolInfo::LOCAL, SymbolType::NOTYPE, 2));
    let empty = kofile.add_symbol(Symbol::new("", KOSValue::STRING(String::from("")), 0, SymbolInfo::LOCAL, SymbolType::NOTYPE, 2));
    let print = kofile.add_symbol(Symbol::new("", KOSValue::STRING(String::from("print()")), 0, SymbolInfo::LOCAL, SymbolType::NOTYPE, 2));

    let mut main_text = RelSection::new(".text");

    main_text.add(RelInstruction::new(0xf0, vec![ first_label as u32 ]));
    main_text.add(RelInstruction::new(0x5a, vec![ main_scope as u32, outer_scope as u32 ]));
    main_text.add(RelInstruction::new(0x60, vec![ ] ));
    main_text.add(RelInstruction::new(0x4e, vec![ arg_marker as u32 ] ));
    main_text.add(RelInstruction::new(0x4e, vec![ arg_marker as u32 ] ));
    main_text.add(RelInstruction::new(0x4e, vec![ b as u32 ] ));
    main_text.add(RelInstruction::new(0x4e, vec![ lib_name as u32 ] ));
    main_text.add(RelInstruction::new(0x52, vec![ ]));
    main_text.add(RelInstruction::new(0x4c, vec![ loader_label as u32, n as u32 ] ));
    main_text.add(RelInstruction::new(0x4f, vec![ ] ));
    main_text.add(RelInstruction::new(0x4e, vec![ arg_marker as u32 ] ));
    main_text.add(RelInstruction::new(0x4e, vec![ msg as u32 ] ));
    main_text.add(RelInstruction::new(0x4c, vec![ empty as u32, print as u32 ] ));
    main_text.add(RelInstruction::new(0x4f, vec![ ] ));
    main_text.add(RelInstruction::new(0x5b, vec![ main_scope as u32 ] ));

    kofile.add_code_section(main_text);

    kofile.write(&mut writer)?;

    writer.write_to_file()?;

    println!("\nReading object file");

    let raw_contents = fs::read("test.ko")?;

    let mut reader = KOFileReader::new(raw_contents)?;

    let _kofile = KOFile::read(&mut reader)?;
    
    Ok(())
}