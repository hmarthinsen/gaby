mod instructions;
mod operands;
mod registers;

use crate::memory::{IORegister, Memory};
use instructions::*;
use operands::{
    ByteRegister, Immediate, Indirect, IndirectHighImmediate, IndirectImmediate, WordRegister,
};
use registers::{Flags, Registers};
use std::fmt::UpperHex;

pub trait ReadImmediate<T: UpperHex> {
    fn immediate(&mut self) -> Immediate<T>;
}

pub trait ReadMem<T> {
    fn read(&mut self, address: u16) -> T;
}

pub trait WriteMem<T> {
    fn write(&mut self, address: u16, data: T);
}

pub struct CPU {
    reg: Registers,
    ime: bool, // Interrupt Master Enable flag.
    cycle: u64,
    mem: Memory,
    curr_instr: String,
    pub print_instructions: bool,
}

impl ReadImmediate<u8> for CPU {
    fn immediate(&mut self) -> Immediate<u8> {
        self.cycle += 1;
        let data = self.mem.read_byte(self.reg.pc);
        self.reg.pc += 1;

        Immediate(data)
    }
}

impl ReadImmediate<u16> for CPU {
    fn immediate(&mut self) -> Immediate<u16> {
        self.cycle += 2;
        let data = self.mem.read_word(self.reg.pc);
        self.reg.pc += 2;

        Immediate(data)
    }
}

impl ReadMem<u8> for CPU {
    fn read(&mut self, address: u16) -> u8 {
        self.cycle += 1;
        self.mem.read_byte(address)
    }
}

impl ReadMem<u16> for CPU {
    fn read(&mut self, address: u16) -> u16 {
        self.cycle += 2;
        self.mem.read_word(address)
    }
}

impl WriteMem<u8> for CPU {
    fn write(&mut self, address: u16, data: u8) {
        self.cycle += 1;
        self.mem.write_byte(address, data);
    }
}

impl WriteMem<u16> for CPU {
    fn write(&mut self, address: u16, data: u16) {
        self.cycle += 2;
        self.mem.write_word(address, data);
    }
}

impl CPU {
    pub fn new(mem: Memory) -> Self {
        Self {
            reg: Registers::new(),
            ime: false,
            cycle: 0,
            mem,
            curr_instr: Default::default(),
            print_instructions: false,
        }
    }

    fn indirect_high_immediate(&mut self) -> IndirectHighImmediate {
        IndirectHighImmediate(self.immediate().0)
    }

    fn indirect_immediate(&mut self) -> IndirectImmediate {
        IndirectImmediate(self.immediate().0)
    }

    fn dispatch_interrupts(&mut self) {
        if self.ime {
            use IORegister::*;
            let interrupt_handler =
                if (self.mem[IF] & 0b0000_0001) & (self.mem[IE] & 0b0000_0001) != 0 {
                    // V-blank interrupt
                    self.mem[IF] &= 0b1111_1110;
                    Some(0x40)
                } else if (self.mem[IF] & 0b0000_0010) & (self.mem[IE] & 0b0000_0010) != 0 {
                    // LCDC status interrupt
                    self.mem[IF] &= 0b1111_1101;
                    Some(0x48)
                } else if (self.mem[IF] & 0b0000_0100) & (self.mem[IE] & 0b0000_0100) != 0 {
                    // Timer overflow interrupt
                    self.mem[IF] &= 0b1111_1011;
                    Some(0x50)
                } else if (self.mem[IF] & 0b0000_1000) & (self.mem[IE] & 0b0000_1000) != 0 {
                    // Serial transfer completion interrupt
                    self.mem[IF] &= 0b1111_0111;
                    Some(0x58)
                } else if (self.mem[IF] & 0b0001_0000) & (self.mem[IE] & 0b0001_0000) != 0 {
                    // Keypad high-to-low interrupt
                    self.mem[IF] &= 0b1110_1111;
                    Some(0x60)
                } else {
                    None
                };

            if let Some(address) = interrupt_handler {
                self.ime = false;

                self.reg.sp -= 2;
                self.mem.write_word(self.reg.sp, self.reg.pc);

                self.reg.pc = address;

                self.cycle += 5;
            }
        }
    }

