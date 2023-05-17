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
        self.pc = 0;
        self.sp = 0x0100;
        self.acc = 0;
        self.x = 0;
        self.y = 0;
        self.status = 0;
    }

    /*
        The reason I have this method instead of just deriving debug is
        because, as of right now, memory is a part of the struct. So printing
        with debug would flood the console with the contents of the NES' RAM.
        TODO: Separate the memory from the CPU struct
     */
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

    pub fn interpret(&mut self, program: Vec<u8>) {

        loop {

            let opcode = program[self.pc as usize];
            self.pc += 1;

            match opcode {

                /* BRK - Force Interrupt */
                0x00 => return,

                /* LDA - Load Accumulator */
                0xA9 => { // Immediate
                    
                    let data: u8 = program[self.pc as usize];

                    // TODO: Add a method for reading a byte
                    self.acc = data;
                    self.pc += 1;

                    self.set_negative_and_zero_bits(self.acc);

                },

                /* TAX - Transfer Accumulator to X */
                0xAA => { // Implied
                    self.x = self.acc;
                    self.set_negative_and_zero_bits(self.x);
                },

                _ => todo!("Instruction invalid or unimplemented")

            };

        }

    }

    fn set_negative_and_zero_bits(&mut self, value: u8) {

        if value == 0 {
            self.status |= 0b0000_0010;
        } else {
            self.status &= 0b1111_1101;
        }

        if value & 0b1000_0000 != 0 {
            self.status |= 0b1000_0000;
        } else {
            self.status &= 0b0111_1111;
        }

    }

}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_init() {

        let cpu = Cpu::new();

        assert_eq!(cpu.pc, 0);
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

        assert_eq!(cpu.pc, 0);
        assert_eq!(cpu.sp, 0x0100);
        assert_eq!(cpu.acc, 0);
        assert_eq!(cpu.x, 0);
        assert_eq!(cpu.y, 0);
        assert_eq!(cpu.status, 0);

    }

    #[test]
    fn test_set_flags() {

        let mut cpu = Cpu::new();

        cpu.set_negative_and_zero_bits(cpu.acc);
        assert!(cpu.status & 0b0000_0010 > 0);

        cpu.acc = 130;
        cpu.set_negative_and_zero_bits(cpu.acc);
        assert!(cpu.status & 0b1000_0000 > 0);

        cpu.acc = 16;
        cpu.set_negative_and_zero_bits(cpu.acc);
        assert_eq!(cpu.status, 0);

    }

    #[test]
    fn test_lda_immediate() {

        let mut cpu = Cpu::new();

        // Negative bit is set
        let program = vec![0xA9, 156, 0x00];
        cpu.interpret(program);

        assert_eq!(cpu.acc, 156);

    }

    #[test]
    fn test_tax () {

        let mut cpu = Cpu::new();
        cpu.acc = 156;

        let program = vec![0xAA, 0x00];
        cpu.interpret(program);

        assert_eq!(cpu.x, 156);
        assert!(cpu.status & 0b1000_0000 > 0);

    }

}