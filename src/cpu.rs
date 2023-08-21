use crate::instructions::{self};
/*
    Aliases for the flags in the 6502 status register.
    More information on these flags can be found here: https://www.nesdev.org/wiki/Status_flags
*/
const CARRY_FLAG: u8 =              0b0000_0001;
const ZERO_FLAG: u8 =               0b0000_0010;
const INTERRUPT_DISABLE_FLAG: u8 =  0b0000_0100;
const DECIMAL_MODE_FLAG: u8 =       0b0000_1000;

/* Bits 4 and 5 are unused */

const OVERFLOW_FLAG: u8 =           0b0100_0000;
const NEGATIVE_FLAG: u8 =           0b1000_0000;

#[derive(PartialEq)]
pub enum RegisterID {
    ACC,
    X,
    Y,
    SP
}

#[derive(Debug)]
pub enum AddressingMode {
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
    Implied,
    Relative,
    None
}

pub struct CPU {

    pc: u16,
    sp: u8,
    acc: u8,
    x: u8,
    y: u8,
    status: u8,
    memory: [u8; 0x10000]

}

impl Default for CPU {
    fn default() -> Self {
        Self::new()
    }   
}

impl CPU {

    /// Create a new 6502 CPU in its default state
    pub fn new() -> CPU {

        CPU {
            pc: 0x8000,
            sp: 0xFF,
            acc: 0,
            x: 0,
            y: 0,
            status: 0,
            memory: [0; 0x10000]
        }
    }

    /// Sets the CPU to the default state
    pub fn reset(&mut self) {
        self.pc = self.mem_read_u16(0xFFFC);
        self.sp = 0xFF;
        self.acc = 0;
        self.x = 0;
        self.y = 0;
        self.status = 0;
    }
    
    // TODO: Separate the memory from the CPU struct
    /// The reason I have this method instead of just deriving debug is
    /// because, as of right now, memory is a part of the CPU struct. So printing
    /// with debug would flood the console with the contents of the NES' RAM and make the output
    /// completely unreadable.
    #[cfg(not(tarpaulin_include))]
    pub fn print_state(&self) {

        println!("Program counter:  {:0X}", self.pc);
        println!("Stack pointer:    {:0X}", self.sp);
        println!("Accumulator:      {:0X}", self.acc);
        println!("X register:       {:0X}", self.x);
        println!("Y register:       {:0X}", self.y);
        println!("Memory at SP:     {:0X}", self.memory[self.sp as usize]);
        println!("Status bits:      NV-BDIZC");
        println!("Status bits:    {:#010b}", self.status);

    }

