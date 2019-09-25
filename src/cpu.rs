mod instructions;
mod operands;
mod registers;

use crate::memory::Memory;
use instructions::*;
use operands::{ByteRegister, Immediate, Indirect, WordRegister};
use registers::{Flags, Registers};

pub struct CPU {
    reg: Registers,
    cycle: u64,
    mem: Memory,
    curr_instr: String,
    pub print_instructions: bool,
}

impl CPU {
    pub fn new(mem: Memory) -> Self {
        Self {
            reg: Registers::new(),
            cycle: 0,
            mem,
            curr_instr: Default::default(),
            print_instructions: false,
        }
    }

    fn read_immediate_byte(&mut self) -> u8 {
        self.cycle += 1;
        let data = self.mem.read_byte(self.reg.pc);
        self.reg.pc += 1;

        data
    }

    fn read_immediate_word(&mut self) -> u16 {
        self.cycle += 2;
        let data = self.mem.read_word(self.reg.pc);
        self.reg.pc += 2;

        data
    }

    fn read_byte(&mut self, address: u16) -> u8 {
        self.cycle += 1;
        self.mem.read_byte(address)
    }

    fn write_byte(&mut self, address: u16, data: u8) {
        self.cycle += 1;
        self.mem.write_byte(address, data);
    }

    fn read_word(&mut self, address: u16) -> u16 {
        self.cycle += 2;
        self.mem.read_word(address)
    }

    fn write_word(&mut self, address: u16, data: u16) {
        self.cycle += 2;
        self.mem.write_word(address, data);
    }

    /// Fetch, decode and execute one instruction.
    pub fn execute(&mut self) -> Result<(), String> {
        use ByteRegister::*;
        use Condition::*;
        use WordRegister::*;

        // Empty the current instruction strings.
        self.curr_instr = Default::default();

        // Fetch.
        if self.print_instructions {
            print!("{:11}, {:04X}: ", self.cycle, self.reg.pc);
        }
        let opcode = self.read_immediate_byte();

        // Decode and execute. Some instructions need cycle corrections.
        match opcode {
            0x00 => self.no_operation(),
            0x01 => self.load(BC, Immediate()),
            0x03 => self.increment(BC),
            0x04 => self.increment(B),
            0x05 => self.decrement(B),
            0x06 => self.load(B, Immediate()),
            0x0B => self.decrement(BC),
            0x0C => self.increment(C),
            0x0D => self.decrement(C),
            0x0E => self.load(C, Immediate()),
            0x11 => self.load(DE, Immediate()),
            0x13 => self.increment(DE),
            0x14 => self.increment(D),
            0x15 => self.decrement(D),
            0x16 => self.load(D, Immediate()),
            0x18 => self.jump_relative(Unconditional),
            0x1B => self.decrement(DE),
            0x1C => self.increment(E),
            0x1D => self.decrement(E),
            0x1E => self.load(E, Immediate()),
            0x20 => self.jump_relative(Zero(false)),
            0x21 => self.load(HL, Immediate()),
            0x23 => self.increment(HL),
            0x24 => self.increment(H),
            0x25 => self.decrement(H),
            0x26 => self.load(H, Immediate()),
            0x28 => self.jump_relative(Zero(true)),
            0x2B => self.decrement(HL),
            0x2C => self.increment(L),
            0x2D => self.decrement(L),
            0x2E => self.load(L, Immediate()),
            0x30 => self.jump_relative(Carry(false)),
            0x31 => self.load(SP, Immediate()),
            0x32 => self.load_and_decrement_hl(Indirect::HL, A),
            0x33 => self.increment(SP),
            0x34 => self.increment::<u8, _>(Indirect::HL),
            0x35 => self.decrement::<u8, _>(Indirect::HL),
            0x36 => self.load(B, Indirect::HL),
            0x38 => self.jump_relative(Carry(true)),
            0x3A => self.load_and_decrement_hl(A, Indirect::HL),
            0x3B => self.decrement(SP),
            0x3C => self.increment(A),
            0x3D => self.decrement(A),
            0x3E => self.load(A, Immediate()),
            // //0x40..=0x7F => unimplemented!(), // TODO: LD
            0xA8 => self.xor(B),
            0xA9 => self.xor(C),
            0xAA => self.xor(D),
            0xAB => self.xor(E),
            0xAC => self.xor(H),
            0xAD => self.xor(L),
            0xAE => self.xor(Indirect::HL),
            0xAF => self.xor(A),
            0xC2 => self.jump(Immediate(), Zero(false)),
            0xC3 => self.jump(Immediate(), Unconditional),
            0xCA => self.jump(Immediate(), Zero(true)),
            // //0xCB => unimplemented!(), // TODO: Go to CB table.
            0xD2 => self.jump(Immediate(), Carry(false)),
            0xDA => self.jump(Immediate(), Carry(true)),
            0xE9 => {
                self.jump(HL, Unconditional);
                self.cycle -= 1;
            }
            0xEE => self.xor(Immediate()),

            _ => return Err(format!["Unimplemented opcode {:#04X}", opcode]),
        }

        if self.print_instructions {
            println!("{}", self.curr_instr);
        }

        Ok(())
    }
}
