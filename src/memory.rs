use rand;
use rand::Rng;
use std::error::Error;
use std::fs::File;
use std::io::Read;

pub struct Memory {
    data: [u8; 0x10000],
}

impl Memory {
    /// Initialize memory with random data.
    pub fn new() -> Self {
        let mut data = [0u8; 0x10000];
        rand::thread_rng().fill(&mut data[..]);
        Self { data }
    }

    pub fn load_rom(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let mut file = File::open(path)?;
        file.read_exact(&mut self.data[..0x8000])?;

        Ok(())
    }

    pub fn read_game_title(&self) -> String {
        let mut title = String::new();
        let bytes = &self.data[0x0134..=0x0142];
        for byte in bytes {
            if *byte != 0 {
                title.push(char::from(*byte));
            }
        }

        title.trim().into()
    }

    pub fn read_cartridge_type(&self) -> u8 {
        self.read_byte(0x0147)
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        self.data[address as usize]
    }

    pub fn read_word(&self, address: u16) -> u16 {
        u16::from_le_bytes([self.read_byte(address), self.read_byte(address + 1)])
    }

    pub fn write_byte(&mut self, address: u16, data: u8) {
        self.data[address as usize] = data;
    }

    pub fn write_word(&mut self, address: u16, data: u16) {
        let bytes = data.to_le_bytes();
        self.write_byte(address, bytes[0]);
        self.write_byte(address + 1, bytes[1]);
    }
}
