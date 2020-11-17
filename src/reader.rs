use std::{error::Error};

pub struct KOFileReader {
    current_index: usize,
    contents: Vec<u8>
}

impl KOFileReader {

    pub fn new(raw_contents: Vec<u8>) -> Result<KOFileReader, Box<dyn Error>> {

        // Return a new instance with the current index at 0
        Ok(KOFileReader {
            current_index: 0,
            contents: raw_contents
        })
    }

    /// Returns the current index of the reader into the byte vector
    pub fn get_current_index(&self) -> usize {
        self.current_index
    }

    pub fn eof(&self) -> bool {
        self.current_index >= (self.contents.len() - 1)
    }

    /// Simply discards the next byte from the contents vector, and advances the current index
    pub fn pop(&mut self, bytes: usize) -> Result<(), Box<dyn Error>> {
        self.current_index += bytes;

        if self.current_index <= self.contents.len() {
            Ok(())
        } else {
            Err("Unexpected EOF reached".into())
        }
    }

    /// Reads the next byte from the contents vector and returns it if there is one
    pub fn next(&mut self) -> Result<u8, Box<dyn Error>> {
        // Increment the index
        self.current_index += 1;

        // Return the next byte or throw an error
        match self.contents.get(self.current_index - 1) {
            Some(byte) => Ok(*byte),
            None => Err("Unexpected EOF reached".into()),
        }
    }

    /// Reads count bytes from the contents and returns a vector of them
    pub fn next_count(&mut self, count: usize) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut read_bytes: Vec<u8> = Vec::with_capacity(count);

        for _ in 0..count {
            read_bytes.push(self.next()?);
        }

        Ok(read_bytes)
    }

    /// Peeks one byte from the contents
    pub fn peek(&self) -> Result<u8, Box<dyn Error>> {
        // Return the next byte or throw an error
        match self.contents.get(self.current_index) {
            Some(byte) => Ok(*byte),
            None => Err("Unexpected EOF reached".into()),
        }
    }

    /// Peeks count bytes from the contents and returns a vector of them
    pub fn peek_count(&mut self, count: usize) -> Result<Vec<u8>, Box<dyn Error>> {
        let original_index = self.current_index;
        let mut peeked: Vec<u8> = Vec::with_capacity(count);

        for _ in 0..count {
            peeked.push(self.next()?);
        }

        self.current_index = original_index;

        Ok(peeked)
    }

    pub fn read_bytes_into_u32(&mut self, bytes: u8) -> Result<u32, Box<dyn Error>> {
        Ok(match bytes {
            0 => panic!("One should never try to read 0 bytes."),
            1 => self.next()? as u32,
            2 => self.read_int16()? as u32,
            3 => (self.read_int16()? as u32) + (self.next()? as u32) * 0x1_00_00u32,
            4 => self.read_int32()? as u32,
            _ => {
                return Err(
                    "Currently reading more than 4 bytes at a time into an address is unsupported"
                        .into(),
                )
            }
        })
    }

    pub fn read_boolean(&mut self) -> Result<bool, Box<dyn Error>> {
        Ok(self.next()? != 0u8)
    }

    pub fn read_byte(&mut self) -> Result<i8, Box<dyn Error>> {
        Ok(self.next()? as i8)
    }

    pub fn read_int16(&mut self) -> Result<i16, Box<dyn Error>> {
        let mut arr: [u8; 2] = [0u8; 2];

        for i in 0..2 {
            arr[i] = self.next()?;
        }

        Ok(i16::from_le_bytes(arr))
    }

    pub fn read_int32(&mut self) -> Result<i32, Box<dyn Error>> {
        let mut arr: [u8; 4] = [0u8; 4];

        for i in 0..4 {
            arr[i] = self.next()?;
        }

        Ok(i32::from_le_bytes(arr))
    }

    pub fn read_uint16(&mut self) -> Result<u16, Box<dyn Error>> {
        let mut arr: [u8; 2] = [0u8; 2];

        for i in 0..2 {
            arr[i] = self.next()?;
        }

        Ok(u16::from_le_bytes(arr))
    }

    pub fn read_uint32(&mut self) -> Result<u32, Box<dyn Error>> {
        let mut arr: [u8; 4] = [0u8; 4];

        for i in 0..4 {
            arr[i] = self.next()?;
        }

        Ok(u32::from_le_bytes(arr))
    }

    pub fn read_float(&mut self) -> Result<f32, Box<dyn Error>> {
        let mut arr: [u8; 4] = [0u8; 4];

        for i in 0..4 {
            arr[i] = self.next()?;
        }

        Ok(f32::from_le_bytes(arr))
    }

    pub fn read_double(&mut self) -> Result<f64, Box<dyn Error>> {
        let mut arr: [u8; 8] = [0u8; 8];

        for i in 0..8 {
            arr[i] = self.next()?;
        }

        Ok(f64::from_le_bytes(arr))
    }

    pub fn read_kos_string(&mut self) -> Result<String, Box<dyn Error>> {
        let len = self.next()? as usize;

        let mut internal = String::with_capacity(len);

        for _ in 0..len {
            internal.push(self.next()? as char);
        }

        Ok(internal)
    }

    pub fn read_string(&mut self) -> Result<String, Box<dyn Error>> {
        let mut internal = String::new();

        loop {

            let c = self.next()? as char;

            if c == '\0' {
                break;
            }

            internal.push(c);
        
        }

        Ok(internal)
    }

}