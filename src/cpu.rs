mod instructions;
mod operands;
mod registers;

use crate::memory::{IORegister, Memory};
use instructions::*;
use operands::{
    ByteRegister, Immediate, Indirect, IndirectHighImmediate, IndirectImmediate, WordRegister,
};
use registers::{Flags, Registers};
use std::{cell::RefCell, fmt::UpperHex, rc::Rc};

pub trait ReadImmediate<T: UpperHex> {
    fn immediate(&mut self) -> Immediate<T>;
}

pub trait ReadMem<T> {
    fn read(&mut self, address: u16) -> T;
}

pub trait WriteMem<T> {
    fn write(&mut self, address: u16, data: T);
}

enum CPUMode {
    Halt,
    Run,
}

pub struct CPU {
    reg: Registers,
    ime: bool, // Interrupt Master Enable flag.
    mode: CPUMode,
    cycles_until_done: u32,
    mem: Rc<RefCell<Memory>>,
    curr_instr: String,
    pub print_instructions: bool,
}

impl ReadImmediate<u8> for CPU {
    fn immediate(&mut self) -> Immediate<u8> {
        self.cycles_until_done += 1;
        let data = self.mem.borrow().read_byte(self.reg.pc);
        self.reg.pc += 1;

        Immediate(data)
    }
}

impl ReadImmediate<u16> for CPU {
    fn immediate(&mut self) -> Immediate<u16> {
        self.cycles_until_done += 2;
        let data = self.mem.borrow().read_word(self.reg.pc);
        self.reg.pc += 2;

        Immediate(data)
    }
}

impl ReadMem<u8> for CPU {
    fn read(&mut self, address: u16) -> u8 {
        self.cycles_until_done += 1;
        self.mem.borrow().read_byte(address)
    }
}

impl ReadMem<u16> for CPU {
    fn read(&mut self, address: u16) -> u16 {
        self.cycles_until_done += 2;
        self.mem.borrow().read_word(address)
    }
}

impl WriteMem<u8> for CPU {
    fn write(&mut self, address: u16, data: u8) {
        self.cycles_until_done += 1;
        self.mem.borrow_mut().write_byte(address, data);
    }
}

impl WriteMem<u16> for CPU {
    fn write(&mut self, address: u16, data: u16) {
        self.cycles_until_done += 2;
        self.mem.borrow_mut().write_word(address, data);
    }
}

