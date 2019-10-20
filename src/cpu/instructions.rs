use crate::cpu::{
    operands::{Source, Target, WordRegister},
    Flags, ReadImmediate, CPU,
};
use std::fmt::{Display, Formatter};

pub enum Condition {
    Unconditional,
    Zero(bool),
    Carry(bool),
}

impl Condition {
    fn is_satisfied(&self, cpu: &CPU) -> bool {
        match self {
            Condition::Unconditional => true,
            Condition::Zero(flag) => cpu.reg.z_flag() == *flag,
            Condition::Carry(flag) => cpu.reg.c_flag() == *flag,
        }
    }
}

impl Display for Condition {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let string = match self {
            Condition::Unconditional => "",
            Condition::Zero(flag) => {
                if *flag {
                    "Z"
                } else {
                    "NZ"
                }
            }
            Condition::Carry(flag) => {
                if *flag {
                    "C"
                } else {
                    "NC"
                }
            }
        };
        write!(f, "{}", string)
    }
}

impl CPU {
    /// NOP
    pub fn no_operation(&mut self) {
        self.curr_instr = "NOP".into();
    }

    /// JP
    pub fn jump(&mut self, word: impl Source<u16>, cond: Condition) {
        self.curr_instr = "JP".to_string() + &cond.to_string() + " " + &word.to_string();

        let address = word.read(self);

        if cond.is_satisfied(self) {
            self.cycles_until_done += 1;
            self.reg.pc = address;
        }
    }

    /// JR
    pub fn jump_relative(&mut self, cond: Condition) {
        self.curr_instr = "JR".to_string() + &cond.to_string() + " ";

        let immediate: u8 = self.immediate().0;
        let offset = immediate as i8;
        self.curr_instr += &format!("{}", offset);

        if cond.is_satisfied(self) {
            self.cycles_until_done += 1;
            self.reg.pc = (i32::from(self.reg.pc) + i32::from(offset)) as u16;
        }
    }

    /// XOR
    pub fn xor(&mut self, byte: impl Source<u8>) {
        self.curr_instr = "XOR ".to_string() + &byte.to_string();

        self.reg.a ^= byte.read(self);

        let flags = if self.reg.a == 0 {
            Flags::Z
        } else {
            Flags::empty()
        };
        self.reg.set_flags(flags);
    }

    /// CP
    pub fn compare(&mut self, byte: impl Source<u8>) {
        self.curr_instr = "CP ".to_string() + &byte.to_string();

        let data = byte.read(self);

        let mut flags = self.reg.flags();
        flags.set(Flags::Z, self.reg.a == data);
        flags.insert(Flags::N);
        flags.set(Flags::H, false); // FIXME: Wrong.
        flags.set(Flags::C, self.reg.a < data);
        self.reg.set_flags(flags);
    }

    /// LD
    pub fn load<T, U: Target<T>, V: Source<T>>(&mut self, target: U, source: V) {
        self.curr_instr = "LD ".to_string() + &target.to_string() + ", " + &source.to_string();

        let data = source.read(self);
        target.write(self, data);
    }

    /// DEC
    pub fn decrement_byte<T: Source<u8> + Target<u8>>(&mut self, data: T) {
        self.curr_instr = "DEC ".to_string() + &data.to_string();

        let result = data.read(self).wrapping_sub(1);
        data.write(self, result);

        let mut flags = self.reg.flags();
        flags.set(Flags::Z, result == 0);
        flags.insert(Flags::N);
        flags.set(Flags::H, (result & 0x0F) == 0x0F);
        self.reg.set_flags(flags);
    }

    /// DEC
    pub fn decrement_word<T: Source<u16> + Target<u16>>(&mut self, data: T) {
        self.curr_instr = "DEC ".to_string() + &data.to_string();

        let result = data.read(self).wrapping_sub(1);
        data.write(self, result);

        self.cycles_until_done += 1;
    }

    /// LDD
    pub fn load_and_decrement_hl<T>(&mut self, target: impl Target<T>, source: impl Source<T>) {
        let instr = "LDD ".to_string() + &target.to_string() + ", " + &source.to_string();

        self.load(target, source);
        self.decrement_word(WordRegister::HL);

        self.cycles_until_done -= 1;
        self.curr_instr = instr;
    }

    /// INC
    pub fn increment_byte<T: Source<u8> + Target<u8>>(&mut self, data: T) {
        self.curr_instr = "INC ".to_string() + &data.to_string();

        let result = data.read(self).wrapping_add(1);
        data.write(self, result);

        let mut flags = self.reg.flags();
        flags.set(Flags::Z, result == 0);
        flags.remove(Flags::N);
        flags.set(Flags::H, result.trailing_zeros() >= 4);
        self.reg.set_flags(flags);
    }

    /// INC
    pub fn increment_word<T: Source<u16> + Target<u16>>(&mut self, data: T) {
        self.curr_instr = "INC ".to_string() + &data.to_string();

        let result = data.read(self).wrapping_add(1);
        data.write(self, result);

        self.cycles_until_done += 1;
    }

    /// HALT
    // TODO: Finish implementation.
    pub fn halt(&self) {
        unimplemented!();
    }

    /// DI
    pub fn disable_interrupts(&mut self) {
        self.curr_instr = "DI".to_string();
        self.ime = false;
    }
}
