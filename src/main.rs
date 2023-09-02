pub mod cpu;
pub mod instructions;
pub mod rom;
pub mod bus;
pub mod cpu_trace;

extern crate lazy_static;
extern crate bitflags;

use cpu::{CPU, Mem};
use bus::Bus;
use rom::ROM;
use cpu_trace::trace;

use std::fs;
use log::{LevelFilter, info, warn, error, trace};
use clap::Parser;

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
    use crate::cpu::StatusRegister;


    simple_logging::log_to_file("logs/log.log", LevelFilter::Debug).unwrap();

    let args = Arguments::parse();

    let file_path = &args.rom_file;
    info!("Target ROM: {}", file_path);

    let cpu_tracing_enabled = args.cpu_tracelog;
    let nestest_ppu_disabled = args.disable_nestest_ppu_output;

    let byte_code = match fs::read(file_path) {
        Ok(byte_code) => byte_code,
        Err(_) => {
            let msg = format!("Unable to read ROM \"{file_path}\"");
            error!("{msg}");
            panic!("{msg}")
        },
    };

    info!("ROM is 0X{:0X} bytes in size", byte_code.len());
    let rom = ROM::new(&byte_code);
    info!("ROM successfully loaded!");
    info!("========================");
    info!("Program ROM: 0X{:0X} bytes", rom.prg_rom.len());
    info!("Character ROM: 0X{:0X} bytes", rom.chr_rom.len());

    // let snake_game_code: &Vec<u8> = &(*games::example::SNAKE_GAME_CODE); // example
    let mut cpu = CPU::new(Bus::new(rom));
    
    if nestest_ppu_disabled {
      warn!("Setting program counter to 0xC000. This is a feature for testing only, and is not intended for use when loading actual games.");
      cpu.pc = 0xC000;
      cpu.status = StatusRegister::from_bits_truncate(0x24);
    } else {
      cpu.reset();
    }

    let _ = simple_logging::log_to_file("logs/cpu_trace.log", LevelFilter::Trace);

    cpu.run_with_callback(move |cpu| {

        if cpu_tracing_enabled {
          let trace_line = trace(cpu);
          trace!("{}", trace_line);
          println!("{}", trace_line);
        }

    });

    info!("0x6000: {:0x}", cpu.mem_read_u16(0x6000));

}