impl CPU {
    pub fn new(mem: Rc<RefCell<Memory>>) -> Self {
        Self {
            reg: Registers::new(),
            ime: false,
            mode: CPUMode::Run,
            cycles_until_done: 0,
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
        let cpu_is_halted = match self.mode {
            CPUMode::Halt => true,
            CPUMode::Run => false,
        };

        if self.ime {
            let mut mem = self.mem.borrow_mut();
            let interrupt_handler = if (mem[IORegister::IF] & 0b0000_0001)
                & (mem[IORegister::IE] & 0b0000_0001)
                != 0
            {
                // V-blank interrupt
                mem[IORegister::IF] &= 0b1111_1110;
                Some(0x40)
            } else if (mem[IORegister::IF] & 0b0000_0010) & (mem[IORegister::IE] & 0b0000_0010) != 0
            {
                // LCDC status interrupt
                mem[IORegister::IF] &= 0b1111_1101;
                Some(0x48)
            } else if (mem[IORegister::IF] & 0b0000_0100) & (mem[IORegister::IE] & 0b0000_0100) != 0
            {
                // Timer overflow interrupt
                mem[IORegister::IF] &= 0b1111_1011;
                Some(0x50)
            } else if (mem[IORegister::IF] & 0b0000_1000) & (mem[IORegister::IE] & 0b0000_1000) != 0
            {
                // Serial transfer completion interrupt
                mem[IORegister::IF] &= 0b1111_0111;
                Some(0x58)
            } else if (mem[IORegister::IF] & 0b0001_0000) & (mem[IORegister::IE] & 0b0001_0000) != 0
            {
                // Keypad high-to-low interrupt
                mem[IORegister::IF] &= 0b1110_1111;
                Some(0x60)
            } else {
                None
            };

            if let Some(address) = interrupt_handler {
                self.ime = false;

                self.reg.sp -= 2;
                mem.write_word(self.reg.sp, self.reg.pc);

                self.reg.pc = address;

                self.cycles_until_done += 5;

                if cpu_is_halted {
                    self.mode = CPUMode::Run;
                }
            }
        } else if cpu_is_halted {
            let mem = self.mem.borrow();
            if (mem[IORegister::IF] & mem[IORegister::IE] & 0b0001_1111) != 0 {
                // An interrupt occured in halt mode with IME = 0.
                // FIXME: HALT bug.
                self.mode = CPUMode::Run;
            }
        }
    }

    pub fn tick(&mut self) -> Result<(), String> {
        self.dispatch_interrupts();

        match self.mode {
            CPUMode::Run => {
                if self.cycles_until_done == 0 {
                    self.execute()?;
                }
                self.cycles_until_done -= 1;
            }
            CPUMode::Halt => {}
        }

        Ok(())
    }

    /// Fetch, decode and execute one instruction.
    fn execute(&mut self) -> Result<(), String> {
        use ByteRegister::*;
        use Condition::*;
        use WordRegister::*;

        // Fetch.
        if self.print_instructions {
            print!("{:04X}: ", self.reg.pc);
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
            0x07 => self.rotate_left(A),
            0x08 => {
                let ind = self.indirect_immediate();
                self.load(ind, SP);
            }
            0x09 => self.add_word(HL, BC),
            0x0A => self.load(A, Indirect::BC),
            0x0B => self.decrement_word(BC),
            0x0C => self.increment_byte(C),
            0x0D => self.decrement_byte(C),
            0x0E => {
                let imm = self.immediate();
                self.load(C, imm);
            }
            0x0F => self.rotate_right(A),
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
            0x17 => self.rotate_left_through_carry(A),
            0x18 => self.jump_relative(Unconditional),
            0x19 => self.add_word(HL, DE),
            0x1A => self.load(A, Indirect::DE),
            0x1B => self.decrement_word(DE),
            0x1C => self.increment_byte(E),
            0x1D => self.decrement_byte(E),
            0x1E => {
                let imm = self.immediate();
                self.load(E, imm);
            }
            0x1F => self.rotate_right_through_carry(A),
            0x20 => self.jump_relative(Zero(false)),
            0x21 => {
                let imm = self.immediate();
                self.load(HL, imm);
            }
            0x22 => self.load_and_increment_hl(Indirect::HL, A),
            0x23 => self.increment_word(HL),
            0x24 => self.increment_byte(H),
            0x25 => self.decrement_byte(H),
            0x26 => {
                let imm = self.immediate();
                self.load(H, imm);
            }
            0x28 => self.jump_relative(Zero(true)),
            0x29 => self.add_word(HL, HL),
            0x2A => self.load_and_increment_hl(A, Indirect::HL),
            0x2B => self.decrement_word(HL),
            0x2C => self.increment_byte(L),
            0x2D => self.decrement_byte(L),
            0x2E => {
                let imm = self.immediate();
                self.load(L, imm);
            }
            0x2F => self.complement_a(),
            0x30 => self.jump_relative(Carry(false)),
            0x31 => {
                let imm = self.immediate();
                self.load(SP, imm);
            }
            0x32 => self.load_and_decrement_hl(Indirect::HL, A),
            0x33 => self.increment_word(SP),
            0x34 => self.increment_byte(Indirect::HL),
            0x35 => self.decrement_byte(Indirect::HL),
            0x36 => {
                let imm: Immediate<u8> = self.immediate();
                self.load(Indirect::HL, imm);
            }
            0x38 => self.jump_relative(Carry(true)),
            0x39 => self.add_word(HL, SP),
            0x3A => self.load_and_decrement_hl(A, Indirect::HL),
            0x3B => self.decrement_word(SP),
            0x3C => self.increment_byte(A),
            0x3D => self.decrement_byte(A),
            0x3E => {
                let imm = self.immediate();
                self.load(A, imm);
            }
            0x40..=0x7F => self.select_load_or_halt(opcode),
            0x80 => self.add_byte(B),
            0x81 => self.add_byte(C),
            0x82 => self.add_byte(D),
            0x83 => self.add_byte(E),
            0x84 => self.add_byte(H),
            0x85 => self.add_byte(L),
            0x86 => self.add_byte(Indirect::HL),
            0x87 => self.add_byte(A),
            0x88 => self.add_with_carry(B),
            0x89 => self.add_with_carry(C),
            0x8A => self.add_with_carry(D),
            0x8B => self.add_with_carry(E),
            0x8C => self.add_with_carry(H),
            0x8D => self.add_with_carry(L),
            0x8E => self.add_with_carry(Indirect::HL),
            0x8F => self.add_with_carry(A),
            0xA0 => self.and(B),
            0xA1 => self.and(C),
            0xA2 => self.and(D),
            0xA3 => self.and(E),
            0xA4 => self.and(H),
            0xA5 => self.and(L),
            0xA6 => self.and(Indirect::HL),
            0xA7 => self.and(A),
            0xA8 => self.xor(B),
            0xA9 => self.xor(C),
            0xAA => self.xor(D),
            0xAB => self.xor(E),
            0xAC => self.xor(H),
            0xAD => self.xor(L),
            0xAE => self.xor(Indirect::HL),
            0xAF => self.xor(A),
            0xB0 => self.or(B),
            0xB1 => self.or(C),
            0xB2 => self.or(D),
            0xB3 => self.or(E),
            0xB4 => self.or(H),
            0xB5 => self.or(L),
            0xB6 => self.or(Indirect::HL),
            0xB7 => self.or(A),
            0xB8 => self.compare(B),
            0xB9 => self.compare(C),
            0xBA => self.compare(D),
            0xBB => self.compare(E),
            0xBC => self.compare(H),
            0xBD => self.compare(L),
            0xBE => self.compare(Indirect::HL),
            0xBF => self.compare(A),
            0xC0 => self.r#return(Zero(false)),
            0xC1 => self.pop(BC),
            0xC2 => {
                let imm = self.immediate();
                self.jump(imm, Zero(false));
            }
            0xC3 => {
                let imm = self.immediate();
                self.jump(imm, Unconditional);
            }
            0xC4 => {
                let imm = self.immediate();
                self.call(imm, Zero(false));
            }
            0xC5 => self.push(BC),
            0xC6 => {
                let imm = self.immediate();
                self.add_byte(imm);
            }
            0xC7 => self.restart(0x00),
            0xC8 => self.r#return(Zero(true)),
            0xC9 => self.r#return(Unconditional),
            0xCA => {
                let imm = self.immediate();
                self.jump(imm, Zero(true));
            }
            0xCB => self.execute_cb()?, // Go to CB table.
            0xCC => {
                let imm = self.immediate();
                self.call(imm, Zero(true));
            }
            0xCD => {
                let imm = self.immediate();
                self.call(imm, Unconditional);
            }
            0xCE => {
                let imm = self.immediate();
                self.add_with_carry(imm);
            }
            0xCF => self.restart(0x08),
            0xD0 => self.r#return(Carry(false)),
            0xD1 => self.pop(DE),
            0xD2 => {
                let imm = self.immediate();
                self.jump(imm, Carry(false));
            }
            0xD4 => {
                let imm = self.immediate();
                self.call(imm, Carry(false));
            }
            0xD5 => self.push(DE),
            0xD7 => self.restart(0x10),
            0xD8 => self.r#return(Carry(true)),
            0xD9 => self.return_and_enable_interrupts(),
            0xDA => {
                let imm = self.immediate();
                self.jump(imm, Carry(true));
            }
            0xDC => {
                let imm = self.immediate();
                self.call(imm, Carry(true));
            }
            0xDF => self.restart(0x18),
            0xE0 => {
                let ind = self.indirect_high_immediate();
                self.load(ind, A);
            }
            0xE1 => self.pop(HL),
            0xE2 => self.load(Indirect::HighC, A),
            0xE5 => self.push(HL),
            0xE6 => {
                let imm = self.immediate();
                self.and(imm);
            }
            0xE7 => self.restart(0x20),
            0xE9 => {
                self.jump(HL, Unconditional);
                self.cycles_until_done -= 1;
            }
            0xEA => {
                let ind = self.indirect_immediate();
                self.load(ind, A);
            }
            0xEE => {
                let imm = self.immediate();
                self.xor(imm);
            }
            0xEF => self.restart(0x28),
            0xF0 => {
                let ind = self.indirect_high_immediate();
                self.load(A, ind);
            }
            0xF1 => self.pop(AF),
            0xF2 => self.load(A, Indirect::HighC),
            0xF3 => self.disable_interrupts(),
            0xF5 => self.push(AF),
            0xF6 => {
                let imm = self.immediate();
                self.or(imm);
            }
            0xF7 => self.restart(0x30),
            0xF9 => {
                self.load(SP, HL);
                self.cycles_until_done += 1;
            }
            0xFA => {
                let ind = self.indirect_immediate();
                self.load(A, ind);
            }
            0xFB => self.enable_interrupts(),
            0xFE => {
                let imm = self.immediate();
                self.compare(imm);
            }
            0xFF => self.restart(0x38),

            _ => return Err(format!["Unimplemented opcode {:#04X}", opcode]),
        }

        if self.print_instructions && opcode != 0xCB {
            println!(
                "[opcode {:02X}, cycles: {}] {}",
                opcode, self.cycles_until_done, self.curr_instr
            );
        }

        Ok(())
    }

    fn execute_cb(&mut self) -> Result<(), String> {
        use ByteRegister::*;

        // Fetch.
        let opcode: u8 = self.immediate().0;

        // Decode and execute. Some instructions need cycle corrections.
        match opcode {
            0x00 => self.rotate_left(B),
            0x01 => self.rotate_left(C),
            0x02 => self.rotate_left(D),
            0x03 => self.rotate_left(E),
            0x04 => self.rotate_left(H),
            0x05 => self.rotate_left(L),
            0x06 => self.rotate_left(Indirect::HL),
            0x07 => self.rotate_left(A),
            0x08 => self.rotate_right(B),
            0x09 => self.rotate_right(C),
            0x0A => self.rotate_right(D),
            0x0B => self.rotate_right(E),
            0x0C => self.rotate_right(H),
            0x0D => self.rotate_right(L),
            0x0E => self.rotate_right(Indirect::HL),
            0x0F => self.rotate_right(A),
            0x10 => self.rotate_left_through_carry(B),
            0x11 => self.rotate_left_through_carry(C),
            0x12 => self.rotate_left_through_carry(D),
            0x13 => self.rotate_left_through_carry(E),
            0x14 => self.rotate_left_through_carry(H),
            0x15 => self.rotate_left_through_carry(L),
            0x16 => self.rotate_left_through_carry(Indirect::HL),
            0x17 => self.rotate_left_through_carry(A),
            0x18 => self.rotate_right_through_carry(B),
            0x19 => self.rotate_right_through_carry(C),
            0x1A => self.rotate_right_through_carry(D),
            0x1B => self.rotate_right_through_carry(E),
            0x1C => self.rotate_right_through_carry(H),
            0x1D => self.rotate_right_through_carry(L),
            0x1E => self.rotate_right_through_carry(Indirect::HL),
            0x1F => self.rotate_right_through_carry(A),
            0x20 => self.shift_left(B),
            0x21 => self.shift_left(C),
            0x22 => self.shift_left(D),
            0x23 => self.shift_left(E),
            0x24 => self.shift_left(H),
            0x25 => self.shift_left(L),
            0x26 => self.shift_left(Indirect::HL),
            0x27 => self.shift_left(A),
            0x28 => self.shift_right_keep_msb(B),
            0x29 => self.shift_right_keep_msb(C),
            0x2A => self.shift_right_keep_msb(D),
            0x2B => self.shift_right_keep_msb(E),
            0x2C => self.shift_right_keep_msb(H),
            0x2D => self.shift_right_keep_msb(L),
            0x2E => self.shift_right_keep_msb(Indirect::HL),
            0x2F => self.shift_right_keep_msb(A),
            0x30 => self.swap(B),
            0x31 => self.swap(C),
            0x32 => self.swap(D),
            0x33 => self.swap(E),
            0x34 => self.swap(H),
            0x35 => self.swap(L),
            0x36 => self.swap(Indirect::HL),
            0x37 => self.swap(A),
            0x38 => self.shift_right(B),
            0x39 => self.shift_right(C),
            0x3A => self.shift_right(D),
            0x3B => self.shift_right(E),
            0x3C => self.shift_right(H),
            0x3D => self.shift_right(L),
            0x3E => self.shift_right(Indirect::HL),
            0x3F => self.shift_right(A),
            0x40..=0x7F => self.select_test_bit(opcode),
            0x80..=0xBF => self.select_reset_bit(opcode),
            0xC0..=0xFF => self.select_set_bit(opcode),
        }

        if self.print_instructions {
            println!(
                "[opcode CB {:02X}, cycles: {}] {}",
                opcode, self.cycles_until_done, self.curr_instr
            );
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

    fn select_reset_bit(&mut self, opcode: u8) {
        let target_bits = opcode & 0b0000_0111;
        use ByteRegister::*;
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

        let target_bit = (opcode & 0b0011_1000) >> 3;

        match target {
            Some(target_reg) => self.reset_bit(target_bit, target_reg),
            None => self.reset_bit(target_bit, Indirect::HL),
        }
    }

    fn select_set_bit(&mut self, opcode: u8) {
        let target_bits = opcode & 0b0000_0111;
        use ByteRegister::*;
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

        let target_bit = (opcode & 0b0011_1000) >> 3;

        match target {
            Some(target_reg) => self.set_bit(target_bit, target_reg),
            None => self.set_bit(target_bit, Indirect::HL),
        }
    }

    fn select_test_bit(&mut self, opcode: u8) {
        let target_bits = opcode & 0b0000_0111;
        use ByteRegister::*;
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

        let target_bit = (opcode & 0b0011_1000) >> 3;

        match target {
            Some(target_reg) => self.test_bit(target_bit, target_reg),
            None => self.test_bit(target_bit, Indirect::HL),
        }
    }
}
