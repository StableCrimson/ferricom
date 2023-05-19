pub mod cpu;
pub mod instructions;

extern crate lazy_static;

use cpu::Cpu;

#[cfg(not(tarpaulin_include))]

fn main() {

    let mut cpu = Cpu::new();

    println!("Load 153 into accumulator, transfer it to the X register");
    cpu.load_and_run(vec![0xA9, 153, 0xAA, 0x00]);
    cpu.print_stats();

}
