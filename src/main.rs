mod audio;
mod cpu;
mod memory;
mod timer;
mod video;

use audio::Audio;
use cpu::CPU;
use memory::Memory;
use sdl2::{
    audio::AudioSpecDesired,
    event::Event,
    keyboard::Keycode,
    pixels::{Color, PixelFormatEnum},
};
use std::{cell::RefCell, env, error::Error, rc::Rc};
use timer::Timer;
use video::Video;

const PROGRAM_NAME: &str = "Gaby";

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("One Game Boy ROM file path must be given as command line argument.");
    }

    let rc_mem = Rc::new(RefCell::new(Memory::new()));
    let title: String;

    {
        let mut mem = rc_mem.borrow_mut();
        mem.load_rom(&args[1])?;
        title = mem.read_game_title();
    }
    println!("Title: {}", title);

    let mut cpu = CPU::new(rc_mem.clone());
    cpu.print_instructions = false;

    let mut audio = Audio::new(rc_mem.clone());
    let mut video = Video::new(rc_mem.clone());
    let mut timer = Timer::new(rc_mem.clone());

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let audio_subsystem = sdl_context.audio()?;

    let window_width = u32::from(video::SCREEN_WIDTH) * 4;
    let window_height = u32::from(video::SCREEN_HEIGHT) * 4;
    let window_title = format!("{} - {}", PROGRAM_NAME, title);

    let window = video_subsystem
        .window(&window_title, window_width, window_height)
        .position_centered()
        .build()?;

    let mut canvas = window.into_canvas().present_vsync().build()?;
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    // Make a texture that is to be copied into the canvas every frame.
    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator.create_texture_streaming(
        PixelFormatEnum::RGB24,
        u32::from(video::SCREEN_WIDTH),
        u32::from(video::SCREEN_HEIGHT),
    )?;

    // Set up audio.
    let desired_spec = AudioSpecDesired {
        freq: Some(65536),
        channels: Some(1),   // mono
        samples: Some(1024), // for less than 1 frame delay
    };

    let audio_queue = audio_subsystem.open_queue(None, &desired_spec)?;

    // Start playback
    audio_queue.resume();

    let mut event_pump = sdl_context.event_pump()?;

    // SDL event loop.
    'render_loop: loop {
        for event in event_pump.poll_iter() {
            match event {
                // Exit the event loop if the user closes the window or presses
                // the escape key.
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'render_loop Ok(()),
                _ => {}
            }
        }

        texture.update(None, video.pixel_data(), 3 * video::SCREEN_WIDTH as usize)?;
        canvas.copy(&texture, None, None)?;

        canvas.present();

        for _ in 0..17556 {
            timer.tick()?;
            video.tick()?;
            audio.tick(&audio_queue)?;
            cpu.tick()?;
        }
    }
}
