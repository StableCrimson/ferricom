
/*
    Aliases for the flags in the 6502 status register.
    More information on these flags can be found here: https://www.nesdev.org/wiki/Status_flags
*/
const CARRY_FLAG: u8 =              0b0000_0001;
const ZERO_FLAG: u8 =               0b0000_0010;
const INTERRUPT_DISABLE_FLAG: u8 =  0b0000_0100;
const DECIMAL_MODE_FLAG: u8 =            0b0000_1000;

/* Bits 4 and 5 are unused */

const OVERFLOW_FLAG: u8 =           0b0100_0000;
const NEGATIVE_FLAG: u8 =           0b1000_0000;

enum AddressingMode {
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndirectX,
    IndirectY,
    Immediate,
}

pub struct Cpu {

    pc: u16,
    sp: u8,
    acc: u8,
    x: u8,
    y: u8,
    status: u8,
    memory: [u8; 0xFFFF]

}

impl Cpu {

    pub fn new() -> Cpu {

        Cpu {
            pc: 0x8000,
            sp: 0,
            acc: 0,
            x: 0,
            y: 0,
            status: 0,
            memory: [0; 0xFFFF]
        }
    }

    pub fn reset(&mut self) {
        self.pc = self.mem_read_u16(0xFFFC);
        self.sp = 0;
        self.acc = 0;
        self.x = 0;
        self.y = 0;
        self.status = 0;
    }
    
    // TODO: Separate the memory from the CPU struct
    /// The reason I have this method instead of just deriving debug is
    /// because, as of right now, memory is a part of the CPU struct. So printing
    /// with debug would flood the console with the contents of the NES' RAM.

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

