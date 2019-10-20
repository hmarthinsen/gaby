use crate::memory::{IORegister, Memory};
use std::cell::RefCell;
use std::rc::Rc;

pub const SCREEN_WIDTH: u8 = 160;
pub const SCREEN_HEIGHT: u8 = 144;

const LY_MAX: u8 = 154;

const TICKS_VBLANK: u32 = 1140;
const TICKS_HBLANK: u32 = 51;
const TICKS_OAM: u32 = 20;
const TICKS_TRANSFER: u32 = 43;
const TICKS_PER_LINE: u32 = TICKS_HBLANK + TICKS_OAM + TICKS_TRANSFER;

// These constants are for both x-/y-direction.
const TILES_PER_BACKGROUND: u16 = 32;
const PIXELS_PER_TILE: u8 = 8;
const PIXELS_PER_BACKGROUND: usize = PIXELS_PER_TILE as usize * TILES_PER_BACKGROUND as usize;
const PIXELS_PER_BACKGROUND_SQUARED: usize = PIXELS_PER_BACKGROUND * PIXELS_PER_BACKGROUND;

const BYTES_PER_TILE: u16 = 16;
const BYTES_PER_PIXEL: usize = 3;
const BYTES_PER_LINE: usize = SCREEN_WIDTH as usize * BYTES_PER_PIXEL;
const BYTES_PER_SCREEN: usize = SCREEN_HEIGHT as usize * BYTES_PER_LINE;

pub struct Video {
    mem: Rc<RefCell<Memory>>,
    /// Pixel data that is written to the screen.
    pixel_data: [u8; BYTES_PER_SCREEN],
    /// Number of ticks left in current LCD mode.
    mode_counter: u32,
    /// Number of ticks left until this line is finished.
    line_counter: u32,
}

impl Video {
    pub fn tick(&mut self) -> Result<(), String> {
        if self.line_counter == 0 {
            let mut mem = self.mem.borrow_mut();
            let ly = mem[IORegister::LY];
            mem[IORegister::LY] = (ly + 1) % LY_MAX;

            if ly == mem[IORegister::LYC] {
                mem[IORegister::STAT] |= 0b0000_0100;

                if (mem[IORegister::STAT] & 0b0100_0000) != 0 {
                    mem[IORegister::IF] |= 0b0000_0010;
                }
            }

            self.line_counter = TICKS_PER_LINE;
        }

        if self.mode_counter == 0 {
            use LCDMode::*;
            match self.lcd_mode() {
                HBlank => {
                    let ly = self.mem.borrow()[IORegister::LY];
                    if ly == 144 {
                        self.set_lcd_mode(VBlank);
                    } else {
                        self.set_lcd_mode(OAM);
                    }
                }
                VBlank => self.set_lcd_mode(OAM),
                OAM => self.set_lcd_mode(Transfer),
                Transfer => self.set_lcd_mode(HBlank),
            }
        }

        self.mode_counter -= 1;
        self.line_counter -= 1;
        Ok(())
    }

    pub fn new(mem: Rc<RefCell<Memory>>) -> Self {
        Self {
            mem,
            pixel_data: [0; BYTES_PER_SCREEN],
            mode_counter: TICKS_OAM,
            line_counter: TICKS_PER_LINE,
        }
    }

    fn lcd_mode(&self) -> LCDMode {
        let stat = self.mem.borrow()[IORegister::STAT];
        let mode = stat & 0b0000_0011;
        use LCDMode::*;
        match mode {
            0 => HBlank,
            1 => VBlank,
            2 => OAM,
            3 => Transfer,
            _ => panic!("This should never happen."),
        }
    }

