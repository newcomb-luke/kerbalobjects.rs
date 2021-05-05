use kerbalobjects::{kofile::KOFile, FromBytes};
use std::io::Read;

#[test]
fn read_kofile() {
    let mut buffer = Vec::with_capacity(2048);
    let mut file = std::fs::File::open("test.ko").expect("Error opening test.ko");

    file.read_to_end(&mut buffer)
        .expect("Error reading test.ko");

    let mut buffer_iter = buffer.iter();

    let _ko = KOFile::from_bytes(&mut buffer_iter).expect("Error reading KO file");
}
