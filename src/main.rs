use std::{error::Error, process, fs};

use kerbalobjects::{KOFile, KOFileWriter, KOFileReader, StringTable, SymbolTable, SymbolTableEntry, SymbolInfo, SymbolType};

fn main() {

    if let Err(e) = run() {
        eprintln!("Application error: {}", e);

        process::exit(1);
    }

}

fn run() -> Result<(), Box<dyn Error>> {

    println!("Writing object file");

    let mut kofile = KOFile::new();

    let mut strtab = StringTable::new(".comment");

    strtab.add("Compiled with a compiler");

    kofile.add_strtab(strtab);

    let mut symtab = SymbolTable::new(".symtab");

    symtab.add( SymbolTableEntry::new("main", 0, 0, SymbolInfo::LOCAL, SymbolType::SECTION, 2) );

    kofile.add_symtab(symtab);

    let mut kofile_writer = KOFileWriter::new("test.ko");

    kofile.write(&mut kofile_writer)?;

    kofile_writer.write_to_file()?;

    let contents = fs::read("test.ko")?;

    let mut kofile_reader = KOFileReader::new(contents)?;

    println!("\nReading object file");

    let _kofile = KOFile::read(&mut kofile_reader)?;

    Ok(())
}