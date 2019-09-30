use rand;
use rand::Rng;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::ops::{Index, IndexMut};

pub enum IORegister {
    P1 = 0xFF00,
    SB = 0xFF01,
    SC = 0xFF02,
    DIV = 0xFF04,
    TIMA = 0xFF05,
    TMA = 0xFF06,
    TAC = 0xFF07,
    IF = 0xFF0F,
    NR10 = 0xFF10,
    NR11 = 0xFF11,
    NR12 = 0xFF12,
    NR13 = 0xFF13,
    NR14 = 0xFF14,
    NR21 = 0xFF16,
    NR22 = 0xFF17,
    NR23 = 0xFF18,
    NR24 = 0xFF19,
    NR30 = 0xFF1A,
    NR31 = 0xFF1B,
    NR32 = 0xFF1C,
    NR33 = 0xFF1D,
    NR34 = 0xFF1E,
    NR41 = 0xFF20,
    NR42 = 0xFF21,
    NR43 = 0xFF22,
    NR44 = 0xFF23,
    NR50 = 0xFF24,
    NR51 = 0xFF25,
    NR52 = 0xFF26,
    LCDC = 0xFF40,
    STAT = 0xFF41,
    SCY = 0xFF42,
    SCX = 0xFF43,
    LY = 0xFF44,
    LYC = 0xFF45,
    DMA = 0xFF46,
    BGP = 0xFF47,
    OBP0 = 0xFF48,
    OBP1 = 0xFF49,
    WY = 0xFF4A,
    WX = 0xFF4B,
    IE = 0xFFFF,
}

pub struct Memory {
    data: [u8; 0x10000],
}

impl Index<IORegister> for Memory {
    type Output = u8;

    fn index(&self, index: IORegister) -> &Self::Output {
        &self.data[index as usize]
    }
}

impl IndexMut<IORegister> for Memory {
    fn index_mut(&mut self, index: IORegister) -> &mut Self::Output {
        &mut self.data[index as usize]
    }
}

impl Index<u16> for Memory {
    type Output = u8;

    fn index(&self, index: u16) -> &Self::Output {
        &self.data[index as usize]
    }
}

impl IndexMut<u16> for Memory {
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        &mut self.data[index as usize]
    }
}

impl Memory {
    /// Initialize memory with random data.
    pub fn new() -> Self {
        let mut data = [0u8; 0x10000];
        rand::thread_rng().fill(&mut data[..]);

        let mut mem = Self { data };

        use IORegister::*;
        // FIXME: What about the other I/O registers?
        mem[TIMA] = 0x00;
        mem[TMA] = 0x00;
        mem[TAC] = 0x00;
        mem[NR10] = 0x80;
        mem[NR11] = 0xBF;
        mem[NR12] = 0xF3;
        mem[NR14] = 0xBF;
        mem[NR21] = 0x3F;
        mem[NR22] = 0x00;
        mem[NR24] = 0xBF;
        mem[NR30] = 0x7F;
        mem[NR31] = 0xFF;
        mem[NR32] = 0x9F;
        mem[NR33] = 0xBF; // FIXME: Should this be NR34?
        mem[NR41] = 0xFF;
        mem[NR42] = 0x00;
        mem[NR43] = 0x00;
        mem[NR44] = 0xBF;
        mem[NR50] = 0x77;
        mem[NR51] = 0xF3;
        mem[NR52] = 0xF1;
        mem[LCDC] = 0x91;
        mem[SCY] = 0x00;
        mem[SCX] = 0x00;
        mem[LYC] = 0x00;
        mem[BGP] = 0xFC;
        mem[OBP0] = 0xFF;
        mem[OBP1] = 0xFF;
        mem[WY] = 0x00;
        mem[WX] = 0x00;
        mem[IE] = 0x00;

        mem
    }

    pub fn load_rom(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let mut file = File::open(path)?;
        file.read_exact(&mut self.data[..0x8000])?;

        if self.read_cartridge_type() != 0 {
            return Err("Only supported cartridge type is ROM only.".into());
        }

        if self.read_rom_size() != 0 {
            return Err("Only 32 kB ROMs are supported.".into());
        }

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

    fn read_cartridge_type(&self) -> u8 {
        self[0x0147]
    }

    fn read_rom_size(&self) -> u8 {
        self[0x0148]
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        self[address]
    }

    pub fn read_word(&self, address: u16) -> u16 {
        u16::from_le_bytes([self.read_byte(address), self.read_byte(address + 1)])
    }

    pub fn write_byte(&mut self, address: u16, data: u8) {
        match address {
            0x0000..=0x7FFF => return, // Can't write to ROM area.
            0xC000..=0xDDFF => self[address + 0x2000] = data, // Write to echo area.
            0xE000..=0xFDFF => self[address - 0x2000] = data, // Write to echo area.
            0xFF00..=0xFFFF => self.write_io(address, data), // TODO: I/O registers.
            _ => {}
        }

        self[address] = data;
    }

    pub fn write_word(&mut self, address: u16, data: u16) {
        let bytes = data.to_le_bytes();
        self.write_byte(address, bytes[0]);
        self.write_byte(address + 1, bytes[1]);
    }

    fn write_io(&self, address: u16, data: u8) {
        unimplemented!(
            "I/O register write, address: {:04x}, data: {:04x}",
            address,
            data
        );
    }
}
