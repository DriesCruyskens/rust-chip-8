use std::fs::File;
use std::io::prelude::*;
use std::io::Error;
use std::path::Path;

pub struct Chip8 {
    pub pc: usize,
    pub v: [u8; 16],
    pub mem: [u8; 4096],
}

impl Chip8 {
    pub fn new() -> Self {
        Chip8 {
            pc: 0x200,
            v: [0; 16],
            mem: [0; 4096],
        }
    }

    pub fn execute_cycle(&mut self) {}

    pub fn load_program(&mut self, path: &Path) -> Result<(), Error> {
        let program = File::open(path)?;
        for (i, byte) in program.bytes().enumerate() {
            self.mem[0x200 + i] = byte?;
        }
        Ok(())
    }
}