    fn set_lcd_mode(&mut self, mode: LCDMode) {
        use LCDMode::*;
        let mode_mask = match mode {
            HBlank => {
                let mut mem = self.mem.borrow_mut();
                if (mem[IORegister::STAT] & 0b0000_1000) != 0 {
                    mem[IORegister::IF] |= 0b0000_0010;
                }

                self.mode_counter = TICKS_HBLANK;
                0b0000_0000
            }
            VBlank => {
                let mut mem = self.mem.borrow_mut();
                if (mem[IORegister::STAT] & 0b0001_0000) != 0 {
                    mem[IORegister::IF] |= 0b0000_0010;
                }
                mem[IORegister::IF] |= 0b0000_0001;

                self.mode_counter = TICKS_VBLANK;
                0b0000_0001
            }
            OAM => {
                let mut mem = self.mem.borrow_mut();
                if (mem[IORegister::STAT] & 0b0010_0000) != 0 {
                    mem[IORegister::IF] |= 0b0000_0010;
                }

                self.mode_counter = TICKS_OAM;
                0b0000_0010
            }
            Transfer => {
                self.render_line();

                self.mode_counter = TICKS_TRANSFER;
                0b0000_0011
            }
        };
        let mut mem = self.mem.borrow_mut();
        let stat_without_mode = mem[IORegister::STAT] & 0b1111_1100;
        mem[IORegister::STAT] = stat_without_mode | mode_mask;
    }

    fn render_line(&mut self) {
        let mem = self.mem.borrow();

        // Draw current line of background.
        let lcdc = mem[IORegister::LCDC];
        let (tile_data_origin, signed_tile_indices) = if (lcdc & 0b0001_0000) != 0 {
            (0x8000, false)
        } else {
            (0x9000, true)
        };

        let bg_tile_map_origin = if (lcdc & 0b0000_1000) != 0 {
            0x9C00
        } else {
            0x9800
        };

        let scx = mem[IORegister::SCX];
        let scy = mem[IORegister::SCY];

        let y = mem[IORegister::LY].wrapping_add(scy);

        for x in 0..SCREEN_WIDTH {
            let x = x.wrapping_add(scx);

            let tile_x = (x / PIXELS_PER_TILE) as u16;
            let tile_y = (y / PIXELS_PER_TILE) as u16;
            let tile_offset = tile_y * TILES_PER_BACKGROUND + tile_x;

            // Coordinate inside current tile.
            let in_tile_x = x % PIXELS_PER_TILE;
            let in_tile_y = y % PIXELS_PER_TILE;

            let tile_index = mem[bg_tile_map_origin + tile_offset];
            let tile_data = if signed_tile_indices {
                let offset = (tile_index as i8) as i32 * BYTES_PER_TILE as i32;
                (tile_data_origin as i32 + offset) as u16
            } else {
                tile_data_origin + (tile_index as u16) * BYTES_PER_TILE
            };

            // Get bytes containing pixel data.
            let pixel_data = (
                mem[tile_data + in_tile_y as u16 * 2],
                mem[tile_data + in_tile_y as u16 * 2 + 1],
            );

            let mask = 1 << in_tile_x;
            let shade = if (pixel_data.1 & mask) == 0 {
                if (pixel_data.0 & mask) == 0 {
                    // 0
                    mem[IORegister::BGP] & 0b0000_0011
                } else {
                    // 1
                    (mem[IORegister::BGP] & 0b0000_1100) >> 2
                }
            } else {
                if (pixel_data.0 & mask) == 0 {
                    // 2
                    (mem[IORegister::BGP] & 0b0011_0000) >> 4
                } else {
                    // 3
                    (mem[IORegister::BGP] & 0b1100_0000) >> 6
                }
            };

            let pixel_value = self.shade_to_rgb(shade);
            let index = y as usize * BYTES_PER_LINE + x as usize * BYTES_PER_PIXEL;
            self.pixel_data[index] = pixel_value;
            self.pixel_data[index + 1] = pixel_value;
            self.pixel_data[index + 2] = pixel_value;
        }
    }

    pub fn pixel_data(&mut self) -> &[u8] {
        &self.pixel_data
    }

    /// Convert 2-bit shade to 8-bit for use in RGB.
    fn shade_to_rgb(&self, shade: u8) -> u8 {
        match shade {
            0 => 0,
            1 => 85,
            2 => 170,
            3 => 255,
            _ => panic!("Only values between 0 and 3 are valid shades."),
        }
    }
}

pub enum LCDMode {
    HBlank,
    VBlank,
    OAM,
    Transfer,
}
