use crate::cpu::{
    operands::{Read, WordRegister, Write},
    Flags, CPU,
};

pub trait Decrement<T> {
    fn decrement(&mut self) -> T;
}

impl Decrement<u8> for u8 {
    fn decrement(&mut self) -> u8 {
        self.wrapping_sub(1)
    }
}

impl Decrement<u16> for u16 {
    fn decrement(&mut self) -> u16 {
        self.wrapping_sub(1)
    }
}

pub trait Increment<T> {
    fn increment(&mut self) -> T;
}

impl Increment<u8> for u8 {
    fn increment(&mut self) -> u8 {
        self.wrapping_add(1)
    }
}

impl Increment<u16> for u16 {
    fn increment(&mut self) -> u16 {
        self.wrapping_add(1)
    }
}

pub enum Condition {
    Unconditional,
    Zero(bool),
    Carry(bool),
}

impl Condition {
    fn to_string(&self) -> String {
        match self {
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
        }
        .into()
    }

    fn is_satisfied(&self, cpu: &CPU) -> bool {
        match self {
            Condition::Unconditional => true,
            Condition::Zero(flag) => cpu.reg.z_flag() == *flag,
            Condition::Carry(flag) => cpu.reg.c_flag() == *flag,
        }
    }
}

impl CPU {
    /// NOP
    pub fn no_operation(&mut self) {
        self.curr_instr = "NOP".into();
    }

    /// JP
    pub fn jump(&mut self, word: impl Read<u16>, cond: Condition) {
        self.curr_instr = "JP".to_string() + &cond.to_string() + " " + &word.to_string(self);

        let address = word.read(self);

        if cond.is_satisfied(self) {
            self.cycle += 1;
            self.reg.pc = address;
        }
    }

    /// JR
    pub fn jump_relative(&mut self, cond: Condition) {
        self.curr_instr = "JR".to_string() + &cond.to_string() + " ";

        let offset = self.read_immediate_byte() as i8;
        self.curr_instr += &format!("{}", offset);

        if cond.is_satisfied(self) {
            self.cycle += 1;
            self.reg.pc = (i32::from(self.reg.pc) + i32::from(offset)) as u16;
        }
    }

    /// XOR
    pub fn xor(&mut self, byte: impl Read<u8>) {
        self.curr_instr = "XOR ".to_string() + &byte.to_string(self);

        self.reg.a ^= byte.read(self);

        let flags = if self.reg.a == 0 {
            Flags::Z
        } else {
            Flags::empty()
        };
        self.reg.set_flags(flags);
    }

    /// LD
    pub fn load<T, U: Write<T>, V: Read<T>>(&mut self, target: U, source: V) {
        self.curr_instr =
            "LD ".to_string() + &target.to_string(self) + ", " + &source.to_string(self);

        let data = source.read(self);
        target.write(self, data);
    }

    /// DEC
    // TODO: Finish implementation.
    pub fn decrement<T: Decrement<T>, U: Read<T> + Write<T>>(&mut self, data: U) {
        self.curr_instr = "DEC ".to_string() + &Write::to_string(&data, self);

        let result = data.read(self).decrement();
        data.write(self, result);
    }

    /// LDD
    // TODO: Finish implementation.
    pub fn load_and_decrement_hl<T: Decrement<T>>(
        &mut self,
        target: impl Write<T>,
        source: impl Read<T>,
    ) {
        let instr = "LDD ".to_string() + &target.to_string(self) + ", " + &source.to_string(self);
        self.load(target, source);
        self.decrement(WordRegister::HL);

        self.curr_instr = instr;
    }

    /// INC
    // TODO: Finish implementation.
    pub fn increment<T: Increment<T>, U: Read<T> + Write<T>>(&mut self, data: U) {
        self.curr_instr = "INC ".to_string() + &Write::to_string(&data, self);

        let result = data.read(self).increment();
        data.write(self, result);
    }

    /// HALT
    // TODO: Finish implementation.
    pub fn halt(&self) {
        unimplemented!();
    }
}
