use crate::memory::Memory;

trait Read<T> {
    fn read(&self, cpu: &mut CPU) -> T;
    fn print(&self, cpu: &CPU) -> String;
}

trait Write<T> {
    fn write(&self, cpu: &mut CPU, data: T);
    fn print(&self, cpu: &CPU) -> String;
}

struct Immediate();

impl Read<u8> for Immediate {
    fn read(&self, cpu: &mut CPU) -> u8 {
        cpu.read_immediate_byte()
    }

    fn print(&self, cpu: &CPU) -> String {
        let byte = cpu.mem.read_byte(cpu.reg.pc);
        format!("{:#04X}", byte)
    }
}

impl Read<u16> for Immediate {
    fn read(&self, cpu: &mut CPU) -> u16 {
        cpu.read_immediate_word()
    }

    fn print(&self, cpu: &CPU) -> String {
        let word = cpu.mem.read_word(cpu.reg.pc);
        format!("{:#06X}", word)
    }
}

enum ByteRegister {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

impl ByteRegister {
    fn str(&self) -> String {
        match self {
            ByteRegister::A => "A",
            ByteRegister::B => "B",
            ByteRegister::C => "C",
            ByteRegister::D => "D",
            ByteRegister::E => "E",
            ByteRegister::H => "H",
            ByteRegister::L => "L",
        }
        .into()
    }
}

impl Read<u8> for ByteRegister {
    fn read(&self, cpu: &mut CPU) -> u8 {
        cpu.reg.byte_register(self)
    }

    fn print(&self, _: &CPU) -> String {
        self.str().into()
    }
}

impl Write<u8> for ByteRegister {
    fn write(&self, cpu: &mut CPU, data: u8) {
        cpu.reg.set_byte_register(self, data);
    }

    fn print(&self, _: &CPU) -> String {
        self.str().into()
    }
}

enum WordRegister {
    BC,
    DE,
    HL,
    SP,
}

impl WordRegister {
    fn str(&self) -> String {
        match self {
            WordRegister::BC => "BC",
            WordRegister::DE => "DE",
            WordRegister::HL => "HL",
            WordRegister::SP => "SP",
        }
        .into()
    }
}

impl Read<u16> for WordRegister {
    fn read(&self, cpu: &mut CPU) -> u16 {
        cpu.reg.word_register(self)
    }

    fn print(&self, _: &CPU) -> String {
        self.str().into()
    }
}

impl Write<u16> for WordRegister {
    fn write(&self, cpu: &mut CPU, data: u16) {
        cpu.reg.set_word_register(self, data);
    }

    fn print(&self, _: &CPU) -> String {
        self.str().into()
    }
}

enum Indirect {
    BC,
    DE,
    HL,
    SP,
    Immediate,
}

impl Read<u8> for Indirect {
    fn read(&self, cpu: &mut CPU) -> u8 {
        let address = match self {
            Indirect::BC => cpu.reg.word_register(&WordRegister::BC),
            Indirect::DE => cpu.reg.word_register(&WordRegister::DE),
            Indirect::HL => cpu.reg.word_register(&WordRegister::HL),
            Indirect::SP => cpu.reg.word_register(&WordRegister::SP),
            Indirect::Immediate => cpu.read_immediate_word(),
        };
        cpu.read_byte(address)
    }

    fn print(&self, cpu: &CPU) -> String {
        match self {
            Indirect::BC => "(BC)".into(),
            Indirect::DE => "(DE)".into(),
            Indirect::HL => "(HL)".into(),
            Indirect::SP => "(SP)".into(),
            Indirect::Immediate => {
                let word = cpu.mem.read_word(cpu.reg.pc);
                format!("({:04X})", word)
            }
        }
    }
}

impl Write<u8> for Indirect {
    fn write(&self, cpu: &mut CPU, data: u8) {
        let address = match self {
            Indirect::BC => cpu.reg.word_register(&WordRegister::BC),
            Indirect::DE => cpu.reg.word_register(&WordRegister::DE),
            Indirect::HL => cpu.reg.word_register(&WordRegister::HL),
            Indirect::SP => cpu.reg.word_register(&WordRegister::SP),
            Indirect::Immediate => cpu.read_immediate_word(),
        };
        cpu.write_byte(address, data);
    }

    fn print(&self, cpu: &CPU) -> String {
        match self {
            Indirect::BC => "(BC)".into(),
            Indirect::DE => "(DE)".into(),
            Indirect::HL => "(HL)".into(),
            Indirect::SP => "(SP)".into(),
            Indirect::Immediate => {
                let word = cpu.mem.read_word(cpu.reg.pc);
                format!("({:04X})", word)
            }
        }
    }
}

trait Decrement<T> {
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

trait Increment<T> {
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

enum Condition {
    Unconditional,
    Zero(bool),
    Carry(bool),
}

impl Condition {
    fn print(&self) -> String {
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
}

struct Registers {
    a: u8,
    f: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    sp: u16,
    pc: u16,
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

    fn set_bc(&mut self, value: u16) {
        let bytes = value.to_le_bytes();
        self.c = bytes[0];
        self.b = bytes[1];
    }

    fn bc(&self) -> u16 {
        u16::from_le_bytes([self.c, self.b])
    }

    fn set_de(&mut self, value: u16) {
        let bytes = value.to_le_bytes();
        self.e = bytes[0];
        self.d = bytes[1];
    }

    fn de(&self) -> u16 {
        u16::from_le_bytes([self.e, self.d])
    }

    fn set_hl(&mut self, value: u16) {
        let bytes = value.to_le_bytes();
        self.l = bytes[0];
        self.h = bytes[1];
    }

    fn hl(&self) -> u16 {
        u16::from_le_bytes([self.l, self.h])
    }

