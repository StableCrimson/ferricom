pub mod cpu;
pub mod instructions;
pub mod rom;

extern crate lazy_static;

use std::{fs, env, process};
use cpu::CPU;
use rom::ROM;

#[cfg(not(tarpaulin_include))]
fn main() {

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("ERROR: No ROM file specified");
        process::exit(1);
    }

    let file_path = &args[1];
    println!("Target ROM: {}", file_path);

    let byte_code = match fs::read(file_path) {
        Ok(byte_code) => byte_code,
        Err(_) => panic!("ERROR: Unable to load ROM \"{file_path}\". File path is invalid")
    };

    println!("ROM is 0X{:?} bytes in size", byte_code.len());
    let rom = ROM::new(&byte_code);
    println!("ROM successfully loaded!");
    println!("========================");
    println!("Program ROM: 0X{:0X} bytes", rom.prg_rom.len());
    println!("Character ROM: 0X{:0X} bytes", rom.chr_rom.len());

    let mut cpu = CPU::new();

    println!("Load 153 into accumulator, transfer it to the X register");
    cpu.load_and_run(vec![0xA9, 153, 0xAA, 0x00]);
    cpu.print_stats();

}
