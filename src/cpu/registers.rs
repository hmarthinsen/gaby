use crate::cpu::operands::{ByteRegister, WordRegister};
use bitflags::bitflags;

bitflags! {
    pub struct Flags: u8 {
        const Z = 0b1000_0000;
        const N = 0b0100_0000;
        const H = 0b0010_0000;
        const C = 0b0001_0000;
    }
}

pub struct Registers {
    pub a: u8,
    pub f: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub sp: u16,
    pub pc: u16,
}

impl Registers {
    pub fn new() -> Self {
        Self {
            a: 0x01,
            f: 0xB0,
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xD8,
            h: 0x01,
            l: 0x4D,
            sp: 0xFFFE,
            pc: 0x0100,
        }
    }

    pub fn set_bc(&mut self, value: u16) {
        let bytes = value.to_le_bytes();
        self.c = bytes[0];
        self.b = bytes[1];
    }

    pub fn bc(&self) -> u16 {
        u16::from_le_bytes([self.c, self.b])
    }

    pub fn set_de(&mut self, value: u16) {
        let bytes = value.to_le_bytes();
        self.e = bytes[0];
        self.d = bytes[1];
    }

    pub fn de(&self) -> u16 {
        u16::from_le_bytes([self.e, self.d])
    }

    pub fn set_hl(&mut self, value: u16) {
        let bytes = value.to_le_bytes();
        self.l = bytes[0];
        self.h = bytes[1];
    }

    pub fn hl(&self) -> u16 {
        u16::from_le_bytes([self.l, self.h])
    }

    pub fn flags(&self) -> Flags {
        Flags::from_bits_truncate(self.f)
    }

    pub fn set_flags(&mut self, flags: Flags) {
        self.f = flags.bits();
    }

    pub fn z_flag(&self) -> bool {
        self.flags().contains(Flags::Z)
    }

    pub fn n_flag(&self) -> bool {
        self.flags().contains(Flags::N)
    }

    pub fn h_flag(&self) -> bool {
        self.flags().contains(Flags::H)
    }

    pub fn c_flag(&self) -> bool {
        self.flags().contains(Flags::C)
    }

    pub fn byte_register(&self, reg: &ByteRegister) -> u8 {
        use ByteRegister::*;
        match reg {
            A => self.a,
            B => self.b,
            C => self.c,
            D => self.d,
            E => self.e,
            H => self.h,
            L => self.l,
        }
    }

    pub fn set_byte_register(&mut self, reg: &ByteRegister, value: u8) {
        use ByteRegister::*;
        match reg {
            A => self.a = value,
            B => self.b = value,
            C => self.c = value,
            D => self.d = value,
            E => self.e = value,
            H => self.h = value,
            L => self.l = value,
        }
    }

    pub fn word_register(&self, reg: &WordRegister) -> u16 {
        use WordRegister::*;
        match reg {
            BC => self.bc(),
            DE => self.de(),
            HL => self.hl(),
            SP => self.sp,
        }
    }

    pub fn set_word_register(&mut self, reg: &WordRegister, value: u16) {
        use WordRegister::*;
        match reg {
            BC => self.set_bc(value),
            DE => self.set_de(value),
            HL => self.set_hl(value),
            SP => self.sp = value,
        }
    }
}
