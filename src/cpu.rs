pub struct Cpu {

pc: u16,
sp: u16,
acc: u8,
x: u8,
y: u8,
status: u8,
memory: [u8; 0xFFFF]

}

impl Cpu {

    pub fn new() -> Cpu {

        let mut cpu = Cpu {
            pc: 0,
            sp: 0,
            acc: 0,
            x: 0,
            y: 0,
            status: 0,
            memory: [0; 0xFFFF]
        };

        cpu.reset();
        cpu

    }

    pub fn reset(&mut self) {
        self.pc = 0xFFFC;
        self.sp = 0x0100;
        self.acc = 0;
        self.x = 0;
        self.y = 0;
        self.status = 0;
    }

    #[cfg(not(tarpaulin_include))]
    pub fn print_stats(&self) {

        println!("Program counter:  {}", self.pc);
        println!("Stack pointer:    {}", self.sp);
        println!("Accumulator:      {}", self.acc);
        println!("X register:       {}", self.x);
        println!("Y register:       {}", self.y);
        println!("Memory at SP:     {}", self.memory[self.sp as usize]);
        println!("Status bits:      NV-BDIZC");
        println!("Status bits:    {:#010b}", self.status);

    }

}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_init() {

        let cpu = Cpu::new();

        assert_eq!(cpu.pc, 0xFFFC);
        assert_eq!(cpu.sp, 0x0100);
        assert_eq!(cpu.acc, 0);
        assert_eq!(cpu.x, 0);
        assert_eq!(cpu.y, 0);
        assert_eq!(cpu.status, 0);

    }

    #[test]
    fn test_reset() {

        let mut cpu = Cpu::new();

        cpu.acc = 52;
        cpu.sp = 1234;
        cpu.pc = 1892;
        cpu.x = 15;
        cpu.y = 16;
        cpu.status = 0b10010000;

        cpu.reset();

        assert_eq!(cpu.pc, 0xFFFC);
        assert_eq!(cpu.sp, 0x0100);
        assert_eq!(cpu.acc, 0);
        assert_eq!(cpu.x, 0);
        assert_eq!(cpu.y, 0);
        assert_eq!(cpu.status, 0);

    }
}