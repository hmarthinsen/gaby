mod cpu;
mod memory;

use cpu::CPU;
use memory::Memory;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut mem = Memory::new();
    mem.load_rom("data/tetris.gb")?;

    let mut cpu = CPU::new(mem);

    loop {
        cpu.execute()?;
    }
}
