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

    let symbol_2 = Symbol::new("_2", KOSValue::SCALARINT(2), 5, SymbolInfo::LOCAL, SymbolType::NOTYPE, 2);

    let symbol_argmarker = Symbol::new("_argmarker", KOSValue::ARGMARKER, 1, SymbolInfo::LOCAL, SymbolType::NOTYPE, 2);

    let symbol_print = Symbol::new("_print", KOSValue::STRING(String::from("print()")), 9, SymbolInfo::LOCAL, SymbolType::NOTYPE, 2);

    let symbol_empty = Symbol::new("_", KOSValue::STRING(String::new()), 2, SymbolInfo::LOCAL, SymbolType::NOTYPE, 2);

    kofile.add_symbol(symbol_2);
    kofile.add_symbol(symbol_argmarker);
    kofile.add_symbol(symbol_print);
    kofile.add_symbol(symbol_empty);

    let mut main_text = RelSection::new(".text");

    // push ARGMARKER
    // push 2
    // push 2
    // add
    // call print()

    main_text.add(RelInstruction::new(0x4e, vec![ 1 ]));
    main_text.add(RelInstruction::new(0x4e, vec![ 0 ]));
    main_text.add(RelInstruction::new(0x4e, vec![ 0 ]));
    main_text.add(RelInstruction::new(0x3c, Vec::new()));
    main_text.add(RelInstruction::new(0x4c, vec![ 3, 2 ]));

    kofile.set_main_text(main_text);

    kofile.write(&mut writer)?;

    writer.write_to_file()?;

    println!("\nReading object file");

    let raw_contents = fs::read("test.ko")?;

    let mut reader = KOFileReader::new(raw_contents)?;

    let _kofile = KOFile::read(&mut reader)?;

    Ok(())
}