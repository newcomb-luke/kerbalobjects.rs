use std::{error::Error ,fs::File};
use std::io::prelude::*;

pub struct KOFileWriter {
    filename: String,
    current_index: usize,
    contents: Vec<u8>,
}

impl KOFileWriter {

    pub fn new(filename: &str) -> KOFileWriter {
        KOFileWriter {
            filename: String::from(filename),
            current_index: 0,
            contents: Vec::new(),
        }
    }

    pub fn write_to_file(&mut self) -> Result<(), Box<dyn Error>> {

        let mut file = File::create(&self.filename)?;

        file.write_all( self.contents.as_slice() )?;

        Ok(())
    }

    /// Returns the current index of the reader into the byte vector
    pub fn get_current_index(&self) -> usize {
        self.current_index
    }

    pub fn write(&mut self, byte: u8) -> Result<(), Box<dyn Error>> {
        self.contents.push(byte);

        Ok(())
    }

    pub fn write_multiple(&mut self, bytes: &Vec<u8>) -> Result<(), Box<dyn Error>> {
        for byte in bytes {
            self.contents.push(*byte);
        }
        Ok(())
    }

    pub fn write_boolean(&mut self, b: bool) -> Result<(), Box<dyn Error>> {
        self.contents.push(b as u8);
        Ok(())
    }

    pub fn write_int16(&mut self, i: i16) -> Result<(), Box<dyn Error>> {
        for b in i16::to_le_bytes(i).iter() {
            self.contents.push(*b);
        }
        Ok(())
    }

    pub fn write_uint16(&mut self, i: u16) -> Result<(), Box<dyn Error>> {
        for b in u16::to_le_bytes(i).iter() {
            self.contents.push(*b);
        }
        Ok(())
    }

    pub fn write_int32(&mut self, i: i32) -> Result<(), Box<dyn Error>> {
        for b in i32::to_le_bytes(i).iter() {
            self.contents.push(*b);
        }
        Ok(())
    }

    pub fn write_uint32(&mut self, i: u32) -> Result<(), Box<dyn Error>> {
        for b in u32::to_le_bytes(i).iter() {
            self.contents.push(*b);
        }
        Ok(())
    }

    pub fn write_float(&mut self, f: f32) -> Result<(), Box<dyn Error>> {
        for b in f32::to_le_bytes(f).iter() {
            self.contents.push(*b);
        }
        Ok(())
    }

    pub fn write_double(&mut self, d: f64) -> Result<(), Box<dyn Error>> {
        for b in f64::to_le_bytes(d).iter() {
            self.contents.push(*b);
        }
        Ok(())
    }

    pub fn write_kos_string(&mut self, s: &str) -> Result<(), Box<dyn Error>> {
        self.contents.push(s.len() as u8);
        for b in s.bytes() {
            self.contents.push(b);
        }
        Ok(())
    }

    pub fn write_string(&mut self, s: &str) -> Result<(), Box<dyn Error>> {
        for b in s.bytes() {
            self.contents.push(b);
        }
        self.contents.push(0);
        Ok(())
    }

}