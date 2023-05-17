
/*

    Aliases for the flags in the 6502 status register.
    More information on these flags can be found here: https://www.nesdev.org/wiki/Status_flags

*/
const CARRY_FLAG: u8 =              0b0000_0001;
const ZERO_FLAG: u8 =               0b0000_0010;
const INTERRUPT_DISABLE_FLAG: u8 =  0b0000_0100;
const DECIMAL_FLAG: u8 =            0b0000_1000;

/* Bits 4 and 5 are unused */

const OVERFLOW_FLAG: u8 =           0b0100_0000;
const NEGATIVE_FLAG: u8 =           0b1000_0000;

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
                    
                    // let data = self.mem_read_u8(self.pc);
                    // self.acc = data;

                    let data = program[self.pc as usize]
;                   self.acc = data;
                    self.pc += 1;
                    
                    self.set_negative_and_zero_bits(self.acc);

                },

                /* TAX - Transfer Accumulator to X */
                0xAA => {
                    self.x = self.acc;
                    self.set_negative_and_zero_bits(self.x);
                },

                /* CLC - Clear Carry Flag */
                0x18 => self.clear_flag(CARRY_FLAG),

                /* CLD - Clear Decimal Mode */
                0xD8 => self.clear_flag(DECIMAL_FLAG),

                /* CLI - Clear Interrupt Disable */
                0x58 => self.clear_flag(INTERRUPT_DISABLE_FLAG),

                /* CLV - Clear Overflow Flag */
                0xB8 => self.clear_flag(OVERFLOW_FLAG),

                /* DEX - Decrement the X Register */
                0xCA => {
                    Cpu::decrement_register(&mut self.x);
                    self.set_negative_and_zero_bits(self.x);
                },

                /* DEY - Decrement the Y Register */
                0x88 => {
                    Cpu::decrement_register(&mut self.y);
                    self.set_negative_and_zero_bits(self.y);
                },

                /* INX - Increment the X Register */
                0xE8 => {
                    Cpu::increment_register(&mut self.x);
                    self.set_negative_and_zero_bits(self.x);
                },

                /* INY - Increment the Y Register */
                0xC8 => {
                    Cpu::increment_register(&mut self.y);
                    self.set_negative_and_zero_bits(self.y);
                }

                _ => todo!("Instruction invalid or unimplemented")

            };

        }

    }

    fn set_negative_and_zero_bits(&mut self, value: u8) {

        if value == 0 {
            self.status |= ZERO_FLAG;
        } else {
            self.status &= !ZERO_FLAG;
        }

        if value & 0b1000_0000 != 0 {
            self.status |= NEGATIVE_FLAG;
        } else {
            self.status &= !NEGATIVE_FLAG;
        }

    }

    // TODO Load the program into memory instead of just accessing it seperately
    // fn mem_read_u8(&mut self, addr: u16) -> u8 {
    //     let data = self.memory[addr as usize];
    //     self.pc += 1;
    //     data
    // }

    // fn mem_write_u8(&mut self, addr: u16, data: u8) {
    //     self.memory[addr as usize] = data;
    //     self.pc += 1;
    // }

    fn increment_register(register: &mut u8) {
        if *register == 0xFF {
            *register = 0;
        } else {
            *register += 1;
        }
    }

    fn decrement_register(register: &mut u8) {
        if *register == 0 {
            *register = 0xFF;
        } else {
            *register -= 1;
        }
    }

    fn clear_flag(&mut self, flag_alias: u8) {
        self.status &= !flag_alias;
    }

}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_cpu_init() {

        let cpu = Cpu::new();

        assert_eq!(cpu.pc, 0);
        assert_eq!(cpu.sp, 0x0100);
        assert_eq!(cpu.acc, 0);
        assert_eq!(cpu.x, 0);
        assert_eq!(cpu.y, 0);
        assert_eq!(cpu.status, 0);

    }

    #[test]
    fn test_cpu_reset() {

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
    fn test_set_negative_and_zero_flags() {

        let mut cpu = Cpu::new();

        cpu.set_negative_and_zero_bits(cpu.acc);
        assert!(cpu.status & ZERO_FLAG > 0);

        cpu.acc = 130;
        cpu.set_negative_and_zero_bits(cpu.acc);
        assert!(cpu.status & NEGATIVE_FLAG > 0);

        cpu.acc = 16;
        cpu.set_negative_and_zero_bits(cpu.acc);
        assert_eq!(cpu.status, 0);

    }

    #[test]
    fn test_increment_register () {

        let mut cpu = Cpu::new();
        cpu.x = 0xFE;

        Cpu::increment_register(&mut cpu.x);
        assert_eq!(cpu.x, 0xFF);

        Cpu::increment_register(&mut cpu.x);
        assert_eq!(cpu.x, 0);

    }

    #[test]
    fn test_decrement_register () {

        let mut cpu = Cpu::new();
        cpu.x = 1;

        Cpu::decrement_register(&mut cpu.x);
        assert_eq!(cpu.x, 0);

        Cpu::decrement_register(&mut cpu.x);
        assert_eq!(cpu.x, 255);

    }


    #[test]
    fn test_clear_flag() {

        let mut cpu = Cpu::new();
        cpu.status = 0b1111_1111;

        cpu.clear_flag(ZERO_FLAG);
        assert_eq!(cpu.status, 0b1111_1101);

    }

    #[test]
    fn test_run_sample_prog_1() {

        /*
            This program does the following:
            Load 0xC0 into the accumulator
            Transfer to the X register
            Increment X
         */

        let mut cpu = Cpu::new();
        let program = vec![0xA9, 0xC0, 0xAA, 0xE8, 0x00];

        cpu.interpret(program);

        assert_eq!(cpu.acc, 0xC0);
        assert_eq!(cpu.x, 0xC1);

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
        assert!(cpu.status & NEGATIVE_FLAG > 0);

    }

    #[test]
    fn test_inx () {

        let mut cpu = Cpu::new();
        cpu.x = 127;
        cpu.set_negative_and_zero_bits(cpu.x);
        assert_eq!(cpu.status & NEGATIVE_FLAG, 0);

        let program = vec![0xE8, 0x00];
        cpu.interpret(program);

        assert_eq!(cpu.x, 128);
        assert!(cpu.status & NEGATIVE_FLAG > 0);

    }

    #[test]
    fn test_iny () {

        let mut cpu = Cpu::new();
        cpu.y = 127;
        cpu.set_negative_and_zero_bits(cpu.y);
        assert_eq!(cpu.status & NEGATIVE_FLAG, 0);

        let program = vec![0xC8, 0x00];
        cpu.interpret(program);

        assert_eq!(cpu.y, 128);
        assert!(cpu.status & NEGATIVE_FLAG > 0);

    }

    #[test]
    fn test_dex () {

        let mut cpu = Cpu::new();
        cpu.x = 128;
        cpu.set_negative_and_zero_bits(cpu.x);
        assert!(cpu.status & NEGATIVE_FLAG > 0);

        let program = vec![0xCA, 0x00];
        cpu.interpret(program);

        assert_eq!(cpu.x, 127);
        assert_eq!(cpu.status & NEGATIVE_FLAG, 0);

    }

    #[test]
    fn test_dey () {

        let mut cpu = Cpu::new();
        cpu.y = 128;
        cpu.set_negative_and_zero_bits(cpu.y);
        assert!(cpu.status & NEGATIVE_FLAG > 0);

        let program = vec![0x88, 0x00];
        cpu.interpret(program);

        assert_eq!(cpu.y, 127);
        assert_eq!(cpu.status & NEGATIVE_FLAG, 0);

    }

    #[test]
    fn test_clc() {

        let mut cpu = Cpu::new();
        cpu.status = 0b1111_1111;

        let program = vec![0x18, 0x00];
        cpu.interpret(program);

        assert_eq!(cpu.status, !CARRY_FLAG);

    }

    #[test]
    fn test_cld() {

        let mut cpu = Cpu::new();
        cpu.status = 0b1111_1111;

        let program = vec![0xD8, 0x00];
        cpu.interpret(program);

        assert_eq!(cpu.status, !DECIMAL_FLAG);

    }

    #[test]
    fn test_cli() {

        let mut cpu = Cpu::new();
        cpu.status = 0b1111_1111;

        let program = vec![0x58, 0x00];
        cpu.interpret(program);

        assert_eq!(cpu.status, !INTERRUPT_DISABLE_FLAG);

    }

    #[test]
    fn test_clv() {

        let mut cpu = Cpu::new();
        cpu.status = 0b1111_1111;

        let program = vec![0xB8, 0x00];
        cpu.interpret(program);

        assert_eq!(cpu.status, !OVERFLOW_FLAG);

    }

}