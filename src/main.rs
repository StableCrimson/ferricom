mod cpu;

use cpu::Cpu;

fn main() {

    let mut cpu = Cpu::new();

    println!("Load 153 into accumulator, transfer it to the X register");
    cpu.run(vec![0xA9, 153, 0xAA, 0x00]);
    cpu.print_stats();

}
