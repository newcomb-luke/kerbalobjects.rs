use std::{error::Error};

use crate::{KOFileReader, KOFileWriter};

pub struct RelInstruction {
    opcode: u8,
    operands: Vec<u32>
}

impl RelInstruction {

    pub fn new(opcode: u8, operands: Vec<u32>) -> RelInstruction {
        RelInstruction {
            opcode,
            operands
        }
    }

    pub fn read(reader: &mut KOFileReader) -> Result<RelInstruction, Box<dyn Error>> {

        let opcode = reader.next()?;

        let num_operands = RelInstruction::get_num_operands(opcode)?;

        let mut operands = Vec::with_capacity(num_operands);

        for _ in 0..num_operands {
            operands.push( reader.read_uint32()? );
        }

        Ok(RelInstruction::new(opcode, operands))

    }

    pub fn write(&self, writer: &mut KOFileWriter) -> Result<(), Box<dyn Error>> {

        writer.write(self.opcode)?;

        for operand in self.operands.iter() {
            writer.write_uint32(*operand)?;
        }

        Ok(())
    }
    
    pub fn size(&self) -> u32 {
        1 + self.operands.len() as u32 * 4
    }

    pub fn get_num_operands(opcode: u8) -> Result<usize, Box<dyn Error>> {
        Ok(match opcode {
            0x31 => 0,
            0x32 => 0,
            0x33 => 0,
            0x34 => 1,
            0x35 => 0,
            0x36 => 1,
            0x37 => 1,
            0x38 => 0,
            0x39 => 0,
            0x3a => 1,
            0x3b => 1,
            0x3c => 0,
            0x3d => 0,
            0x3e => 0,
            0x3f => 0,
            0x40 => 0,
            0x41 => 0,
            0x42 => 0,
            0x43 => 0,
            0x44 => 0,
            0x45 => 0,
            0x46 => 0,
            0x47 => 0,
            0x48 => 0,
            0x49 => 0,
            0x4a => 0,
            0x4b => 0,
            0x4c => 2,
            0x4d => 1,
            0x4e => 1,
            0x4f => 0,
            0x50 => 0,
            0x51 => 0,
            0x52 => 0,
            0x53 => 2,
            0x54 => 0,
            0x55 => 0,
            0x56 => 0,
            0x57 => 1,
            0x58 => 1,
            0x59 => 1,
            0x5a => 2,
            0x5b => 1,
            0x5c => 1,
            0x5d => 2,
            0x5e => 1,
            0x5f => 0,
            0x60 => 0,
            0x61 => 0,
            0x62 => 0,
    
            0xce => 1,
            0xcd => 2,
            0xf0 => 1,
            _ => return Err(format!("Invalid opcode found: {}, cannot determine the number of operands.", opcode).into()),
        })
    }

    pub fn get_opcode(&self) -> u8 {
        self.opcode
    }

    pub fn get_operands(&self) -> &Vec<u32> {
        &self.operands
    }

}