mod cpu;
mod memory;

use cpu::CPU;
use memory::Memory;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut mem = Memory::new();
    mem.load_rom("data/tetris.gb")?;

    println!("Title: {}", mem.read_game_title());

    if mem.read_cartridge_type() != 0 {
        return Err("Only supported cartridge type is ROM only.".into());
    }

    let mut cpu = CPU::new(mem);
    cpu.print_instructions = true;

    loop {
        cpu.execute()?;
    }
}
