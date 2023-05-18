mod cpu;

use cpu::Cpu;

#[cfg(not(tarpaulin_include))]

fn main() {

    let cpu = Cpu::new();
    cpu.print_stats();

}
