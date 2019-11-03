use crate::memory::{IORegister, Memory};
use rand::Rng;
use sdl2::audio::AudioQueue;
use std::{cell::RefCell, rc::Rc};

pub struct Audio {
    mem: Rc<RefCell<Memory>>,
    // tick_disabled: [bool; 4],
    output_enabled: [bool; 4],
    length_counters: [usize; 4],
    envelope_counters: [u8; 4],
    envelope_values: [u8; 4],
    frequency_timers: [u16; 4],
    waveform_positions: [usize; 4],
    sample_buffer: [f32; 1024],
    sample_buffer_index: usize,
    current_samples: [f32; 4],
    decimation_timer: usize,
    frame_timer: usize,
    frame_step: usize,
    volume_timer: usize,
    sweep_timer: usize,
}

impl Audio {
    // NR register address tables used when looping over all four channels:
    const NRX1: [u16; 4] = [
        IORegister::NR11,
        IORegister::NR21,
        IORegister::NR31,
        IORegister::NR41,
    ];
    const NRX2: [u16; 4] = [
        IORegister::NR12,
        IORegister::NR22,
        IORegister::NR32,
        IORegister::NR42,
    ];
    const NRX3: [u16; 4] = [
        IORegister::NR13,
        IORegister::NR23,
        IORegister::NR33,
        IORegister::NR43,
    ];
    const NRX4: [u16; 4] = [
        IORegister::NR14,
        IORegister::NR24,
        IORegister::NR34,
        IORegister::NR44,
    ];

