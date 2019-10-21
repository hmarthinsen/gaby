use crate::memory::{IORegister, Memory};
use std::cell::RefCell;
use std::rc::Rc;

pub struct Timer {
    mem: Rc<RefCell<Memory>>,
    div_counter: u32,
    timer_counter: u32,
}

impl Timer {
    const DIV_COUNTER_MAX: u32 = 64;

    pub fn new(mem: Rc<RefCell<Memory>>) -> Self {
        Self {
            mem,
            div_counter: 0,
            timer_counter: 0,
        }
    }

    pub fn tick(&mut self) -> Result<(), String> {
        let mut mem = self.mem.borrow_mut();

        if self.div_counter == 0 {
            mem[IORegister::DIV] = mem[IORegister::DIV].wrapping_add(1);

            self.div_counter = Timer::DIV_COUNTER_MAX;
        }
        self.div_counter -= 1;

        let timer_running = (mem[IORegister::TAC] & 0b0000_0100) != 0;
        if timer_running {
            if self.timer_counter == 0 {
                let (incremented, overflow) = mem[IORegister::TIMA].overflowing_add(1);
                mem[IORegister::TIMA] = if overflow {
                    mem[IORegister::IF] |= 0b0000_0100;
                    mem[IORegister::TMA]
                } else {
                    incremented
                };

                let clock_speed = mem[IORegister::TAC] & 0b0000_0011;
                self.timer_counter = match clock_speed {
                    0 => 256,
                    1 => 4,
                    2 => 16,
                    3 => 64,
                    _ => panic!("This should never happen!"),
                }
            }
            self.timer_counter -= 1;
        }
        Ok(())
    }
}
