use std::io::Read;

use kerbalobjects::{ksmfile::KSMFile, FromBytes};

#[test]
fn read_kos_ksm() {
    let mut buffer = Vec::with_capacity(2048);
    let mut file = std::fs::File::open("example.ksm").expect("Error opening KSM file");

    file.read_to_end(&mut buffer)
        .expect("Error reading example.ksm");

    let mut buffer_iter = buffer.iter().peekable();

    let _ksm = KSMFile::from_bytes(&mut buffer_iter, true).expect("Error reading KSM file");
}
