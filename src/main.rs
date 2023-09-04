pub mod cpu;
pub mod ppu;
pub mod instructions;
pub mod rom;
pub mod bus;
pub mod gamepad;

extern crate lazy_static;
extern crate bitflags;

use cpu::CPU;
use cpu::cpu_trace::trace;
use cpu::cpu_status_flags::CPUFlags;
use bus::Bus;
use ppu::{palette, render, PPU};
use ppu::frame::Frame;
use rom::ROM;
use gamepad::Gamepad;
use gamepad::gamepad_register::JoypadButton;

use log::{LevelFilter, info, warn, error, trace};
use clap::Parser;
use std::collections::HashMap;
use sdl2::{pixels::PixelFormatEnum, event::Event, keyboard::Keycode};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
pub struct Arguments {

  /// Path to the ROM file to load, ending in `.nes`
  rom_file: String,

  /// Enable generating a tracelog of the CPU.
  /// Will be found in `./logs/cpu_trace.log`
  #[arg(short, long, default_value_t=false)]
  cpu_tracelog: bool,

  /// Enable if running the `nestest.nes` ROM without a PPU.
  /// This sets the CPU program counter to 0xC000,
  /// which will skip the graphical output
  #[arg(short, long, default_value_t=false)]
  disable_nestest_ppu_output: bool

}

#[cfg(not(tarpaulin_include))]
fn main() {

    simple_logging::log_to_file("logs/log.log", LevelFilter::Debug).unwrap();

    let args = Arguments::parse();

    let file_path = &args.rom_file;
    info!("Target ROM: {}", file_path);

    let cpu_tracing_enabled = args.cpu_tracelog;
    let nestest_ppu_disabled = args.disable_nestest_ppu_output;

    let rom = match ROM::from_path(file_path) {
      Ok(rom) => rom,
      Err(msg) => {
        error!("{msg}");
        panic!("{msg}");
      }
    };

    // info!("ROM is 0X{:0X} bytes in size", byte_code.len());
    info!("ROM successfully loaded!");
    info!("========================");
    info!("Program ROM: 0X{:0X} bytes", rom.prg_rom.len());
    info!("Character ROM: 0X{:0X} bytes", rom.chr_rom.len());

    let window_title = format!("ferricom v0.1.0 EXPERIMENTAL | {}", rom.name);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window(&window_title, (256.0 * 3.0) as u32, (240.0 * 3.0) as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    canvas.set_scale(3.0, 3.0).unwrap();

    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_target(PixelFormatEnum::RGB24, 256, 240)
        .unwrap();

    let mut frame = Frame::new();

    let mut key_map = HashMap::new();
    key_map.insert(Keycode::Down, JoypadButton::DOWN);
    key_map.insert(Keycode::Up, JoypadButton::UP);
    key_map.insert(Keycode::Right, JoypadButton::RIGHT);
    key_map.insert(Keycode::Left, JoypadButton::LEFT);
    key_map.insert(Keycode::Space, JoypadButton::SELECT);
    key_map.insert(Keycode::Return, JoypadButton::START);
    key_map.insert(Keycode::A, JoypadButton::BUTTON_A);
    key_map.insert(Keycode::S, JoypadButton::BUTTON_B);

    let bus = Bus::new(rom, move |ppu: &PPU, gamepad: &mut Gamepad| {
      render::render(ppu, &mut frame);
        texture.update(None, &frame.data, 256 * 3).unwrap();

        canvas.copy(&texture, None, None).unwrap();

        canvas.present();
        for event in event_pump.poll_iter() {
            match event {
              Event::Quit { .. }
              | Event::KeyDown {
                  keycode: Some(Keycode::Escape),
                  ..
              } => std::process::exit(0),

              Event::KeyDown { keycode, .. } => {
                if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                    gamepad.set_button_pressed_status(*key, true);
                }
            }
            Event::KeyUp { keycode, .. } => {
                if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
                    gamepad.set_button_pressed_status(*key, false);
                }
            }

              _ => { /* do nothing */ }
            }
         }
    });

    let mut cpu = CPU::new(bus);
    
    if nestest_ppu_disabled {
      warn!("Setting program counter to 0xC000. This is a feature for testing only, and is not intended for use when loading actual games.");
      cpu.pc = 0xC000;
      cpu.status = CPUFlags::from_bits_truncate(0x24);
    } else {
      cpu.reset();
    }

    let _ = simple_logging::log_to_file("logs/cpu_trace.log", LevelFilter::Trace);

    cpu.run_with_callback(move |cpu| {

        if cpu_tracing_enabled {
          let trace_line = trace(cpu);
          trace!("{}", trace_line);
          // println!("{}", trace_line);
        }

    });
}