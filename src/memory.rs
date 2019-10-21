use rand::Rng;
use std::{
    error::Error,
    fs::File,
    io::Read,
    ops::{Index, IndexMut},
};

pub struct IORegister;

impl IORegister {
    pub const P1: u16 = 0xFF00;
    // pub const SB: u16 = 0xFF01;
    pub const SC: u16 = 0xFF02;
    pub const DIV: u16 = 0xFF04;
    pub const TIMA: u16 = 0xFF05;
    pub const TMA: u16 = 0xFF06;
    pub const TAC: u16 = 0xFF07;
    pub const IF: u16 = 0xFF0F;
    pub const NR10: u16 = 0xFF10;
    pub const NR11: u16 = 0xFF11;
    pub const NR12: u16 = 0xFF12;
    // pub const NR13: u16 = 0xFF13;
    pub const NR14: u16 = 0xFF14;
    pub const NR21: u16 = 0xFF16;
    pub const NR22: u16 = 0xFF17;
    // pub const NR23: u16 = 0xFF18;
    pub const NR24: u16 = 0xFF19;
    pub const NR30: u16 = 0xFF1A;
    pub const NR31: u16 = 0xFF1B;
    pub const NR32: u16 = 0xFF1C;
    pub const NR33: u16 = 0xFF1D;
    // pub const NR34: u16 = 0xFF1E;
    pub const NR41: u16 = 0xFF20;
    pub const NR42: u16 = 0xFF21;
    pub const NR43: u16 = 0xFF22;
    pub const NR44: u16 = 0xFF23;
    pub const NR50: u16 = 0xFF24;
    pub const NR51: u16 = 0xFF25;
    pub const NR52: u16 = 0xFF26;
    pub const LCDC: u16 = 0xFF40;
    pub const STAT: u16 = 0xFF41;
    pub const SCY: u16 = 0xFF42;
    pub const SCX: u16 = 0xFF43;
    pub const LY: u16 = 0xFF44;
    pub const LYC: u16 = 0xFF45;
    pub const DMA: u16 = 0xFF46;
    pub const BGP: u16 = 0xFF47;
    pub const OBP0: u16 = 0xFF48;
    pub const OBP1: u16 = 0xFF49;
    pub const WY: u16 = 0xFF4A;
    pub const WX: u16 = 0xFF4B;
    pub const IE: u16 = 0xFFFF;
}

pub struct Memory {
    pub data: [u8; 0x10000],
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
    const OAM: u16 = 0xFE00;
    const OAM_SIZE: u16 = 160;

    /// Initialize memory with random data.
    pub fn new() -> Self {
        let mut data = [0u8; 0x10000];
        rand::thread_rng().fill(&mut data[..]);

        let mut mem = Self { data };

        // FIXME: What about the other I/O registers?
        mem[IORegister::P1] = 0x00;
        mem[IORegister::SC] = 0x00;
        mem[IORegister::TIMA] = 0x00;
        mem[IORegister::TMA] = 0x00;
        mem[IORegister::TAC] = 0x00;
        mem[IORegister::NR10] = 0x80;
        mem[IORegister::NR11] = 0xBF;
        mem[IORegister::NR12] = 0xF3;
        mem[IORegister::NR14] = 0xBF;
        mem[IORegister::NR21] = 0x3F;
        mem[IORegister::NR22] = 0x00;
        mem[IORegister::NR24] = 0xBF;
        mem[IORegister::NR30] = 0x7F;
        mem[IORegister::NR31] = 0xFF;
        mem[IORegister::NR32] = 0x9F;
        mem[IORegister::NR33] = 0xBF; // FIXME: Should this be NR34?
        mem[IORegister::NR41] = 0xFF;
        mem[IORegister::NR42] = 0x00;
        mem[IORegister::NR43] = 0x00;
        mem[IORegister::NR44] = 0xBF;
        mem[IORegister::NR50] = 0x77;
        mem[IORegister::NR51] = 0xF3;
        mem[IORegister::NR52] = 0xF1;
        mem[IORegister::LCDC] = 0x91; // FIXME: Manual says 0x83.
        mem[IORegister::SCY] = 0x00;
        mem[IORegister::SCX] = 0x00;
        mem[IORegister::LY] = 0x00; // FIXME: Correct?
        mem[IORegister::LYC] = 0x00;
        mem[IORegister::BGP] = 0xFC;
        mem[IORegister::OBP0] = 0xFF;
        mem[IORegister::OBP1] = 0xFF;
        mem[IORegister::WY] = 0x00;
        mem[IORegister::WX] = 0x00;
        mem[IORegister::IE] = 0x00;

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
        match address {
            IORegister::P1 => 0xFF, // No buttons pressed.
            _ => self[address],
        }
    }

    pub fn read_word(&self, address: u16) -> u16 {
        u16::from_le_bytes([self.read_byte(address), self.read_byte(address + 1)])
    }

    pub fn write_byte(&mut self, address: u16, data: u8) {
        match address {
            0x0000..=0x7FFF => return, // Can't write to ROM area.
            0xC000..=0xDDFF => self[address + 0x2000] = data, // Write to echo area.
            0xE000..=0xFDFF => self[address - 0x2000] = data, // Write to echo area.
            0xFF00..=0xFFFF => {
                self.write_io(address, data);
                return;
            }
            _ => {}
        }

        self[address] = data;
    }

    pub fn write_word(&mut self, address: u16, data: u16) {
        let bytes = data.to_le_bytes();
        self.write_byte(address, bytes[0]);
        self.write_byte(address + 1, bytes[1]);
    }

    fn write_io(&mut self, address: u16, data: u8) {
        match address {
            IORegister::DIV => self[IORegister::DIV] = 0,
            IORegister::DMA => self.dma_transfer(data),
            _ => self[address] = data,
        };
    }

    // Transfer 160 bytes to OAM memory.
    fn dma_transfer(&mut self, source_address: u8) {
        let address = u16::from(source_address) << 8;

        for offset in 0..Memory::OAM_SIZE {
            self[Memory::OAM + offset] = self[address + offset];
        }
    }
}