    fn z_flag(&self) -> bool {
        (self.f & 0x80) != 0
    }

    fn n_flag(&self) -> bool {
        (self.f & 0x40) != 0
    }

    fn h_flag(&self) -> bool {
        (self.f & 0x20) != 0
    }

    fn c_flag(&self) -> bool {
        (self.f & 0x10) != 0
    }

    fn byte_register(&self, reg: &ByteRegister) -> u8 {
        match reg {
            ByteRegister::A => self.a,
            ByteRegister::B => self.b,
            ByteRegister::C => self.c,
            ByteRegister::D => self.d,
            ByteRegister::E => self.e,
            ByteRegister::H => self.h,
            ByteRegister::L => self.l,
        }
    }

    fn set_byte_register(&mut self, reg: &ByteRegister, value: u8) {
        match reg {
            ByteRegister::A => self.a = value,
            ByteRegister::B => self.b = value,
            ByteRegister::C => self.c = value,
            ByteRegister::D => self.d = value,
            ByteRegister::E => self.e = value,
            ByteRegister::H => self.h = value,
            ByteRegister::L => self.l = value,
        }
    }

    fn word_register(&self, reg: &WordRegister) -> u16 {
        match reg {
            WordRegister::BC => self.bc(),
            WordRegister::DE => self.de(),
            WordRegister::HL => self.hl(),
            WordRegister::SP => self.sp,
        }
    }

    fn set_word_register(&mut self, reg: &WordRegister, value: u16) {
        match reg {
            WordRegister::BC => self.set_bc(value),
            WordRegister::DE => self.set_de(value),
            WordRegister::HL => self.set_hl(value),
            WordRegister::SP => self.sp = value,
        }
    }
}

pub struct CPU {
    reg: Registers,
    cycle: u64,
    mem: Memory,
    curr_instr: String,
    print_instructions: bool,
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

    pub fn set_print_instructions(&mut self, state: bool) {
        self.print_instructions = state;
    }

    /// NOP
    fn no_operation(&mut self) {
        self.curr_instr = "NOP".into();
    }

    /// JP
    fn jump(&mut self, word: impl Read<u16>, cond: Condition) {
        self.curr_instr = "JP".to_string() + &cond.print() + " " + &word.print(self);

        let shall_jump = match cond {
            Condition::Unconditional => true,
            Condition::Zero(flag) => self.reg.z_flag() == flag,
            Condition::Carry(flag) => self.reg.c_flag() == flag,
        };

        let address = word.read(self);

        if shall_jump {
            self.cycle += 1;
            self.reg.pc = address;
        }
    }

    /// JR
    // TODO: Finish implementation.
    fn jump_relative(&mut self, cond: Condition) {
        self.curr_instr = "JR".to_string() + &cond.print() + " ";

        let shall_jump = match cond {
            Condition::Unconditional => true,
            Condition::Zero(flag) => self.reg.z_flag() == flag,
            Condition::Carry(flag) => self.reg.c_flag() == flag,
        };

        let offset = self.read_immediate_byte() as i8;

        if shall_jump {
            self.reg.pc = (self.reg.pc as i32 + offset as i32) as u16;
        }

        self.curr_instr += &format!("{}", offset);
    }

    /// XOR
    // TODO: Finish implementation.
    fn xor(&mut self, byte: impl Read<u8>) {
        self.curr_instr = "XOR ".to_string() + &byte.print(self);

        self.reg.a ^= byte.read(self);

        if self.reg.a == 0 {
            self.reg.f = 0b1000_0000;
        } else {
            self.reg.f = 0;
        }
    }

    /// LD
    // TODO: Finish implementation.
    fn load<T>(&mut self, target: impl Write<T>, source: impl Read<T>) {
        self.curr_instr = "LD ".to_string() + &target.print(self) + ", " + &source.print(self);

        let data = source.read(self);
        target.write(self, data);
    }

    /// DEC
    // TODO: Finish implementation.
    fn decrement<T: Decrement<T>>(&mut self, data: impl Read<T> + Write<T>) {
        self.curr_instr = "DEC ".to_string() + &self::Write::print(&data, self);

        let result = data.read(self).decrement();
        data.write(self, result);
    }

    /// LDD
    // TODO: Finish implementation.
    fn load_and_decrement_hl<T: Decrement<T>>(
        &mut self,
        target: impl Write<T>,
        source: impl Read<T>,
    ) {
        let instr = "LDD ".to_string() + &target.print(self) + ", " + &source.print(self);
        self.load(target, source);
        self.decrement(WordRegister::HL);

        self.curr_instr = instr;
    }

    /// INC
    // TODO: Finish implementation.
    fn increment<T: Increment<T>>(&mut self, data: impl Read<T> + Write<T>) {
        self.curr_instr = "INC ".to_string() + &self::Write::print(&data, self);

        let result = data.read(self).increment();
        data.write(self, result);
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

    /// Fetch, decode and execute one instruction.
    pub fn execute(&mut self) -> Result<(), String> {
        use ByteRegister::*;
        use Condition::*;
        use WordRegister::*;

        // Empty the current instruction strings.
        self.curr_instr = Default::default();

        // Fetch.
        if self.print_instructions {
            print!("{:04X}: ", self.reg.pc);
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
            0x2B => self.decrement(HL),
            0x2C => self.increment(L),
            0x2D => self.decrement(L),
            0x2E => self.load(L, Immediate()),
            0x31 => self.load(SP, Immediate()),
            0x32 => self.load_and_decrement_hl(Indirect::HL, A),
            0x33 => self.increment(SP),
            0x34 => self.increment(Indirect::HL),
            0x35 => self.decrement(Indirect::HL),
            0x36 => self.load(B, Indirect::HL),
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
