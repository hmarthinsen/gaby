use crate::memory::{IORegister, Memory};
use sdl2::audio::AudioQueue;
use std::{cell::RefCell, rc::Rc};

pub struct Audio {
    mem: Rc<RefCell<Memory>>,
    tick_disabled: bool,
    frequency_counter: u16,
    duty_waveform_index: usize,
    sample_buffer: [f32; 1024],
    sample_buffer_index: usize,
    current_sample: f32,
    decimation_timer: usize,
    frame_timer: usize,
    frame_counter: usize,
    volume_timer: usize,
    sweep_timer: usize,
}

impl Audio {
    pub fn tick(&mut self, audio_queue: &AudioQueue<f32>) -> Result<(), String> {
        // Just looking at sound 2 for now.
        let mut mem = self.mem.borrow_mut();
        if mem[IORegister::NR24] & 0b1000_0000 != 0 {
            self.tick_disabled = false;
        }

        let mut current_length = mem[IORegister::NR21] & 0b0011_1111;

        if self.frame_timer == 0 {
            self.frame_counter = (self.frame_counter + 1) % 8;

            if self.frame_counter % 2 == 0 {
                if current_length != 63 {
                    current_length += 1;
                    mem[IORegister::NR21] = (mem[IORegister::NR21] & 0b1100_0000) | current_length;
                }

                if self.frame_counter % 4 == 2 {
                    self.sweep_timer -= 1;
                }
            } else if self.frame_counter == 7 {
                self.volume_timer -= 1;
            }

            self.frame_timer = 2047;
        } else {
            self.frame_timer -= 1;
        }

        // TODO: Implement disabling if envelope goes out of range.
        if self.tick_disabled {
            return Ok(());
        }

        if self.frequency_counter != 0 {
            self.frequency_counter -= 1;
        } else {
            // Advance to next duty waveform sample.
            self.frequency_counter = 2048
                - u16::from_le_bytes([mem[IORegister::NR23], mem[IORegister::NR24] & 0b0000_0111]);

            self.duty_waveform_index = (self.duty_waveform_index + 1) % 8;

            if current_length != 63 {
                let duty_id = (mem[IORegister::NR21] & 0b1100_0000) >> 6;
                let duty_pattern: u8 = match duty_id {
                    0 => 0b0000_0001,
                    1 => 0b1000_0001,
                    2 => 0b1000_0111,
                    3 => 0b0111_1110,
                    _ => panic!("This should never happen!"),
                };
                self.current_sample =
                    f32::from((duty_pattern >> self.duty_waveform_index) & 0b0000_0001);
            } else {
                self.current_sample = 0.0;
            }
        }

        if self.decimation_timer == 0 {
            self.sample_buffer[self.sample_buffer_index] = self.current_sample;

            if self.sample_buffer_index == 1023 {
                audio_queue.queue(&self.sample_buffer);
                self.sample_buffer_index = 0;
            } else {
                self.sample_buffer_index += 1;
            }

            self.decimation_timer = 15;
        } else {
            self.decimation_timer -= 1;
        }

        Ok(())
    }

    pub fn new(mem: Rc<RefCell<Memory>>) -> Self {
        Self {
            mem,
            tick_disabled: false,
            frequency_counter: 0,
            duty_waveform_index: 0,
            sample_buffer: [0.0; 1024],
            sample_buffer_index: 0,
            current_sample: 0.0,
            decimation_timer: 0,
            frame_counter: 0,
            frame_timer: 0,
            sweep_timer: 0,
            volume_timer: 0,
        }
    }
}