    /// Loads the program into memory, starting at address 0x8000.
    /// Calling this method WILL reset the CPU state. If you want to test the CPU
    /// while in a custom state, do not call this and instead set the state, call load(), then run()
    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run();
    }

    /// Loads a program into memory at the specified address.
    /// Allows for use of custom test code that you may not want to begin executing
    /// at the default 6502 reset vector.
    pub fn load_custom_program(&mut self, program: Vec<u8>, start_vector: u16) {
        let program_length = program.len();
        self.memory[(start_vector as usize)..((start_vector as usize) + program_length)].copy_from_slice(&program[..]);
        self.mem_write_u16(0xFFFC, start_vector);
    }

    /// Loads a program into memory
    pub fn load(&mut self, program: Vec<u8>) {
        // self.memory[0x8000..(0x8000 + program.len())].copy_from_slice(&program[..]);
        // self.mem_write_u16(0xFFFC, 0x8000);
        self.load_custom_program(program, 0x8000);
    }

    /// Begins execution
    pub fn run(&mut self) {

        let ins_set = &(*instructions::CPU_INSTRUCTION_SET);

        // TODO REMOVE LATER
        println!("IMPLEMENTED {} OF 256 INSTRUCTIONS", ins_set.len());

        loop {

            let opcode = self.mem_read_u8(self.pc);
            self.pc += 1;
            let current_pc = self.pc;

            let ins = *ins_set.get(&opcode).unwrap_or_else(|| panic!("Instruction {} is invalid or unimplemented", opcode));

            match opcode {

                0x00 => return,
                0xEA => (),
                0x69 | 0x65 | 0x75 | 0x6D | 0x7D | 0x79 | 0x61 | 0x71 => self.add_with_carry(&ins.addressing_mode),
                0x29 | 0x25 | 0x35 | 0x2D | 0x3D | 0x39 | 0x21 | 0x31 => self.and(&ins.addressing_mode),
                0x24 | 0x2C => self.bit(&ins.addressing_mode),
                0xC9 | 0xC5 | 0xD5 | 0xCD | 0xDD | 0xD9 | 0xC1 | 0xD1 => self.compare_register(&ins.addressing_mode, &RegisterID::ACC),
                0xE0 | 0xE4 | 0xEC => self.compare_register(&ins.addressing_mode, &RegisterID::X),
                0xC0 | 0xC4 | 0xCC => self.compare_register(&ins.addressing_mode, &RegisterID::Y),
                0xA9 | 0xA5 | 0xB5 | 0xAD | 0xBD | 0xB9 | 0xA1 | 0xB1 => self.load_register(&ins.addressing_mode, &RegisterID::ACC),
                0xA2 | 0xA6 | 0xB6 | 0xAE | 0xBE => self.load_register(&ins.addressing_mode, &RegisterID::X),
                0xA0 | 0xA4 | 0xB4 | 0xAC | 0xBC => self.load_register(&ins.addressing_mode, &RegisterID::Y),
                0x85 | 0x95 | 0x8D | 0x9D | 0x99 | 0x81 | 0x91 => self.store_register(&ins.addressing_mode, &RegisterID::ACC),
                0x86 | 0x96 | 0x8E => self.store_register(&ins.addressing_mode, &RegisterID::X),
                0x84 | 0x94 | 0x8C => self.store_register(&ins.addressing_mode, &RegisterID::Y),
                0xAA => self.transfer_register(&RegisterID::ACC, &RegisterID::X),
                0xA8 => self.transfer_register(&RegisterID::ACC, &RegisterID::Y),
                0xBA => self.transfer_register(&RegisterID::SP, &RegisterID::X),
                0x8A => self.transfer_register(&RegisterID::X, &RegisterID::ACC),
                0x9A => self.transfer_register(&RegisterID::X, &RegisterID::SP),
                0x98 => self.transfer_register(&RegisterID::Y, &RegisterID::ACC),
                0x18 => self.clear_flag(CARRY_FLAG),
                0xD8 => self.clear_flag(DECIMAL_MODE_FLAG),
                0x58 => self.clear_flag(INTERRUPT_DISABLE_FLAG),
                0xB8 => self.clear_flag(OVERFLOW_FLAG),
                0x38 => self.set_flag(CARRY_FLAG),
                0xF8 => self.set_flag(DECIMAL_MODE_FLAG),
                0x78 => self.set_flag(INTERRUPT_DISABLE_FLAG),
                0xCA => self.decrement_register(&RegisterID::X),
                0x88 => self.decrement_register(&RegisterID::Y),
                0xE8 => self.increment_register(&RegisterID::X),
                0xC8 => self.increment_register(&RegisterID::Y),
                0xE6 | 0xF6 | 0xEE | 0xFE => self.increment_memory(&ins.addressing_mode),
                0xC6 | 0xD6 | 0xCE | 0xDE => self.decrement_memory(&ins.addressing_mode),
                0x0A => self.acc_shift_left(),
                0x4A => self.acc_shift_right(),
                0x06 | 0x16 | 0x0E | 0x1E => self.mem_shift_left(&ins.addressing_mode),
                0x46 | 0x56 | 0x4E | 0x5E => self.mem_shift_right(&ins.addressing_mode),
                0x90 => self.branch_if_flag_clear(CARRY_FLAG),
                0xB0 => self.branch_if_flag_set(CARRY_FLAG),
                0xF0 => self.branch_if_flag_set(ZERO_FLAG),
                0xD0 => self.branch_if_flag_clear(ZERO_FLAG),
                0x30 => self.branch_if_flag_set(NEGATIVE_FLAG),
                0x10 => self.branch_if_flag_clear(NEGATIVE_FLAG),
                0x70 => self.branch_if_flag_set(OVERFLOW_FLAG),
                0x50 => self.branch_if_flag_clear(OVERFLOW_FLAG),
                0x4C | 0x6C => self.jump(&ins.addressing_mode),
                0x20 => self.jump_to_subroutine(&ins.addressing_mode),
                0x60 => self.return_from_subroutine(),
                0x49 | 0x45 | 0x55 | 0x4D | 0x5D | 0x59 | 0x41 | 0x51 => self.exclusive_or(&ins.addressing_mode),
                _ => todo!("Opcode [0x{:0X}] is invalid or unimplemented", opcode)

            }

            if current_pc == self.pc {
                self.pc += (ins.bytes-1) as u16;
            }
        }
    }

    fn get_operand_address(&mut self, addressing_mode: &AddressingMode) -> u16 {

        match *addressing_mode {

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

            },
            AddressingMode::Relative => {
                let offset = self.mem_read_u8(self.pc) as i8;
                self.pc.wrapping_add_signed(offset as i16)
            }
            _ => panic!("Addressing mode {:?} instruction should not be reading an address", *addressing_mode)
        }

    }

    fn page_crossed(&self, target_addr: u16) -> bool {
        (self.pc & 0xFF00) != (target_addr & 0xFF00)
    }

    fn set_negative_and_zero_bits(&mut self, value: u8) {

        if value == 0 {
            self.status |= ZERO_FLAG;
        } else {
            self.status &= !ZERO_FLAG;
        }

        if value & 0b1000_0000 == 0b1000_0000 {
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

    fn increment_register(&mut self, target_register: &RegisterID) {

        let register_ref = match target_register {
            RegisterID::X => &mut self.x,
            RegisterID::Y => &mut self.y,
            _ => panic!("Stack pointer or accumulator should not be targets")
        };

        let data = (*register_ref).wrapping_add(1);
        *register_ref = data;
        self.set_negative_and_zero_bits(data);
    }

    fn decrement_register(&mut self, target_register: &RegisterID) {

        let register_ref = match target_register {
            RegisterID::X => &mut self.x,
            RegisterID::Y => &mut self.y,
            _ => panic!("Stack pointer or accumulator should not be targets")
        };

        let data = (*register_ref).wrapping_sub(1);
        *register_ref = data;
        self.set_negative_and_zero_bits(data);

    }

    fn increment_memory(&mut self, addressing_mode: &AddressingMode) {

        let target_addr = self.get_operand_address(addressing_mode);
        let mut data = self.mem_read_u8(target_addr);
        
        data = data.wrapping_add(1);

        self.mem_write_u8(target_addr, data);
        self.set_negative_and_zero_bits(data);
    }

    fn decrement_memory(&mut self, addressing_mode: &AddressingMode) {

        let target_addr = self.get_operand_address(addressing_mode);
        let mut data = self.mem_read_u8(target_addr);
        
        data = data.wrapping_sub(1);

        self.mem_write_u8(target_addr, data);
        self.set_negative_and_zero_bits(data);
    }

    fn exclusive_or(&mut self, addressing_mode: &AddressingMode) {

        let target_addr = self.get_operand_address(addressing_mode);
        let data = self.mem_read_u8(target_addr);

        self.acc ^= data;
        self.set_negative_and_zero_bits(self.acc);

    }

    fn stack_push_u8(&mut self, value: u8) {
        let stack_addr = 0x0100 | self.sp as u16;
        self.mem_write_u8(stack_addr, value);
        self.sp -= 1;
    }

    fn stack_pop_u8(&mut self) -> u8 {
        self.sp += 1;
        let stack_addr = 0x0100 | self.sp as u16;
        self.mem_read_u8(stack_addr)
    }

    fn stack_push_u16(&mut self, addr: u16) {

        let msb = (addr >> 8) as u8;
        let lsb = (addr & 0xFF) as u8;

        self.stack_push_u8(lsb);
        self.stack_push_u8(msb);

    }

    fn stack_pop_u16(&mut self) -> u16 {
        let msb = self.stack_pop_u8() as u16;
        let lsb = self.stack_pop_u8() as u16;
        (msb << 8) | lsb
    }

    fn jump(&mut self, addressing_mode: &AddressingMode) {
        let target_addr = self.get_operand_address(addressing_mode);
        self.pc = target_addr;

        // TODO: There is a bug in the 6502 that involves getting the address on a page boundary

    }

    fn jump_to_subroutine(&mut self, addressing_mode: &AddressingMode) {

        // We want to return to the instruction AFTER this
        // because otherwise we'll just come back to the
        // JSR instruction and loop
        // We're doing +2 (because we read 2 bytes after the instruction)
        // and -1 because we want to store the target return-1
        let return_addr = self.pc + 1;
        self.stack_push_u16(return_addr);

        let target_addr = self.get_operand_address(addressing_mode);
        self.pc = target_addr;

    }

    fn return_from_subroutine(&mut self) {
        let target_addr = self.stack_pop_u16();
        self.pc = target_addr + 1;
    }

    // ! Really wanted to do a guardian clause instead, but tarpaulin wasn't covering the early return
    fn branch_if_flag_set(&mut self, flag_alias: u8) {

        if self.status & flag_alias == flag_alias {

            let target_addr = self.get_operand_address(&AddressingMode::Relative);

            if self.page_crossed(target_addr) {
                // TODO: When cycles are implemented
                // TODO: Logger
                println!("Page was crossed! Current page: 0x{:0X} New page: 0x{:0X}", (self.pc & 0xFF00) >> 8, (target_addr & 0xFF00) >> 8);

            }

            self.pc = target_addr;
        }

    }

    // ! Really wanted to do a guardian clause instead, but tarpaulin wasn't covering the early return
    fn branch_if_flag_clear(&mut self, flag_alias: u8) {

        if self.status & flag_alias != flag_alias {
            
            let target_addr = self.get_operand_address(&AddressingMode::Relative);

            if self.page_crossed(target_addr) {
                // TODO: When cycles are implemented
                // TODO: Logger
                println!("Page was crossed! Current page: 0x{:0X} New page: 0x{:0X}", (self.pc & 0xFF00) >> 8, (target_addr & 0xFF00) >> 8);

            }

            self.pc = target_addr;

        }

    }

    fn add_with_carry(&mut self, addressing_mode: &AddressingMode) {

        let address = self.get_operand_address(addressing_mode);
        let data = self.mem_read_u8(address);
        self.acc = self.acc.wrapping_add(data);

        self.set_negative_and_zero_bits(self.acc);

        if self.acc <= data {
            self.set_flag(CARRY_FLAG);
        }

        // TODO: ? Set overflow flag if sign bit is incorrect???

    }

    fn acc_shift_left(&mut self) {
        
        if self.acc & 0b1000_0000 > 0 {
            self.set_flag(CARRY_FLAG);
        }

        self.acc <<= 1;
        self.set_negative_and_zero_bits(self.acc);

    }

    fn acc_shift_right(&mut self) {

        if self.acc & 1 == 1 {
            self.set_flag(CARRY_FLAG);
        }

        self.acc >>= 1;
        self.set_negative_and_zero_bits(self.acc);

    }

    fn mem_shift_left(&mut self, addressing_mode: &AddressingMode) {

        let address = self.get_operand_address(addressing_mode);
        let mut data = self.mem_read_u8(address);
        
        if data & 0b1000_0000 > 0 {
            self.set_flag(CARRY_FLAG);
        }

        data <<= 1;

        self.set_negative_and_zero_bits(data);
        self.mem_write_u8(address, data);

    }

    fn mem_shift_right(&mut self, addressing_mode: &AddressingMode) {

        let address = self.get_operand_address(addressing_mode);
        let mut data = self.mem_read_u8(address);
        
        if data & 1 == 1 {
            self.set_flag(CARRY_FLAG);
        }

        data >>= 1;

        self.set_negative_and_zero_bits(data);
        self.mem_write_u8(address, data);

    }

    fn clear_flag(&mut self, flag_alias: u8) {
        self.status &= !flag_alias;
    }

    fn set_flag(&mut self, flag_alias: u8) {
        self.status |= flag_alias;
    }

    fn load_register(&mut self, addressing_mode: &AddressingMode, target_register: &RegisterID) {

        let address = self.get_operand_address(addressing_mode);
        let data = self.mem_read_u8(address);
        let register_ref = match target_register {
            RegisterID::ACC => &mut self.acc,
            RegisterID::X => &mut self.x,
            RegisterID::Y => &mut self.y,
            _ => panic!("Stack pointer should not be a target for loading")
        };
        
        *register_ref = data;
        self.set_negative_and_zero_bits(data);

    }

    fn store_register(&mut self, addressing_mode: &AddressingMode, target_register: &RegisterID) {

        let register_value = match target_register {
            RegisterID::ACC => self.acc,
            RegisterID::X => self.x,
            RegisterID::Y => self.y,
            _ => panic!("Stack pointer should not be a target for storing")
        };

        let address = self.get_operand_address(addressing_mode);
        self.mem_write_u8(address, register_value);

    }

    fn transfer_register(&mut self, source_register: &RegisterID, target_register: &RegisterID) {

        let source_value = match source_register {
            RegisterID::ACC => self.acc,
            RegisterID::X => self.x,
            RegisterID::Y => self.y,
            RegisterID::SP => self.sp
        };

        let target_ref = match target_register {
            RegisterID::ACC => &mut self.acc,
            RegisterID::X => &mut self.x,
            RegisterID::Y => &mut self.y,
            RegisterID::SP => &mut self.sp
        };

        *target_ref = source_value;

        if *target_register != RegisterID::SP {
            self.set_negative_and_zero_bits(source_value);
        }

    }

    fn and(&mut self, addressing_mode: &AddressingMode) {

        let address = self.get_operand_address(addressing_mode);
        let data = self.mem_read_u8(address);
        self.acc &= data;
        self.set_negative_and_zero_bits(self.acc);

    }

    fn bit(&mut self, addressing_mode: &AddressingMode) {

        let address = self.get_operand_address(addressing_mode);
        let data = self.mem_read_u8(address);
        let result = data & self.acc;
        self.set_negative_and_zero_bits(result);

        if result & OVERFLOW_FLAG > 0 {
            self.status |= OVERFLOW_FLAG;
        }

    }

    fn compare_register(&mut self, addressing_mode: &AddressingMode, target_register: &RegisterID) {

        let address = self.get_operand_address(addressing_mode);
        let data = self.mem_read_u8(address);
        let register_value = match target_register {
            RegisterID::ACC => self.acc,
            RegisterID::X => self.x,
            RegisterID::Y => self.y,
            _ => panic!("Stack pointer should not be a target for comparing")
        };

        let result = register_value - data;

        if register_value == data {
            self.set_flag(ZERO_FLAG);
        }

        if register_value >= data {
            self.set_flag(CARRY_FLAG);
        }

        if result & NEGATIVE_FLAG > 0 {
            self.set_flag(NEGATIVE_FLAG);
        }

    }

}

#[cfg(test)]
mod tests {

    use std::vec;
    use super::*;

    #[test]
    fn test_cpu_default() {
        
        let cpu = CPU::default();

        assert_eq!(cpu.pc, 0x8000);
        assert_eq!(cpu.sp, 0xFF);
        assert_eq!(cpu.acc, 0);
        assert_eq!(cpu.x, 0);
        assert_eq!(cpu.y, 0);
        assert_eq!(cpu.status, 0);

    }

    #[test]
    fn test_cpu_init() {

        let cpu = CPU::new();

        assert_eq!(cpu.pc, 0x8000);
        assert_eq!(cpu.sp, 0xFF);
        assert_eq!(cpu.acc, 0);
        assert_eq!(cpu.x, 0);
        assert_eq!(cpu.y, 0);
        assert_eq!(cpu.status, 0);

    }

    #[test]
    fn test_cpu_reset() {

        let mut cpu = CPU::new();
        cpu.mem_write_u16(0xFFFC, 0x8000);

        cpu.acc = 52;
        cpu.sp = 124;
        cpu.pc = 1892;
        cpu.x = 15;
        cpu.y = 16;
        cpu.status = 0b10010000;

        cpu.reset();

        assert_eq!(cpu.pc, 0x8000);
        assert_eq!(cpu.sp, 0xFF);
        assert_eq!(cpu.acc, 0);
        assert_eq!(cpu.x, 0);
        assert_eq!(cpu.y, 0);
        assert_eq!(cpu.status, 0);

    }

    #[test]
    fn test_mem_page_crossed() {

        let mut cpu = CPU::new();

        cpu.pc = 0xFF;

        assert!(!cpu.page_crossed(0xFE));
        assert!(cpu.page_crossed(0x100));

    }

    #[test]
    fn test_set_negative_and_zero_flags() {

        let mut cpu = CPU::new();

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

        let mut cpu = CPU::new();
        cpu.x = 0xFE;

        cpu.increment_register(&RegisterID::X);
        assert_eq!(cpu.x, 0xFF);

        cpu.increment_register(&RegisterID::X);
        assert_eq!(cpu.x, 0);

    }

    #[test]
    fn test_decrement_register () {

        let mut cpu = CPU::new();
        cpu.x = 1;

        cpu.decrement_register(&RegisterID::X);
        assert_eq!(cpu.x, 0);

        cpu.decrement_register(&RegisterID::X);
        assert_eq!(cpu.x, 255);

    }

    #[test]
    fn test_mem_read_u8 () {

        let mut cpu = CPU::new();
        cpu.memory[162] = 0xAF;

        assert_eq!(cpu.mem_read_u8(162), 0xAF);

    }

    #[test]
    fn test_mem_read_u16 () {

        let mut cpu = CPU::new();
        cpu.memory[162] = 0x80;
        cpu.memory[163] = 0x08;

        assert_eq!(cpu.mem_read_u16(162), 0x0880);

    }

    #[test]
    fn test_mem_write_u8 () {

        let mut cpu = CPU::new();
        let data: u8 = 0x12;
        cpu.mem_write_u8(162, data);

        assert_eq!(cpu.memory[162], 0x12);

    }

    #[test]
    fn test_mem_write_u16 () {

        let mut cpu = CPU::new();
        let data: u16 = 0x1234;
        cpu.mem_write_u16(162, data);

        assert_eq!(cpu.memory[162], 0x34);
        assert_eq!(cpu.memory[163], 0x12);

    }

    #[test]
    fn test_clear_flag() {

        let mut cpu = CPU::new();
        cpu.status = 0b1111_1111;

        cpu.clear_flag(ZERO_FLAG);
        assert_eq!(cpu.status, 0b1111_1101);

    }

    #[test]
    fn test_get_operand_address_immediate() {

        let mut cpu = CPU::new();
        cpu.acc = 0x10;
        cpu.x = 0x11;
        cpu.y = 0x12;
        cpu.sp = 0x13;
        cpu.pc = 0xF0;

        assert_eq!(cpu.get_operand_address(&AddressingMode::Immediate), 0xF0);

    }

    #[test]
    fn test_get_operand_address_absolute() {

        let mut cpu = CPU::new();
        cpu.acc = 0x10;
        cpu.x = 0x11;
        cpu.y = 0x12;
        cpu.sp = 0x13;
        cpu.pc = 0xF0;

        cpu.memory[0xF0] = 0x88;
        cpu.memory[0xF1] = 0x80;

        // Absolute addressing
        assert_eq!(cpu.get_operand_address(&AddressingMode::Absolute), 0x8088);
        assert_eq!(cpu.get_operand_address(&AddressingMode::AbsoluteX), 0x8099);
        assert_eq!(cpu.get_operand_address(&AddressingMode::AbsoluteY), 0x809A);

        cpu.memory[0xF0] = 0xF0;
        cpu.memory[0xF1] = 0xFF;

        // Absolute addressing wrap around
        assert_eq!(cpu.get_operand_address(&AddressingMode::AbsoluteX), 0x01);
        assert_eq!(cpu.get_operand_address(&AddressingMode::AbsoluteY), 0x02);

    }

    #[test]
    fn test_get_operand_address_zero_page() {

        let mut cpu = CPU::new();
        cpu.acc = 0x10;
        cpu.x = 0x11;
        cpu.y = 0x12;
        cpu.sp = 0x13;
        cpu.pc = 0xF0;

        cpu.memory[0xF0] = 0x88;
        cpu.memory[0xF1] = 0x80;

        // Zero page addressing
        assert_eq!(cpu.get_operand_address(&AddressingMode::ZeroPage), 0x88);
        assert_eq!(cpu.get_operand_address(&AddressingMode::ZeroPageX), 0x99);
        assert_eq!(cpu.get_operand_address(&AddressingMode::ZeroPageY), 0x9A);

        cpu.memory[0xF0] = 0xF0;
        cpu.memory[0xF1] = 0xFF;

        // Zero page addressing wrap around
        assert_eq!(cpu.get_operand_address(&AddressingMode::ZeroPageX), 0x01);
        assert_eq!(cpu.get_operand_address(&AddressingMode::ZeroPageY), 0x02);

    }

    #[test]
    fn test_get_operand_address_indirect() {

        let mut cpu = CPU::new();
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
        println!("Indirect X: {}", cpu.get_operand_address(&AddressingMode::IndirectX));
        assert_eq!(cpu.get_operand_address(&AddressingMode::Indirect), 0x1234);
        assert_eq!(cpu.get_operand_address(&AddressingMode::IndirectX), 0x5678);
        assert_eq!(cpu.get_operand_address(&AddressingMode::IndirectY), 0x679B);

    }

    #[test]
    fn test_get_operand_address_relative() {

        let mut cpu = CPU::new();
        cpu.acc = 0x10;
        cpu.x = 0x11;
        cpu.y = 0x12;
        cpu.sp = 0x13;
        cpu.pc = 0xF0;

        cpu.memory[0xF0] = 0b0000_0001;
        cpu.memory[0xF1] = 0x80;
        cpu.memory[0x8088] = 0x34;
        cpu.memory[0x8089] = 0x12;
        cpu.memory[0x88] = 0x89;
        cpu.memory[0x89] = 0x67;
        cpu.memory[0x99] = 0x78;
        cpu.memory[0x9A] = 0x56;

        println!("Indirect X: {}", cpu.get_operand_address(&AddressingMode::Relative));
        assert_eq!(cpu.get_operand_address(&AddressingMode::Relative), 0xF1);

        cpu.memory[0xF0] = 0b1111_1100;
        assert_eq!(cpu.get_operand_address(&AddressingMode::Relative), 0b1110_1100);

    }

    #[test]
    #[should_panic]
    fn test_get_operand_address_implied_panics() {
        let mut cpu = CPU::new();
        cpu.get_operand_address(&AddressingMode::Implied);
    }

    #[test]
    #[should_panic]
    fn test_load_register_sp_panics() {
        let mut cpu = CPU::new();
        cpu.load_register(&AddressingMode::Immediate, &RegisterID::SP);
    }

    #[test]
    #[should_panic]
    fn test_store_register_sp_panics() {
        let mut cpu = CPU::new();
        cpu.store_register(&AddressingMode::Immediate, &RegisterID::SP);
    }

    #[test]
    #[should_panic]
    fn test_compare_register_sp_panics() {
        let mut cpu = CPU::new();
        cpu.compare_register(&AddressingMode::Immediate, &RegisterID::SP);
    }

    #[test]
    #[should_panic]
    fn test_increment_register_panics() {
        let mut cpu = CPU::new();
        cpu.increment_register(&RegisterID::SP);
    }

    #[test]
    #[should_panic]
    fn test_decrement_register_panics() {
        let mut cpu = CPU::new();
        cpu.decrement_register(&RegisterID::SP);
    }

    #[test]
    fn test_adc() {

        let mut cpu = CPU::new();
        let program = vec![0xA9, 0xF0, 0x69, 0x0F, 0x00];
        cpu.load_and_run(program);

        assert_eq!(cpu.acc, 0xFF);

        let program = vec![0xA9, 0xF0, 0x69, 0x10, 0x00];
        cpu.load_and_run(program);

        assert_eq!(cpu.status & CARRY_FLAG, CARRY_FLAG)

    }

    #[test]
    fn test_and() {
        
        let mut cpu = CPU::new();
        let program = vec![0xA9, 0b1010_1010, 0x29, 0b1111_0000, 0x00];
        cpu.load_and_run(program);

        assert_eq!(cpu.acc, 0b1010_0000);

    }

    #[test]
    fn test_asl_acc() {
        
        let mut cpu = CPU::new();
        let program = vec![0xA9, 0b0101_0101, 0x0A, 0x00];
        cpu.load_and_run(program);

        assert_eq!(cpu.acc, 0b1010_1010);
        assert_eq!(cpu.status & CARRY_FLAG, 0);

        let program = vec![0xA9, 0b1010_1010, 0x0A, 0x00];
        cpu.load_and_run(program);

        assert_eq!(cpu.acc, 0b0101_0100);
        assert_eq!(cpu.status & CARRY_FLAG, CARRY_FLAG);

    }

    #[test]
    fn test_asl_mem() {
        
        let mut cpu = CPU::new();
        let program = vec![0xA9, 0b0101_0101, 0x0E, 0x01, 0x80, 0xAD, 0x01, 0x80, 0x00];
        cpu.load_and_run(program);

        assert_eq!(cpu.acc, 0b1010_1010);
        assert_eq!(cpu.status & CARRY_FLAG, 0);

        let program = vec![0xA9, 0b1010_1010, 0x0E, 0x01, 0x80, 0xAD, 0x01, 0x80, 0x00];
        cpu.load_and_run(program);

        assert_eq!(cpu.acc, 0b0101_0100);
        assert_eq!(cpu.status & CARRY_FLAG, CARRY_FLAG);

    }

    #[test]
    fn test_bcc() {

        let mut cpu = CPU::new();

        // Branch condition is met
        let program = vec![0x90, 0b1111_1101];
        cpu.load_and_run(program);

        assert_eq!(cpu.pc, 0x7FFF);

        // Branch condition is NOT met
        let program = vec![0x90, 0b1111_1101];
        cpu.load(program);
        cpu.set_flag(CARRY_FLAG);
        cpu.run();

        assert_eq!(cpu.pc, 0x8000);

    }

    #[test]
    fn test_bcs() {

        let mut cpu = CPU::new();

        // Branch condition is met
        let program = vec![0xB0, 0b1111_1101];

        cpu.load(program);
        cpu.set_flag(CARRY_FLAG);
        cpu.run();

        assert_eq!(cpu.pc, 0x7FFF);

        // Branch condition is NOT met
        let program = vec![0xB0, 0b1111_1101];
        cpu.load(program);
        cpu.clear_flag(CARRY_FLAG);
        cpu.run();

        assert_eq!(cpu.pc, 0x8000);

    }

    #[test]
    fn test_beq() {

        let mut cpu = CPU::new();

        // Branch condition is met
        let program = vec![0xF0, 0b1111_1101];

        cpu.load(program);
        cpu.set_flag(ZERO_FLAG);
        cpu.run();

        assert_eq!(cpu.pc, 0x7FFF);

        // Branch condition is NOT met
        let program = vec![0xF0, 0b1111_1101];
        cpu.load(program);
        cpu.clear_flag(ZERO_FLAG);
        cpu.run();

        assert_eq!(cpu.pc, 0x8000);

    }

    #[test]
    fn test_bne() {

        let mut cpu = CPU::new();

        // Branch condition is met
        let program = vec![0xD0, 0b1111_1101];

        cpu.load(program);
        cpu.run();

        assert_eq!(cpu.pc, 0x7FFF);

        // Branch condition is NOT met
        let program = vec![0xD0, 0b1111_1101];
        cpu.load(program);
        cpu.set_flag(ZERO_FLAG);
        cpu.run();

        assert_eq!(cpu.pc, 0x8000);

    }

    #[test]
    fn test_bmi() {

        let mut cpu = CPU::new();

        // Branch condition is met
        let program = vec![0x30, 0b1111_1101];

        cpu.load(program);
        cpu.set_flag(NEGATIVE_FLAG);
        cpu.run();

        assert_eq!(cpu.pc, 0x7FFF);

        // Branch condition is NOT met
        let program = vec![0x30, 0b1111_1101];
        cpu.load(program);
        cpu.clear_flag(NEGATIVE_FLAG);
        cpu.run();

        assert_eq!(cpu.pc, 0x8000);

    }

    #[test]
    fn test_bpl() {

        let mut cpu = CPU::new();

        // Branch condition is met
        let program = vec![0x10, 0b1111_1101];

        cpu.load(program);
        cpu.run();

        assert_eq!(cpu.pc, 0x7FFF);

        // Branch condition is NOT met
        let program = vec![0x10, 0b1111_1101];
        cpu.load(program);
        cpu.set_flag(ZERO_FLAG);
        cpu.run();

        assert_eq!(cpu.pc, 0x8000);

    }

    #[test]
    fn test_bvc() {

        let mut cpu = CPU::new();

        // Branch condition is met
        let program = vec![0x50, 0b1111_1101];

        cpu.load(program);
        cpu.run();

        assert_eq!(cpu.pc, 0x7FFF);

        // Branch condition is NOT met
        let program = vec![0x50, 0b1111_1101];
        cpu.load(program);
        cpu.set_flag(OVERFLOW_FLAG);
        cpu.run();

        assert_eq!(cpu.pc, 0x8000);

    }

    #[test]
    fn test_bvs() {

        let mut cpu = CPU::new();

        // Branch condition is met
        let program = vec![0x70, 0b1111_1101];

        cpu.load(program);
        cpu.set_flag(OVERFLOW_FLAG);
        cpu.run();

        assert_eq!(cpu.pc, 0x7FFF);

        // Branch condition is NOT met
        let program = vec![0x70, 0b1111_1101];
        cpu.load(program);
        cpu.clear_flag(OVERFLOW_FLAG);
        cpu.run();

        assert_eq!(cpu.pc, 0x8000);

    }

    #[test]
    fn test_bit() {

        let mut cpu = CPU::new();
        let program = vec![0xA9, 0xF0, 0x2C, 0x06, 0x80, 0x00, 0b1110_0000];
        cpu.load_and_run(program);

        assert!(cpu.status & NEGATIVE_FLAG > 0);
        assert!(cpu.status & OVERFLOW_FLAG > 0);
        assert_eq!(cpu.acc, 0xF0);

    }

    #[test]
    fn test_cmp() {

        let mut cpu = CPU::new();
        let program = vec![0xA9, 0xF0, 0xC9, 0xF0, 0x00];
        cpu.load_and_run(program);

        assert!(cpu.status & NEGATIVE_FLAG > 0);
        assert!(cpu.status & ZERO_FLAG > 0);
        assert!(cpu.status & CARRY_FLAG > 0);

        let program = vec![0xA9, 0xF0, 0xC9, 0x00, 0x00];
        cpu.load_and_run(program);

        assert!(cpu.status & NEGATIVE_FLAG > 0);
        assert!(cpu.status & ZERO_FLAG == 0);
        assert!(cpu.status & CARRY_FLAG > 0);

    }

    #[test]
    fn test_cpx() {

        let mut cpu = CPU::new();
        let program = vec![0xA2, 0xF0, 0xE0, 0xF0, 0x00];
        cpu.load_and_run(program);

        assert!(cpu.status & NEGATIVE_FLAG > 0);
        assert!(cpu.status & ZERO_FLAG > 0);
        assert!(cpu.status & CARRY_FLAG > 0);

        let program = vec![0xA2, 0xF0, 0xE0, 0x00, 0x00];
        cpu.load_and_run(program);

        assert!(cpu.status & NEGATIVE_FLAG > 0);
        assert!(cpu.status & ZERO_FLAG == 0);
        assert!(cpu.status & CARRY_FLAG > 0);

    }

    #[test]
    fn test_cpy() {

        let mut cpu = CPU::new();
        let program = vec![0xA0, 0xF0, 0xC0, 0xF0, 0x00];
        cpu.load_and_run(program);

        assert!(cpu.status & NEGATIVE_FLAG > 0);
        assert!(cpu.status & ZERO_FLAG > 0);
        assert!(cpu.status & CARRY_FLAG > 0);

        let program = vec![0xA0, 0xF0, 0xC0, 0x00, 0x00];
        cpu.load_and_run(program);

        assert!(cpu.status & NEGATIVE_FLAG > 0);
        assert!(cpu.status & ZERO_FLAG == 0);
        assert!(cpu.status & CARRY_FLAG > 0);

    }

    #[test]
    fn test_dec() {

        let mut cpu = CPU::new();
        let program = vec![0xCE, 0x04, 0x80, 0x00, 0b1111_1111];
        cpu.load_and_run(program);

        assert_eq!(cpu.memory[0x8004], 0b1111_1110);
        assert_eq!(cpu.status & NEGATIVE_FLAG, NEGATIVE_FLAG);

    }

    #[test]
    fn test_eor() {

        let mut cpu = CPU::new();
        let program = vec![0xA9, 0b1111_1111, 0x49, 0b0101_0101];
        cpu.load_and_run(program);

        assert_eq!(cpu.acc, 0b1010_1010);
        assert_eq!(cpu.status & NEGATIVE_FLAG, NEGATIVE_FLAG);

    }

    #[test]
    fn test_inc() {

        let mut cpu = CPU::new();
        let program = vec![0xEE, 0x04, 0x80, 0x00, 0b1111_1111];
        cpu.load_and_run(program);

        assert_eq!(cpu.memory[0x8004], 0x00);
        assert_eq!(cpu.status & ZERO_FLAG, ZERO_FLAG);

    }

    #[test]
    fn test_jmp() {

        let mut cpu = CPU::new();
        let program = vec![0x4C, 0xEF, 0xFE];
        cpu.load_and_run(program);

        // The reason that it's 0xFEF0 is because
        // we jump to 0xFEEF and then read the next
        // instruction which is BRK so the final state
        // is 0xFEEF + 1
        assert_eq!(cpu.pc, 0xFEF0);

    }

    #[test]
    fn test_jsr() {

        let mut cpu = CPU::new();
        let program = vec![0x20, 0xEF, 0xFE];
        cpu.load_and_run(program);

        // The reason that it's 0xFEF0 is because
        // we jump to 0xFEEF and then read the next
        // instruction which is BRK so the final state
        // is 0xFEEF + 1
        assert_eq!(cpu.pc, 0xFEF0);
        assert_eq!(cpu.sp, 0xFD);
        assert_eq!(cpu.stack_pop_u16(), 0x8002);

    }

    #[test]
    fn test_lda_immediate() {

        let mut cpu = CPU::new();

        // Negative bit is set
        let program = vec![0xA9, 156, 0x00];
        cpu.load_and_run(program);

        assert_eq!(cpu.acc, 156);

    }

    #[test]
    fn test_lda_zero_page() {

        let mut cpu = CPU::new();

        let program = vec![0xA5, 0x04, 0x00];
        cpu.load(program);
        cpu.memory[0x04] = 0x13;
        cpu.run();

        assert_eq!(cpu.acc, 0x13);

    }

    #[test]
    fn test_lda_zero_page_x() {

        let mut cpu = CPU::new();

        let program = vec![0xA9, 0xFA, 0xAA, 0xB5, 0x0A, 0x00];
        cpu.load(program);
        cpu.memory[0x04] = 0x13;
        cpu.run();

        assert_eq!(cpu.acc, 0x13);

    }

    #[test]
    fn test_lda_absolute() {

        let mut cpu = CPU::new();

        let program = vec![0xAD, 0x04, 0x80, 0x00, 0x13];
        cpu.load_and_run(program);

        assert_eq!(cpu.acc, 0x13);

    }

    #[test]
    fn test_lda_absolute_x() {

        let mut cpu = CPU::new();

        let program = vec![0xA9, 0x04, 0xAA, 0xBD, 0x03, 0x80, 0x00, 0x13];
        cpu.load_and_run(program);

        assert_eq!(cpu.acc, 0x13);

    }

    #[test]
    fn test_lda_absolute_y() {

        let mut cpu = CPU::new();

        let program = vec![0xA9, 0x04, 0xA8, 0xB9, 0x03, 0x80, 0x00, 0x13];
        cpu.load_and_run(program);

        assert_eq!(cpu.acc, 0x13);

    }

    #[test]
    fn test_lda_indirect_x() {

        let mut cpu = CPU::new();

        let program = vec![0xA9, 0x10, 0xAA, 0xA1, 0xEF, 0x00];
        cpu.load(program);
        cpu.memory[0xFF] = 0x01;
        cpu.memory[0x100] = 0x00;
        cpu.memory[0x01] = 0x13;
        cpu.run();

        assert_eq!(cpu.acc, 0x13);

    }

    #[test]
    fn test_lda_indirect_y() {

        let mut cpu = CPU::new();

        let program = vec![0xA9, 0x10, 0xA8, 0xB1, 0xEF, 0x00];
        cpu.load(program);
        cpu.memory[0xEF] = 0x01;
        cpu.memory[0xF0] = 0x00;
        cpu.memory[0x11] = 0x13;
        cpu.run();

        assert_eq!(cpu.acc, 0x13);

    }

    #[test]
    fn test_ldx() {

        let mut cpu = CPU::new();
        let program = vec![0xA2, 0xFF, 0x00];
        cpu.load_and_run(program);

        assert_eq!(cpu.x, 0xFF);
        assert_eq!(cpu.status & NEGATIVE_FLAG, NEGATIVE_FLAG);
        assert_eq!(cpu.status & ZERO_FLAG, 0);

    }

    #[test]
    fn test_ldy() {

        let mut cpu = CPU::new();
        let program = vec![0xA0, 0xFF, 0x00];
        cpu.load_and_run(program);

        assert_eq!(cpu.y, 0xFF);
        assert_eq!(cpu.status & NEGATIVE_FLAG, NEGATIVE_FLAG);
        assert_eq!(cpu.status & ZERO_FLAG, 0);

    }

    #[test]
    fn test_lsr_acc() {
        
        let mut cpu = CPU::new();
        let program = vec![0xA9, 0b0101_0101, 0x4A, 0x00];
        cpu.load_and_run(program);

        assert_eq!(cpu.acc, 0b0010_1010);
        assert_eq!(cpu.status & CARRY_FLAG, CARRY_FLAG);

        let program = vec![0xA9, 0b1010_1010, 0x4A, 0x00];
        cpu.load_and_run(program);

        assert_eq!(cpu.acc, 0b0101_0101);
        assert_eq!(cpu.status & CARRY_FLAG, 0);

    }

    #[test]
    fn test_lsr_mem() {
        
        let mut cpu = CPU::new();
        let program = vec![0xA9, 0b0101_0101, 0x4E, 0x01, 0x80, 0xAD, 0x01, 0x80, 0x00];
        cpu.load_and_run(program);

        assert_eq!(cpu.acc, 0b0010_1010);
        assert_eq!(cpu.status & CARRY_FLAG, CARRY_FLAG);

        let program = vec![0xA9, 0b1010_1010, 0x4E, 0x01, 0x80, 0xAD, 0x01, 0x80, 0x00];
        cpu.load_and_run(program);

        assert_eq!(cpu.acc, 0b0101_0101);
        assert_eq!(cpu.status & CARRY_FLAG, 0);

    }

    #[test]
    fn test_nop() {

        let mut cpu = CPU::new();
        let program = vec![0xEA, 0x00];
        cpu.load_and_run(program);

        assert_eq!(cpu.pc, 0x8002)

    }

    #[test]
    fn test_run_sample_prog_1() {

        /*
            This program does the following:
            Load 0xC0 into the accumulator
            Transfer to the X register
            Increment X
         */

        let mut cpu = CPU::new();
        let program = vec![0xA9, 0xC0, 0xAA, 0xE8, 0x00];

        cpu.load_and_run(program);

        assert_eq!(cpu.acc, 0xC0);
        assert_eq!(cpu.x, 0xC1);

    }

    #[test]
    fn test_rts() {

        let mut cpu = CPU::new();
        let program = vec![0x60, 0xEF, 0xFE];
        cpu.load(program);
        cpu.stack_push_u16(0x8002);
        cpu.run();

        assert_eq!(cpu.pc, 0x8004);
        assert_eq!(cpu.sp, 0xFF); // Stack should be empty now

    }

    #[test]
    fn test_sec() {

        let mut cpu = CPU::new();
        let program = vec![0x38];
        cpu.load_and_run(program);

        assert_eq!(cpu.status & CARRY_FLAG, CARRY_FLAG)

    }

    #[test]
    fn test_sed() {

        let mut cpu = CPU::new();
        let program = vec![0xF8];
        cpu.load_and_run(program);
        
        assert_eq!(cpu.status & DECIMAL_MODE_FLAG, DECIMAL_MODE_FLAG)

    }

    #[test]
    fn test_sei() {

        let mut cpu = CPU::new();
        let program = vec![0x78];
        cpu.load_and_run(program);
        
        assert_eq!(cpu.status & INTERRUPT_DISABLE_FLAG, INTERRUPT_DISABLE_FLAG)

    }

    #[test]
    fn test_sta() {

        let mut cpu = CPU::new();
        let program = vec![0xA9, 0x13, 0x8D, 0xFF, 0x80];
        cpu.load_and_run(program);
    
        assert_eq!(cpu.memory[0x80FF], 0x13);

    }

    #[test]
    fn test_stx() {

        let mut cpu = CPU::new();
        let program = vec![0xA2, 0x13, 0x8E, 0xFF, 0x80];
        cpu.load_and_run(program);
    
        assert_eq!(cpu.memory[0x80FF], 0x13);

    }

    #[test]
    fn test_sty() {

        let mut cpu = CPU::new();
        let program = vec![0xA0, 0x13, 0x8C, 0xFF, 0x80];
        cpu.load_and_run(program);
    
        assert_eq!(cpu.memory[0x80FF], 0x13);

    }

    #[test]
    fn test_tax () {

        let mut cpu = CPU::new();
        cpu.acc = 156;

        let program = vec![0xAA, 0x00];
        cpu.load(program);
        cpu.run();

        assert_eq!(cpu.x, 156);
        assert!(cpu.status & NEGATIVE_FLAG > 0);

    }

    #[test]
    fn test_tay () {

        let mut cpu = CPU::new();
        cpu.acc = 156;

        let program = vec![0xA8, 0x00];
        cpu.load(program);
        cpu.run();

        assert_eq!(cpu.y, 156);
        assert!(cpu.status & NEGATIVE_FLAG > 0);

    }

    #[test]
    fn test_tsx () {

        let mut cpu = CPU::new();
        cpu.sp = 156;

        let program = vec![0xBA, 0x00];
        cpu.load(program);
        cpu.run();

        assert_eq!(cpu.x, 156);
        assert!(cpu.status & NEGATIVE_FLAG > 0);

    }

    #[test]
    fn test_txa () {

        let mut cpu = CPU::new();
        cpu.x = 156;

        let program = vec![0x8A, 0x00];
        cpu.load(program);
        cpu.run();

        assert_eq!(cpu.acc, 156);
        assert!(cpu.status & NEGATIVE_FLAG > 0);

    }

    #[test]
    fn test_txs () {

        let mut cpu = CPU::new();
        cpu.x = 156;

        let program = vec![0x9A, 0x00];
        cpu.load(program);
        cpu.run();

        assert_eq!(cpu.sp, 156);

    }

    #[test]
    fn test_tya () {

        let mut cpu = CPU::new();
        cpu.y = 156;

        let program = vec![0x98, 0x00];
        cpu.load(program);
        cpu.run();

        assert_eq!(cpu.acc, 156);
        assert!(cpu.status & NEGATIVE_FLAG > 0);

    }

    #[test]
    fn test_inx () {

        let mut cpu = CPU::new();
        cpu.x = 127;
        cpu.set_negative_and_zero_bits(cpu.x);
        assert_eq!(cpu.status & NEGATIVE_FLAG, 0);

        let program = vec![0xE8, 0x00];
        cpu.load(program);
        cpu.run();

        assert_eq!(cpu.x, 128);
        assert_eq!(cpu.status & NEGATIVE_FLAG, NEGATIVE_FLAG);

    }

    #[test]
    fn test_iny () {

        let mut cpu = CPU::new();
        cpu.y = 127;
        cpu.set_negative_and_zero_bits(cpu.y);
        assert_eq!(cpu.status & NEGATIVE_FLAG, 0);

        let program = vec![0xC8, 0x00];
        cpu.load(program);
        cpu.run();

        assert_eq!(cpu.y, 128);
        assert_eq!(cpu.status & NEGATIVE_FLAG, NEGATIVE_FLAG);

    }

    #[test]
    fn test_dex () {

        let mut cpu = CPU::new();
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

        let mut cpu = CPU::new();
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

        let mut cpu = CPU::new();
        cpu.status = 0b1111_1111;

        let program = vec![0x18, 0x00];
        cpu.load(program);
        cpu.status = 0b1111_1111;
        cpu.run();

        assert_eq!(cpu.status, !CARRY_FLAG);

    }

    #[test]
    fn test_cld() {

        let mut cpu = CPU::new();
        cpu.status = 0b1111_1111;

        let program = vec![0xD8, 0x00];
        cpu.load(program);
        cpu.status = 0b1111_1111;
        cpu.run();

        assert_eq!(cpu.status, !DECIMAL_MODE_FLAG);

    }

    #[test]
    fn test_cli() {

        let mut cpu = CPU::new();
        cpu.status = 0b1111_1111;

        let program = vec![0x58, 0x00];
        cpu.load(program);
        cpu.status = 0b1111_1111;
        cpu.run();

        assert_eq!(cpu.status, !INTERRUPT_DISABLE_FLAG);

    }

    #[test]
    fn test_clv() {

        let mut cpu = CPU::new();

        let program = vec![0xB8, 0x00];
        cpu.load(program);
        cpu.status = 0b1111_1111;
        cpu.run();

        assert_eq!(cpu.status, !OVERFLOW_FLAG);

    }

}