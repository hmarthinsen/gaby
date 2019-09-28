use crate::cpu::CPU;

pub trait Read<T> {
    fn read(&self, cpu: &mut CPU) -> T;
    fn to_string(&self, cpu: &CPU) -> String;
}

pub trait Write<T> {
    fn write(&self, cpu: &mut CPU, data: T);
    fn to_string(&self, cpu: &CPU) -> String;
}

pub struct Immediate();

impl Read<u8> for Immediate {
    fn read(&self, cpu: &mut CPU) -> u8 {
        cpu.read_immediate_byte()
    }

    fn to_string(&self, cpu: &CPU) -> String {
        let byte = cpu.mem.read_byte(cpu.reg.pc);
        format!("{:#04X}", byte)
    }
}

impl Read<u16> for Immediate {
    fn read(&self, cpu: &mut CPU) -> u16 {
        cpu.read_immediate_word()
    }

    fn to_string(&self, cpu: &CPU) -> String {
        let word = cpu.mem.read_word(cpu.reg.pc);
        format!("{:#06X}", word)
    }
}

pub enum ByteRegister {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

impl ByteRegister {
    fn to_string(&self) -> String {
        use ByteRegister::*;
        match self {
            A => "A",
            B => "B",
            C => "C",
            D => "D",
            E => "E",
            H => "H",
            L => "L",
        }
        .into()
    }
}

impl Read<u8> for ByteRegister {
    fn read(&self, cpu: &mut CPU) -> u8 {
        cpu.reg.byte_register(self)
    }

    fn to_string(&self, _: &CPU) -> String {
        self.to_string()
    }
}

impl Write<u8> for ByteRegister {
    fn write(&self, cpu: &mut CPU, data: u8) {
        cpu.reg.set_byte_register(self, data);
    }

    fn to_string(&self, _: &CPU) -> String {
        self.to_string()
    }
}

pub enum WordRegister {
    BC,
    DE,
    HL,
    SP,
}

impl WordRegister {
    fn to_string(&self) -> String {
        use WordRegister::*;
        match self {
            BC => "BC",
            DE => "DE",
            HL => "HL",
            SP => "SP",
        }
        .into()
    }
}

impl Read<u16> for WordRegister {
    fn read(&self, cpu: &mut CPU) -> u16 {
        cpu.reg.word_register(self)
    }

    fn to_string(&self, _: &CPU) -> String {
        self.to_string()
    }
}

impl Write<u16> for WordRegister {
    fn write(&self, cpu: &mut CPU, data: u16) {
        cpu.reg.set_word_register(self, data);
    }

    fn to_string(&self, _: &CPU) -> String {
        self.to_string()
    }
}

pub enum Indirect {
    BC,
    DE,
    HL,
    Immediate,
    HighImmediate, // (0xFF00 + immediate byte)
    HighC, // (0xFF00 + C)
}

impl Indirect {
    fn to_string(&self, cpu: &CPU) -> String {
        use Indirect::*;
        match self {
            BC => "(BC)".into(),
            DE => "(DE)".into(),
            HL => "(HL)".into(),
            Immediate => {
                let word = cpu.mem.read_word(cpu.reg.pc);
                format!("({:#06X})", word)
            },
            HighImmediate => {
                let byte = cpu.mem.read_byte(cpu.reg.pc);
                format!("(0xFF00 + {:#04X})", byte)
            },
            HighC => "(0xFF00 + C)".into(),
        }
    }

    fn address(&self, cpu: &mut CPU) -> u16 {
        use Indirect::*;
        match self {
            BC => cpu.reg.word_register(&WordRegister::BC),
            DE => cpu.reg.word_register(&WordRegister::DE),
            HL => cpu.reg.word_register(&WordRegister::HL),
            Immediate => cpu.read_immediate_word(),
            HighImmediate => 0xFF00 + u16::from(cpu.read_immediate_byte()),
            HighC => 0xFF00 + u16::from(cpu.reg.byte_register(&ByteRegister::C)),
        }
    }
}

impl Read<u8> for Indirect {
    fn read(&self, cpu: &mut CPU) -> u8 {
        let address = self.address(cpu);
        cpu.read_byte(address)
    }

    fn to_string(&self, cpu: &CPU) -> String {
        self.to_string(cpu)
    }
}

impl Read<u16> for Indirect {
    fn read(&self, cpu: &mut CPU) -> u16 {
        let address = self.address(cpu);
        cpu.read_word(address)
    }

    fn to_string(&self, cpu: &CPU) -> String {
        self.to_string(cpu)
    }
}

impl Write<u8> for Indirect {
    fn write(&self, cpu: &mut CPU, data: u8) {
        let address = self.address(cpu);
        cpu.write_byte(address, data);
    }

    fn to_string(&self, cpu: &CPU) -> String {
        self.to_string(cpu)
    }
}

impl Write<u16> for Indirect {
    fn write(&self, cpu: &mut CPU, data: u16) {
        let address = self.address(cpu);
        cpu.write_word(address, data);
    }

    fn to_string(&self, cpu: &CPU) -> String {
        self.to_string(cpu)
    }
}
