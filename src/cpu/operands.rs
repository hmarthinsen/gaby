use crate::cpu::{ReadMem, WriteMem, CPU};
use std::fmt::{Display, Formatter, UpperHex};

pub trait Source<T>: ToString {
    fn read(&self, cpu: &mut CPU) -> T;
}

pub trait Target<T>: ToString {
    fn write(&self, cpu: &mut CPU, data: T);
}

pub struct Immediate<T: UpperHex>(pub T);

impl<T: UpperHex> Display for Immediate<T> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{:#X}", self.0)
    }
}

impl<T: Copy + UpperHex> Source<T> for Immediate<T> {
    fn read(&self, _cpu: &mut CPU) -> T {
        self.0
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

impl Display for ByteRegister {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        use ByteRegister::*;
        let string = match self {
            A => "A",
            B => "B",
            C => "C",
            D => "D",
            E => "E",
            H => "H",
            L => "L",
        };
        write!(f, "{}", string)
    }
}

impl Source<u8> for ByteRegister {
    fn read(&self, cpu: &mut CPU) -> u8 {
        cpu.reg.byte_register(self)
    }
}

impl Target<u8> for ByteRegister {
    fn write(&self, cpu: &mut CPU, data: u8) {
        cpu.reg.set_byte_register(self, data);
    }
}

pub enum WordRegister {
    BC,
    DE,
    HL,
    SP,
}

impl Display for WordRegister {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        use WordRegister::*;
        let string = match self {
            BC => "BC",
            DE => "DE",
            HL => "HL",
            SP => "SP",
        };
        write!(f, "{}", string)
    }
}

impl Source<u16> for WordRegister {
    fn read(&self, cpu: &mut CPU) -> u16 {
        cpu.reg.word_register(self)
    }
}

impl Target<u16> for WordRegister {
    fn write(&self, cpu: &mut CPU, data: u16) {
        cpu.reg.set_word_register(self, data);
    }
}

pub enum Indirect {
    BC,
    DE,
    HL,
    HighC, // (0xFF00 + C)
}

impl Indirect {
    fn address(&self, cpu: &mut CPU) -> u16 {
        use Indirect::*;
        match self {
            BC => cpu.reg.word_register(&WordRegister::BC),
            DE => cpu.reg.word_register(&WordRegister::DE),
            HL => cpu.reg.word_register(&WordRegister::HL),
            HighC => 0xFF00 + u16::from(cpu.reg.byte_register(&ByteRegister::C)),
        }
    }
}

impl Display for Indirect {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        use Indirect::*;
        let string = match self {
            BC => "(BC)",
            DE => "(DE)",
            HL => "(HL)",
            HighC => "(0xFF00 + C)",
        };
        write!(f, "{}", string)
    }
}

impl<T> Source<T> for Indirect
where
    CPU: ReadMem<T>,
{
    fn read(&self, cpu: &mut CPU) -> T {
        let address = self.address(cpu);
        cpu.read(address)
    }
}

impl<T> Target<T> for Indirect
where
    CPU: WriteMem<T>,
{
    fn write(&self, cpu: &mut CPU, data: T) {
        let address = self.address(cpu);
        cpu.write(address, data);
    }
}

pub struct IndirectHighImmediate(pub u8);
pub struct IndirectImmediate(pub u16);

impl IndirectHighImmediate {
    fn address(&self) -> u16 {
        0xFF00 + u16::from(self.0)
    }
}

impl IndirectImmediate {
    fn address(&self) -> u16 {
        self.0
    }
}

impl Display for IndirectHighImmediate {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "(0xFF00 + {:#04X})", self.0)
    }
}

impl Display for IndirectImmediate {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "({:#06X})", self.0)
    }
}

impl<T> Source<T> for IndirectHighImmediate
where
    CPU: ReadMem<T>,
{
    fn read(&self, cpu: &mut CPU) -> T {
        let address = self.address();
        cpu.read(address)
    }
}

impl<T> Source<T> for IndirectImmediate
where
    CPU: ReadMem<T>,
{
    fn read(&self, cpu: &mut CPU) -> T {
        let address = self.address();
        cpu.read(address)
    }
}

impl<T> Target<T> for IndirectHighImmediate
where
    CPU: WriteMem<T>,
{
    fn write(&self, cpu: &mut CPU, data: T) {
        let address = self.address();
        cpu.write(address, data)
    }
}

impl<T> Target<T> for IndirectImmediate
where
    CPU: WriteMem<T>,
{
    fn write(&self, cpu: &mut CPU, data: T) {
        let address = self.address();
        cpu.write(address, data)
    }
}