    /// Fetch, decode and execute one instruction.
    pub fn execute(&mut self) -> Result<(), String> {
        use ByteRegister::*;
        use Condition::*;
        use WordRegister::*;

        self.dispatch_interrupts();

        // Fetch.
        if self.print_instructions {
            print!("{:11}, {:04X}: ", self.cycle, self.reg.pc);
        }
        let opcode: u8 = self.immediate().0;

        // Decode and execute. Some instructions need cycle corrections.
        match opcode {
            0x00 => self.no_operation(),
            0x01 => {
                let imm = self.immediate();
                self.load(BC, imm);
            }
            0x02 => self.load(Indirect::BC, A),
            0x03 => self.increment_word(BC),
            0x04 => self.increment_byte(B),
            0x05 => self.decrement_byte(B),
            0x06 => {
                let imm = self.immediate();
                self.load(B, imm);
            }
            0x08 => {
                let ind = self.indirect_immediate();
                self.load(ind, SP);
            }
            0x0A => self.load(A, Indirect::BC),
            0x0B => self.decrement_word(BC),
            0x0C => self.increment_byte(C),
            0x0D => self.decrement_byte(C),
            0x0E => {
                let imm = self.immediate();
                self.load(C, imm);
            }
            0x11 => {
                let imm = self.immediate();
                self.load(DE, imm);
            }
            0x12 => self.load(Indirect::DE, A),
            0x13 => self.increment_word(DE),
            0x14 => self.increment_byte(D),
            0x15 => self.decrement_byte(D),
            0x16 => {
                let imm = self.immediate();
                self.load(D, imm);
            }
            0x18 => self.jump_relative(Unconditional),
            0x1A => self.load(A, Indirect::DE),
            0x1B => self.decrement_word(DE),
            0x1C => self.increment_byte(E),
            0x1D => self.decrement_byte(E),
            0x1E => {
                let imm = self.immediate();
                self.load(E, imm);
            }
            0x20 => self.jump_relative(Zero(false)),
            0x21 => {
                let imm = self.immediate();
                self.load(HL, imm);
            }
            0x23 => self.increment_word(HL),
            0x24 => self.increment_byte(H),
            0x25 => self.decrement_byte(H),
            0x26 => {
                let imm = self.immediate();
                self.load(H, imm);
            }
            0x28 => self.jump_relative(Zero(true)),
            0x2B => self.decrement_word(HL),
            0x2C => self.increment_byte(L),
            0x2D => self.decrement_byte(L),
            0x2E => {
                let imm = self.immediate();
                self.load(L, imm);
            }
            0x30 => self.jump_relative(Carry(false)),
            0x31 => {
                let imm = self.immediate();
                self.load(SP, imm);
            }
            0x32 => self.load_and_decrement_hl(Indirect::HL, A),
            0x33 => self.increment_word(SP),
            0x34 => self.increment_byte(Indirect::HL),
            0x35 => self.decrement_byte(Indirect::HL),
            0x36 => self.load(B, Indirect::HL),
            0x38 => self.jump_relative(Carry(true)),
            0x3A => self.load_and_decrement_hl(A, Indirect::HL),
            0x3B => self.decrement_word(SP),
            0x3C => self.increment_byte(A),
            0x3D => self.decrement_byte(A),
            0x3E => {
                let imm = self.immediate();
                self.load(A, imm);
            }
            0x40..=0x7F => self.select_load_or_halt(opcode),
            0xA8 => self.xor(B),
            0xA9 => self.xor(C),
            0xAA => self.xor(D),
            0xAB => self.xor(E),
            0xAC => self.xor(H),
            0xAD => self.xor(L),
            0xAE => self.xor(Indirect::HL),
            0xAF => self.xor(A),
            0xC2 => {
                let imm = self.immediate();
                self.jump(imm, Zero(false));
            }
            0xC3 => {
                let imm = self.immediate();
                self.jump(imm, Unconditional);
            }
            0xCA => {
                let imm = self.immediate();
                self.jump(imm, Zero(true));
            }
            // //0xCB => unimplemented!(), // TODO: Go to CB table.
            0xD2 => {
                let imm = self.immediate();
                self.jump(imm, Carry(false));
            }
            0xDA => {
                let imm = self.immediate();
                self.jump(imm, Carry(true));
            }
            0xE0 => {
                let ind = self.indirect_high_immediate();
                self.load(ind, A);
            }
            0xE2 => self.load(Indirect::HighC, A),
            0xE9 => {
                self.jump(HL, Unconditional);
                self.cycle -= 1;
            }
            0xEA => {
                let ind = self.indirect_immediate();
                self.load(ind, A);
            }
            0xEE => {
                let imm = self.immediate();
                self.xor(imm);
            }
            0xF0 => {
                let ind = self.indirect_high_immediate();
                self.load(A, ind);
            }
            0xF2 => self.load(A, Indirect::HighC),
            0xF3 => self.disable_interrupts(),
            0xF9 => {
                self.load(SP, HL);
                self.cycle += 1;
            }
            0xFA => {
                let ind = self.indirect_immediate();
                self.load(A, ind);
            }

            _ => return Err(format!["Unimplemented opcode {:#04X}", opcode]),
        }

        if self.print_instructions {
            println!("[opcode {:02X}] {}", opcode, self.curr_instr);
        }

        Ok(())
    }

    /// Select target and source for load instruction based on opcode.
    fn select_load_or_halt(&mut self, opcode: u8) {
        let source_bits = opcode & 0b0000_0111;
        use ByteRegister::*;
        let source: Option<ByteRegister> = match source_bits {
            0x0 => Some(B),
            0x1 => Some(C),
            0x2 => Some(D),
            0x3 => Some(E),
            0x4 => Some(H),
            0x5 => Some(L),
            0x6 => None, // Signifies Indirect::HL
            0x7 => Some(A),
            _ => panic!("This should never happen."),
        };

        let target_bits = (opcode & 0b0011_1000) >> 3;
        let target: Option<ByteRegister> = match target_bits {
            0x0 => Some(B),
            0x1 => Some(C),
            0x2 => Some(D),
            0x3 => Some(E),
            0x4 => Some(H),
            0x5 => Some(L),
            0x6 => None, // Signifies Indirect::HL
            0x7 => Some(A),
            _ => panic!("This should never happen."),
        };

        match (target, source) {
            (Some(target_reg), Some(source_reg)) => self.load(target_reg, source_reg),
            (Some(target_reg), None) => self.load(target_reg, Indirect::HL),
            (None, Some(source_reg)) => self.load(Indirect::HL, source_reg),
            (None, None) => self.halt(),
        }
    }
}