    pub fn tick(&mut self, audio_queue: &AudioQueue<f32>) -> Result<(), String> {
        let mut mem = self.mem.borrow_mut();

        // Check if any of the channels are to be restarted.
        for i in 0..4 {
            let io_address = (Audio::NRX4[i] & 0x00FF) as usize;
            if mem.io_written_to[io_address] {
                mem.io_written_to[io_address] = false;

                if mem[Audio::NRX4[i]] & 0b1000_0000 != 0 {
                    self.output_enabled[i] = true;

                    if self.length_counters[i] == 0 {
                        self.length_counters[i] = 64;
                    }

                    self.frequency_timers[i] = 2048
                        - u16::from_le_bytes([
                            mem[Audio::NRX3[i]],
                            mem[Audio::NRX4[i]] & 0b0000_0111,
                        ]);
                    self.envelope_counters[i] = mem[Audio::NRX2[i]] & 0b0000_0111;
                    self.envelope_values[i] = mem[Audio::NRX2[i]] & 0b1111_0000;

                    // TODO: Set all noise channel LFSR bits to 1.
                    // TODO: Set wave channel position to 0.
                    // TODO: Channel 1 does several things:
                    // - Square 1's frequency is copied to the shadow register.
                    // - The sweep timer is reloaded.
                    // - The internal enabled flag is set if either the sweep period or shift are non-zero, cleared otherwise.
                    // - If the sweep shift is non-zero, frequency calculation and the overflow check are performed immediately.
                }
            }
        }

        // Load length counter if NRx1 was written to.
        for i in 0..4 {
            let io_address = (Audio::NRX1[i] & 0x00FF) as usize;
            if mem.io_written_to[io_address] {
                mem.io_written_to[io_address] = false;

                if i == 2 {
                    self.length_counters[i] = 256 - mem[Audio::NRX1[i]] as usize;
                } else {
                    self.length_counters[i] = 64 - (mem[Audio::NRX1[i]] & 0b0011_1111) as usize;
                }
            }
        }

        // 512 Hz frame sequencer for timing of lengths, volume envelopes and sweeps.
        if self.frame_timer == 0 {
            if self.frame_step % 2 == 0 {
                // Length counters
                for i in 0..4 {
                    if self.length_counters[i] == 0 {
                        self.output_enabled[i] = false;
                    } else {
                        if mem[Audio::NRX4[i]] & 0b0100_0000 != 0 {
                            self.length_counters[i] -= 1;
                        }
                    }
                }

                if self.frame_step % 4 == 2 {
                    // TODO: Sweeps
                    self.sweep_timer -= 1;
                }
            } else if self.frame_step == 7 {
                // TODO: Volume envelopes
                for &i in &[0, 1, 3] {
                    if self.envelope_counters[i] == 0 {
                        let step_length = mem[Audio::NRX2[i]] & 0b0000_0111;
                        if step_length != 0 {
                            if mem[Audio::NRX2[i]] & 0b0000_1000 == 0 {
                                if self.envelope_values[i] > 0 {
                                    self.envelope_values[i] -= 1;
                                }
                            } else {
                                if self.envelope_values[i] < 0xF {
                                    self.envelope_values[i] += 1;
                                }
                            }
                        }

                        self.envelope_counters[i] = step_length;
                    } else {
                        self.envelope_counters[i] -= 1;
                    }
                }
                self.volume_timer -= 1;

                self.frame_step = 0;
            }

            self.frame_timer = 2047;
        } else {
            self.frame_timer -= 1;
        }

        // TODO: Implement disabling if envelope goes out of range.
        // if self.tick_disabled {
        //     return Ok(());
        // }

        // Rectangle sounds
        for i in 0..2 {
            if self.frequency_timers[i] != 0 {
                self.frequency_timers[i] -= 1;
            } else {
                // Advance to next duty waveform sample.
                self.frequency_timers[i] = 2048
                    - u16::from_le_bytes([mem[Audio::NRX3[i]], mem[Audio::NRX4[i]] & 0b0000_0111]);

                self.waveform_positions[i] = (self.waveform_positions[i] + 1) % 8;

                self.current_samples[i] = if self.output_enabled[i] {
                    let duty_id = (mem[Audio::NRX1[i]] & 0b1100_0000) >> 6;
                    let waveform: u8 = match duty_id {
                        0 => 0b0000_0001,
                        1 => 0b1000_0001,
                        2 => 0b1000_0111,
                        3 => 0b0111_1110,
                        _ => panic!("This should never happen!"),
                    };

                    0.25 - f32::from(
                        ((waveform >> self.waveform_positions[i]) & 0b0000_0001)
                            * self.envelope_values[i],
                    ) / 30.0
                } else {
                    -0.25
                };
            }
        }

        // Wave table sound
        let i = 2;
        if self.frequency_timers[i] != 0 {
            self.frequency_timers[i] -= 1;
        } else {
            // Advance to next waveform sample.
            self.frequency_timers[i] = (2048
                - u16::from_le_bytes([mem[Audio::NRX3[i]], mem[Audio::NRX4[i]] & 0b0000_0111]))
                / 2;

            self.waveform_positions[i] = (self.waveform_positions[i] + 1) % 32;

            let address = (0xFF30 + self.waveform_positions[i] / 2) as u16;
            let offset = self.waveform_positions[i] % 2;

            self.current_samples[i] = if self.output_enabled[i] {
                let val = if offset == 0 {
                    mem[address] >> 4
                } else {
                    mem[address] & 0b0000_1111
                };

                0.25 - f32::from(val) / 30.0
            } else {
                -0.25
            };
        }

        // Noise sound
        self.current_samples[3] = if self.output_enabled[3] {
            let mut rng = rand::thread_rng();
            let y: bool = rng.gen();

            0.25 - f32::from((y as u8) * self.envelope_values[3]) / 30.0
        } else {
            -0.25
        };

        if self.decimation_timer == 0 {
            self.sample_buffer[self.sample_buffer_index] = 0.05
                * (self.current_samples[0]
                    + self.current_samples[1]
                    + self.current_samples[2]
                    + self.current_samples[3]);

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
            //tick_disabled: false,
            frequency_timers: [0; 4],
            length_counters: [0; 4],
            envelope_counters: [0; 4],
            envelope_values: [0; 4],
            output_enabled: [false; 4],
            waveform_positions: [0; 4],
            sample_buffer: [0.0; 1024],
            sample_buffer_index: 0,
            current_samples: [0.0; 4],
            decimation_timer: 15,
            frame_step: 0,
            frame_timer: 2047,
            sweep_timer: 0,
            volume_timer: 0,
        }
    }
}