    /// Loads the program into memory, starting at address 0x8000.
    /// Calling this method WILL reset the CPU state. If you want to test the CPU
    /// While in a custom state, do not call this, and instead set the state, call load(), then run()
    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run();
    }

    pub fn load(&mut self, program: Vec<u8>) {
        self.memory[0x8000..(0x8000 + program.len())].copy_from_slice(&program[..]);
        self.mem_write_u16(0xFFFC, 0x8000);
    }

    pub fn run(&mut self) {

        loop {

            let opcode = self.mem_read_u8(self.pc);
            self.pc += 1;

            match opcode {

                /* BRK - Force Interrupt */
                0x00 => return,

                /* LDA - Load Accumulator */
                0xA9 => self.lda(AddressingMode::Immediate),
                0xA5 => self.lda(AddressingMode::ZeroPage),     // TODO: Unit test
                0xB5 => self.lda(AddressingMode::ZeroPageX),    // TODO: Unit test
                0xAD => self.lda(AddressingMode::Absolute),     // TODO: Unit test
                0xBD => self.lda(AddressingMode::AbsoluteX),    // TODO: Unit test
                0xB9 => self.lda(AddressingMode::AbsoluteY),    // TODO: Unit test
                0xA1 => self.lda(AddressingMode::IndirectX),    // TODO: Unit test
                0xB1 => self.lda(AddressingMode::IndirectY),    // TODO: Unit test

                /* TAX - Transfer Accumulator to X */
                0xAA => {
                    self.x = self.acc;
                    self.set_negative_and_zero_bits(self.x);
                },

                /* TAY - Transfer Accumulator to Y */
                0xA8 => {
                    self.y = self.acc;
                    self.set_negative_and_zero_bits(self.y);
                },

                /* TSX - Transfer Stack Pointer to X */
                0xBA => {
                    self.x = self.sp;
                    self.set_negative_and_zero_bits(self.x);
                },

                /* TXA - Transfer X to Accumulator */
                0x8A => {
                    self.acc = self.x;
                    self.set_negative_and_zero_bits(self.acc);
                },

                /* TXS - Transfer X to Stack Pointer */
                0x9A => self.sp = self.x,

                /* TYA - Transfer Y to Accumulator */
                0x98 => {
                    self.acc = self.y;
                    self.set_negative_and_zero_bits(self.acc);
                },

                /* CLC - Clear Carry Flag */
                0x18 => self.clear_flag(CARRY_FLAG),

                /* CLD - Clear Decimal Mode */
                0xD8 => self.clear_flag(DECIMAL_MODE_FLAG),

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

                _ => todo!("Opcode [0x{:0X}] is invalid or unimplemented", opcode)

            };

        }

    }

    fn get_operand_address(&mut self, addressing_mode: AddressingMode) -> u16 {

        match addressing_mode {

            AddressingMode::Immediate => self.pc,
            AddressingMode::Absolute => self.mem_read_u16(self.pc),
            AddressingMode::AbsoluteX => self.mem_read_u16(self.pc).wrapping_add(self.x as u16),
            AddressingMode::AbsoluteY => self.mem_read_u16(self.pc).wrapping_add(self.y as u16),
            AddressingMode::ZeroPage => self.mem_read_u8(self.pc) as u16,
            AddressingMode::ZeroPageX => self.mem_read_u8(self.pc).wrapping_add(self.x) as u16,
            AddressingMode::ZeroPageY => self.mem_read_u8(self.pc).wrapping_add(self.y) as u16,
            AddressingMode::Indirect => {
                let target_addr = self.mem_read_u16(self.pc);
                self.mem_read_u16(target_addr)
            },
            AddressingMode::IndirectX => {

                let initial_read_addr = self.mem_read_u8(self.pc);
                let offset_addr = initial_read_addr.wrapping_add(self.x);

                let lsb = self.mem_read_u8(offset_addr as u16);
                let msb = self.mem_read_u8(offset_addr.wrapping_add(1) as u16);

                (msb as u16) << 8 | lsb as u16

            },
            AddressingMode::IndirectY => {

                let initial_read_addr = self.mem_read_u8(self.pc);

                let lsb = self.mem_read_u8(initial_read_addr as u16);
                let msb = self.mem_read_u8(initial_read_addr.wrapping_add(1) as u16);
                let target_addr = (msb as u16) << 8 | lsb as u16;

                target_addr.wrapping_add(self.y as u16)

            }
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

    fn mem_read_u8(&mut self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn mem_read_u16(&mut self, addr: u16) -> u16 {

        let lsb = self.mem_read_u8(addr) as u16;
        let msb = self.mem_read_u8(addr+1) as u16;

        let data: u16 = (msb << 8) | lsb;
        data

    }

    fn mem_write_u8(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }

    fn mem_write_u16(&mut self, addr: u16, data: u16) {

        let msb = (data >> 8) as u8;
        let lsb = data as u8;

        self.mem_write_u8(addr, lsb);
        self.mem_write_u8(addr+1, msb);

    }

    fn increment_register(register: &mut u8) {
        *register = (*register).wrapping_add(1);
    }

    fn decrement_register(register: &mut u8) {
        *register = (*register).wrapping_sub(1);
    }

    fn clear_flag(&mut self, flag_alias: u8) {
        self.status &= !flag_alias;
    }

    fn lda(&mut self, addressing_mode: AddressingMode) {

        let address = self.get_operand_address(addressing_mode);
        let data = self.mem_read_u8(address);
        self.acc = data;
        self.pc += 1;
        self.set_negative_and_zero_bits(self.acc);

    }

}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_cpu_init() {

        let cpu = Cpu::new();

        assert_eq!(cpu.pc, 0x8000);
        assert_eq!(cpu.sp, 0);
        assert_eq!(cpu.acc, 0);
        assert_eq!(cpu.x, 0);
        assert_eq!(cpu.y, 0);
        assert_eq!(cpu.status, 0);

    }

    #[test]
    fn test_cpu_reset() {

        let mut cpu = Cpu::new();
        cpu.mem_write_u16(0xFFFC, 0x8000);

        cpu.acc = 52;
        cpu.sp = 124;
        cpu.pc = 1892;
        cpu.x = 15;
        cpu.y = 16;
        cpu.status = 0b10010000;

        cpu.reset();

        assert_eq!(cpu.pc, 0x8000);
        assert_eq!(cpu.sp, 0);
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
    fn test_mem_read_u8 () {

        let mut cpu = Cpu::new();
        cpu.memory[162] = 0xAF;

        assert_eq!(cpu.mem_read_u8(162), 0xAF);

    }

    #[test]
    fn test_mem_read_u16 () {

        let mut cpu = Cpu::new();
        cpu.memory[162] = 0x80;
        cpu.memory[163] = 0x08;

        assert_eq!(cpu.mem_read_u16(162), 0x0880);

    }

    #[test]
    fn test_mem_write_u8 () {

        let mut cpu = Cpu::new();
        let data: u8 = 0x12;
        cpu.mem_write_u8(162, data);

        assert_eq!(cpu.memory[162], 0x12);

    }

    #[test]
    fn test_mem_write_u16 () {

        let mut cpu = Cpu::new();
        let data: u16 = 0x1234;
        cpu.mem_write_u16(162, data);

        assert_eq!(cpu.memory[162], 0x34);
        assert_eq!(cpu.memory[163], 0x12);

    }

    #[test]
    fn test_clear_flag() {

        let mut cpu = Cpu::new();
        cpu.status = 0b1111_1111;

        cpu.clear_flag(ZERO_FLAG);
        assert_eq!(cpu.status, 0b1111_1101);

    }

    #[test]
    fn test_get_operand_address_immediate() {

        let mut cpu = Cpu::new();
        cpu.acc = 0x10;
        cpu.x = 0x11;
        cpu.y = 0x12;
        cpu.sp = 0x13;
        cpu.pc = 0xF0;

        assert_eq!(cpu.get_operand_address(AddressingMode::Immediate), 0xF0);

    }

    #[test]
    fn test_get_operand_address_absolute() {

        let mut cpu = Cpu::new();
        cpu.acc = 0x10;
        cpu.x = 0x11;
        cpu.y = 0x12;
        cpu.sp = 0x13;
        cpu.pc = 0xF0;

        cpu.memory[0xF0] = 0x88;
        cpu.memory[0xF1] = 0x80;

        // Absolute addressing
        assert_eq!(cpu.get_operand_address(AddressingMode::Absolute), 0x8088);
        assert_eq!(cpu.get_operand_address(AddressingMode::AbsoluteX), 0x8099);
        assert_eq!(cpu.get_operand_address(AddressingMode::AbsoluteY), 0x809A);

        cpu.memory[0xF0] = 0xF0;
        cpu.memory[0xF1] = 0xFF;

        // Absolute addressing wrap around
        assert_eq!(cpu.get_operand_address(AddressingMode::AbsoluteX), 0x01);
        assert_eq!(cpu.get_operand_address(AddressingMode::AbsoluteY), 0x02);

    }

    #[test]
    fn test_get_operand_address_zero_page() {

        let mut cpu = Cpu::new();
        cpu.acc = 0x10;
        cpu.x = 0x11;
        cpu.y = 0x12;
        cpu.sp = 0x13;
        cpu.pc = 0xF0;

        cpu.memory[0xF0] = 0x88;
        cpu.memory[0xF1] = 0x80;

        // Zero page addressing
        assert_eq!(cpu.get_operand_address(AddressingMode::ZeroPage), 0x88);
        assert_eq!(cpu.get_operand_address(AddressingMode::ZeroPageX), 0x99);
        assert_eq!(cpu.get_operand_address(AddressingMode::ZeroPageY), 0x9A);

        cpu.memory[0xF0] = 0xF0;
        cpu.memory[0xF1] = 0xFF;

        // Zero page addressing wrap around
        assert_eq!(cpu.get_operand_address(AddressingMode::ZeroPageX), 0x01);
        assert_eq!(cpu.get_operand_address(AddressingMode::ZeroPageY), 0x02);

    }

    #[test]
    fn test_get_operand_address_indirect() {

        let mut cpu = Cpu::new();
        cpu.acc = 0x10;
        cpu.x = 0x11;
        cpu.y = 0x12;
        cpu.sp = 0x13;
        cpu.pc = 0xF0;

        cpu.memory[0xF0] = 0x88;
        cpu.memory[0xF1] = 0x80;
        cpu.memory[0x8088] = 0x34;
        cpu.memory[0x8089] = 0x12;
        cpu.memory[0x88] = 0x89;
        cpu.memory[0x89] = 0x67;
        cpu.memory[0x99] = 0x78;
        cpu.memory[0x9A] = 0x56;

        // Indirect addressing
        println!("Indirect X: {}", cpu.get_operand_address(AddressingMode::IndirectX));
        assert_eq!(cpu.get_operand_address(AddressingMode::Indirect), 0x1234);
        assert_eq!(cpu.get_operand_address(AddressingMode::IndirectX), 0x5678);
        assert_eq!(cpu.get_operand_address(AddressingMode::IndirectY), 0x679B);

    }

    #[test]
    fn test_lda_immediate() {

        let mut cpu = Cpu::new();

        // Negative bit is set
        let program = vec![0xA9, 156, 0x00];
        cpu.load_and_run(program);

        assert_eq!(cpu.acc, 156);

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

        cpu.load_and_run(program);

        assert_eq!(cpu.acc, 0xC0);
        assert_eq!(cpu.x, 0xC1);

    }

    #[test]
    fn test_tax () {

        let mut cpu = Cpu::new();
        cpu.acc = 156;

        let program = vec![0xAA, 0x00];
        cpu.load(program);
        cpu.run();

        assert_eq!(cpu.x, 156);
        assert!(cpu.status & NEGATIVE_FLAG > 0);

    }

    #[test]
    fn test_tay () {

        let mut cpu = Cpu::new();
        cpu.acc = 156;

        let program = vec![0xA8, 0x00];
        cpu.load(program);
        cpu.run();

        assert_eq!(cpu.y, 156);
        assert!(cpu.status & NEGATIVE_FLAG > 0);

    }

    #[test]
    fn test_tsx () {

        let mut cpu = Cpu::new();
        cpu.sp = 156;

        let program = vec![0xBA, 0x00];
        cpu.load(program);
        cpu.run();

        assert_eq!(cpu.x, 156);
        assert!(cpu.status & NEGATIVE_FLAG > 0);

    }

    #[test]
    fn test_txa () {

        let mut cpu = Cpu::new();
        cpu.x = 156;

        let program = vec![0x8A, 0x00];
        cpu.load(program);
        cpu.run();

        assert_eq!(cpu.acc, 156);
        assert!(cpu.status & NEGATIVE_FLAG > 0);

    }

    #[test]
    fn test_txs () {

        let mut cpu = Cpu::new();
        cpu.x = 156;

        let program = vec![0x9A, 0x00];
        cpu.load(program);
        cpu.run();

        assert_eq!(cpu.sp, 156);

    }

    #[test]
    fn test_tya () {

        let mut cpu = Cpu::new();
        cpu.y = 156;

        let program = vec![0x98, 0x00];
        cpu.load(program);
        cpu.run();

        assert_eq!(cpu.acc, 156);
        assert!(cpu.status & NEGATIVE_FLAG > 0);

    }

    #[test]
    fn test_inx () {

        let mut cpu = Cpu::new();
        cpu.x = 127;
        cpu.set_negative_and_zero_bits(cpu.x);
        assert_eq!(cpu.status & NEGATIVE_FLAG, 0);

        let program = vec![0xE8, 0x00];
        cpu.load(program);
        cpu.run();

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
        cpu.load(program);
        cpu.run();

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
        cpu.load(program);
        cpu.run();

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
        cpu.load(program);
        cpu.run();

        assert_eq!(cpu.y, 127);
        assert_eq!(cpu.status & NEGATIVE_FLAG, 0);

    }

    #[test]
    fn test_clc() {

        let mut cpu = Cpu::new();
        cpu.status = 0b1111_1111;

        let program = vec![0x18, 0x00];
        cpu.load(program);
        cpu.status = 0b1111_1111;
        cpu.run();

        assert_eq!(cpu.status, !CARRY_FLAG);

    }

    #[test]
    fn test_cld() {

        let mut cpu = Cpu::new();
        cpu.status = 0b1111_1111;

        let program = vec![0xD8, 0x00];
        cpu.load(program);
        cpu.status = 0b1111_1111;
        cpu.run();

        assert_eq!(cpu.status, !DECIMAL_MODE_FLAG);

    }

    #[test]
    fn test_cli() {

        let mut cpu = Cpu::new();
        cpu.status = 0b1111_1111;

        let program = vec![0x58, 0x00];
        cpu.load(program);
        cpu.status = 0b1111_1111;
        cpu.run();

        assert_eq!(cpu.status, !INTERRUPT_DISABLE_FLAG);

    }

    #[test]
    fn test_clv() {

        let mut cpu = Cpu::new();

        let program = vec![0xB8, 0x00];
        cpu.load(program);
        cpu.status = 0b1111_1111;
        cpu.run();

        assert_eq!(cpu.status, !OVERFLOW_FLAG);

    }

}