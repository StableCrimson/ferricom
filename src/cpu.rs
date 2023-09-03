use crate::instructions::{self};
use crate::bus::Bus;

use bitflags::bitflags;

bitflags! {

    /// Aliases for the flags in the 6502 status register.
    /// More information on these flags can be found here: <https://www.nesdev.org/wiki/Status_flags>
    #[derive(Clone, Copy, Debug)]
    pub struct CPUFlags: u8 {
        const CARRY =              0b0000_0001;
        const ZERO =               0b0000_0010;
        const INTERRUPT_DISABLE =  0b0000_0100;
        const DECIMAL_MODE =       0b0000_1000;

        /*
            Bits 4 and 5 are somewhat unused.
            They are used to represent any of 4 interrupt status types
        */
        const BREAK_COMMAND_4 =    0b0001_0000;
        const BREAK_COMMAND_5 =    0b0010_0000;

        const OVERFLOW =           0b0100_0000;
        const NEGATIVE =           0b1000_0000;
    }
}

/// For instructions that perform the same operation
/// but on different registers (Ex: `CMP`, `CPX`, `CPY`)
/// Makes things more concise because we can have one general function
/// that we just pass the registers into, instead of having a function
/// for each unique instruction.
#[derive(PartialEq)]
pub enum RegisterID {
    ACC,
    X,
    Y,
    SP
}

/// Represents each of the addressing modes the 6502 supports.
/// This will be used to determine how the address for an operand will
/// be retrieved or how an instruction behaves.
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

    pub pc: u16,
    pub sp: u8,
    pub acc: u8,
    pub x: u8,
    pub y: u8,
    pub status: CPUFlags,
    pub bus: Bus,

}

pub trait Mem {

    fn mem_read_u8(&mut self, addr: u16) -> u8;

    fn mem_read_u16(&mut self, addr: u16) -> u16 {
        let lsb: u16 = self.mem_read_u8(addr) as u16;
        let msb = self.mem_read_u8(addr + 1) as u16;
        (msb << 8) | lsb
    }

    fn mem_write_u8(&mut self, addr: u16, data: u8);

    fn mem_write_u16(&mut self, addr: u16, data: u16) {
        let msb = (data >> 8) as u8;
        let lsb = (data & 0xff) as u8;
        self.mem_write_u8(addr, lsb);
        self.mem_write_u8(addr + 1, msb);
    }

}

impl Mem for CPU {

    fn mem_read_u8(&mut self, addr: u16) -> u8 {
        self.bus.mem_read_u8(addr)
    }

    fn mem_read_u16(&mut self, addr: u16) -> u16 {
        self.bus.mem_read_u16(addr)
    }

    fn mem_write_u8(&mut self, addr: u16, data: u8) {
        self.bus.mem_write_u8(addr, data);
    }

    fn mem_write_u16(&mut self, addr: u16, data: u16) {
        self.bus.mem_write_u16(addr, data);
    }

}

impl CPU {

    /// Create a new 6502 CPU in its default state,
    /// able to provide a custom `Bus` if you want to
    /// for some reason
    pub fn new(bus: Bus) -> Self {
        CPU {
            pc: 0x0000,
            sp: 0xFD,
            acc: 0,
            x: 0,
            y: 0,
            status: CPUFlags::from_bits_truncate(0x24), // Break flags
            bus
        }
    }

    /// Sets the CPU to the default state
    pub fn reset(&mut self) {
        self.pc = self.mem_read_u16(0xFFFC);
        self.sp = 0xFF;
        self.acc = 0;
        self.x = 0;
        self.y = 0;
        self.status = CPUFlags::from_bits_truncate(0);
    }

    /// DEPRECATED?? Maybe only useful for testing??
    /// Loads the program into memory, starting at address 0x8000.
    /// Calling this method WILL reset the CPU state. If you want to test the CPU
    /// while in a custom state, do not call this and instead set the state, call load(), then run()
    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        // self.reset();
        self.run();
    }

    /// DEPRECATED?? Maybe only useful for testing??
    /// Loads a program into memory at the specified address.
    /// Allows for use of custom test code that you may not want to begin executing
    /// at the default 6502 reset vector.
    pub fn load_custom_program(&mut self, program: Vec<u8>, start_vector: u16) {        
        for (index, byte) in program.iter().enumerate() {
            self.mem_write_u8(start_vector+index as u16, *byte);
        }
        self.pc = 0x0600;
    }

    /// DEPRECATED?? Maybe only useful for testing??
    /// Loads a program into memory
    pub fn load(&mut self, program: Vec<u8>) {
        self.load_custom_program(program, 0x0600);
    }

    /// Begins execution with no callback
    pub fn run(&mut self) {
        self.run_with_callback(|_| {});
    }

    /// Begins execution with a provided callback function. This is really useful for debugging,
    /// as you can inject methods that are run each time the CPU fetches an instruction.
    /// `callback` is executed before the program counter is incremented and the next instruction is executed.
    pub fn run_with_callback<F>(&mut self, mut callback: F) where F: FnMut(&mut CPU), {

        let ins_set = &(*instructions::CPU_INSTRUCTION_SET);

        // TODO REMOVE LATER
        // println!("IMPLEMENTED {} OF 256 INSTRUCTIONS", ins_set.len());

        loop {

            callback(self);

            let opcode = self.mem_read_u8(self.pc);
            let ins = *ins_set.get(&opcode).unwrap_or_else(|| panic!("Instruction {} is invalid or unimplemented", opcode));

            self.pc += 1;
            let current_pc = self.pc;

            match opcode {

                0x00 => return,
                0xEA => (),
                0x1A | 0x3A | 0x5A | 0x7A | 0xDA | 0xFA => (),
                0x80 => (),
                0x1C | 0x3C | 0x5C | 0x7C | 0xDC | 0xFC => self.nop_read(&ins.addressing_mode),
                0x04 | 0x44 | 0x64 | 0x0C | 0x14 | 0x34 | 0x54 | 0x74 | 0xD4 | 0xF4 => self.nop_read(&ins.addressing_mode),
                0x69 | 0x65 | 0x75 | 0x6D | 0x7D | 0x79 | 0x61 | 0x71 => self.add_with_carry(&ins.addressing_mode),
                0xE9 | 0xE5 | 0xF5 | 0xED | 0xFD | 0xF9 | 0xE1 | 0xF1 => self.subtract_with_carry(&ins.addressing_mode),
                0xEB => self.subtract_with_carry(&ins.addressing_mode),
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
                0xA3 | 0xA7 | 0xAF | 0xB3 | 0xB7 | 0xBF => self.load_acc_and_x(&ins.addressing_mode),
                0x83 | 0x87 | 0x8F | 0x97 => self.store_registers(&ins.addressing_mode, &RegisterID::ACC, &RegisterID::X),
                0xAA => self.transfer_register(&RegisterID::ACC, &RegisterID::X),
                0xA8 => self.transfer_register(&RegisterID::ACC, &RegisterID::Y),
                0xBA => self.transfer_register(&RegisterID::SP, &RegisterID::X),
                0x8A => self.transfer_register(&RegisterID::X, &RegisterID::ACC),
                0x9A => self.transfer_register(&RegisterID::X, &RegisterID::SP),
                0x98 => self.transfer_register(&RegisterID::Y, &RegisterID::ACC),
                0x18 => self.clear_flag(CPUFlags::CARRY),
                0xD8 => self.clear_flag(CPUFlags::DECIMAL_MODE),
                0x58 => self.clear_flag(CPUFlags::INTERRUPT_DISABLE),
                0xB8 => self.clear_flag(CPUFlags::OVERFLOW),
                0x38 => self.set_flag(CPUFlags::CARRY),
                0xF8 => self.set_flag(CPUFlags::DECIMAL_MODE),
                0x78 => self.set_flag(CPUFlags::INTERRUPT_DISABLE),
                0xCA => self.decrement_register(&RegisterID::X),
                0x88 => self.decrement_register(&RegisterID::Y),
                0xE8 => self.increment_register(&RegisterID::X),
                0xC8 => self.increment_register(&RegisterID::Y),
                0xE6 | 0xF6 | 0xEE | 0xFE => self.increment_memory(&ins.addressing_mode),
                0xC6 | 0xD6 | 0xCE | 0xDE => self.decrement_memory(&ins.addressing_mode),
                0xC3 | 0xC7 | 0xCF | 0xD3 | 0xD7 | 0xDB | 0xDF => self.decrement_memory_unofficial(&ins.addressing_mode),
                0xE3 | 0xE7 | 0xEF | 0xF3 | 0xF7 | 0xFB | 0xFF  => self.increment_mem_and_subtract_from_acc(&ins.addressing_mode),
                0x0A => self.acc_shift_left(),
                0x4A => self.acc_shift_right(),
                0x2A => self.rotate_acc_left(),
                0x6A => self.rotate_acc_right(),
                0x03 | 0x07 | 0x0F | 0x13 | 0x17 | 0x1B | 0x1F => self.arithmetic_shift_left_and_or_with_acc(&ins.addressing_mode),
                0x43 | 0x47 | 0x4F | 0x53 | 0x57 | 0x5B | 0x5F => self.logical_shift_right_and_xor_with_acc(&ins.addressing_mode),
                0x23 | 0x27 | 0x2F | 0x33 | 0x37 | 0x3B | 0x3F => self.rotate_left_and_and_with_acc(&ins.addressing_mode),
                0x63 | 0x67 | 0x6F | 0x73 | 0x77 | 0x7B | 0x7F => self.rotate_right_and_add_to_acc(&ins.addressing_mode),
                0x06 | 0x16 | 0x0E | 0x1E => self.mem_shift_left(&ins.addressing_mode),
                0x46 | 0x56 | 0x4E | 0x5E => self.mem_shift_right(&ins.addressing_mode),
                0x26 | 0x36 | 0x2E | 0x3E => self.rotate_mem_left(&ins.addressing_mode),
                0x66 | 0x76 | 0x6E | 0x7E => self.rotate_mem_right(&ins.addressing_mode),
                0xB0 => self.branch_if(self.is_flag_set(CPUFlags::CARRY)),
                0xF0 => self.branch_if(self.is_flag_set(CPUFlags::ZERO)),
                0x30 => self.branch_if(self.is_flag_set(CPUFlags::NEGATIVE)),
                0x70 => self.branch_if(self.is_flag_set(CPUFlags::OVERFLOW)),
                0x90 => self.branch_if(!self.is_flag_set(CPUFlags::CARRY)),
                0xD0 => self.branch_if(!self.is_flag_set(CPUFlags::ZERO)),
                0x10 => self.branch_if(!self.is_flag_set(CPUFlags::NEGATIVE)),
                0x50 => self.branch_if(!self.is_flag_set(CPUFlags::OVERFLOW)),
                0x4C | 0x6C => self.jump(&ins.addressing_mode),
                0x20 => self.jump_to_subroutine(&ins.addressing_mode),
                0x60 => self.return_from_subroutine(),
                0x40 => self.return_from_interrupt(),
                0x48 => self.stack_push_u8(self.acc),
                0x08 => self.stack_push_status(),
                0x68 => self.stack_pop_acc(),
                0x28 => self.stack_pop_status(),
                0x09 | 0x05 | 0x15 | 0x0D | 0x1D | 0x19 | 0x01 | 0x11 => self.inclusive_or(&ins.addressing_mode),
                0x49 | 0x45 | 0x55 | 0x4D | 0x5D | 0x59 | 0x41 | 0x51 => self.exclusive_or(&ins.addressing_mode),
                _ => todo!("Opcode [0x{:0X}] is invalid or unimplemented", opcode)

            }

            self.bus.tick_cycles(ins.cycles);

            if current_pc == self.pc {
                self.pc += (ins.bytes-1) as u16;
            }
        }
    }

    fn get_operand_address(&mut self, addressing_mode: &AddressingMode) -> (u16, bool) {
        self.get_absolute_address(addressing_mode, self.pc)
    }

    pub fn get_absolute_address(&mut self, addressing_mode: &AddressingMode, addr: u16) -> (u16, bool) {
        
        match addressing_mode {

            AddressingMode::Immediate => (addr, false),
            AddressingMode::Absolute => (self.mem_read_u16(addr), false),
            AddressingMode::AbsoluteX => {
                let base_addr = self.mem_read_u16(addr);
                let target_addr = base_addr.wrapping_add(self.x as u16);
                (target_addr, self.page_crossed(base_addr, target_addr))
            },
            AddressingMode::AbsoluteY => {
                let base_addr = self.mem_read_u16(addr);
                let target_addr = base_addr.wrapping_add(self.y as u16);
                (target_addr, self.page_crossed(base_addr, target_addr))
            },
            AddressingMode::ZeroPage => (self.mem_read_u8(addr) as u16, false),
            AddressingMode::ZeroPageX => (self.mem_read_u8(addr).wrapping_add(self.x) as u16, false),
            AddressingMode::ZeroPageY => (self.mem_read_u8(addr).wrapping_add(self.y) as u16, false),
            AddressingMode::Indirect => {

                let target_addr = self.mem_read_u16(addr);

                if target_addr & 0xFF == 0xFF {
                    let lsb = self.mem_read_u8(target_addr);
                    let msb = self.mem_read_u8(target_addr & 0xFF00);
                    ((msb as u16) << 8 | lsb as u16, false)
                } else {
                    (self.mem_read_u16(target_addr), false)
                }

            },
            AddressingMode::IndirectX => {

                let initial_read_addr = self.mem_read_u8(addr);
                let offset_addr = initial_read_addr.wrapping_add(self.x);

                let lsb = self.mem_read_u8(offset_addr as u16);
                let msb = self.mem_read_u8(offset_addr.wrapping_add(1) as u16);

                ((msb as u16) << 8 | lsb as u16, false)

            },
            AddressingMode::IndirectY => {

                let initial_read_addr = self.mem_read_u8(addr);

                let lsb = self.mem_read_u8(initial_read_addr as u16);
                let msb = self.mem_read_u8(initial_read_addr.wrapping_add(1) as u16);
                let target_addr_base = (msb as u16) << 8 | lsb as u16;
                let target_addr = target_addr_base.wrapping_add(self.y as u16);

                (target_addr, self.page_crossed(target_addr_base, target_addr))

            },
            AddressingMode::Relative => {
                let offset = self.mem_read_u8(addr) as i8;
                let relative_addr = addr.wrapping_add_signed(offset as i16).wrapping_add(1);
                (relative_addr, self.page_crossed(self.pc.wrapping_add(1), relative_addr))
            }
            _ => panic!("Addressing mode {:?} instruction should not be reading an address", addressing_mode)
        }
    }

    fn increment_register(&mut self, target_register: &RegisterID) {

        let register_ref = match target_register {
            RegisterID::X => &mut self.x,
            RegisterID::Y => &mut self.y,
            _ => panic!("Stack pointer or accumulator should not be targets")
        };

        let data = (*register_ref).wrapping_add(1);
        *register_ref = data;
        self.set_negative_and_zero_flags(data);
    }

    fn decrement_register(&mut self, target_register: &RegisterID) {

        let register_ref = match target_register {
            RegisterID::X => &mut self.x,
            RegisterID::Y => &mut self.y,
            _ => panic!("Stack pointer or accumulator should not be targets")
        };

        let data = (*register_ref).wrapping_sub(1);
        *register_ref = data;
        self.set_negative_and_zero_flags(data);

    }

    fn increment_memory(&mut self, addressing_mode: &AddressingMode) {

        let (target_addr, _) = self.get_operand_address(addressing_mode);
        let mut data = self.mem_read_u8(target_addr);
        
        data = data.wrapping_add(1);

        self.mem_write_u8(target_addr, data);
        self.set_negative_and_zero_flags(data);
    }

    fn decrement_memory(&mut self, addressing_mode: &AddressingMode) {

        let (target_addr, _) = self.get_operand_address(addressing_mode);
        let mut data = self.mem_read_u8(target_addr);
        
        data = data.wrapping_sub(1);

        self.mem_write_u8(target_addr, data);
        self.set_negative_and_zero_flags(data);
    }

    /// Don't set negative and zero bits, and if the 
    fn decrement_memory_unofficial(&mut self, addressing_mode: &AddressingMode) {

        let (target_addr, _) = self.get_operand_address(addressing_mode);
        let mut data = self.mem_read_u8(target_addr);
        data = data.wrapping_sub(1);
        self.mem_write_u8(target_addr, data);

        self.conditional_flag_set(data <= self.acc, CPUFlags::CARRY);
        self.set_negative_and_zero_flags(self.acc.wrapping_sub(data));

    }

    fn increment_mem_and_subtract_from_acc(&mut self, addressing_mode: &AddressingMode) {
        self.increment_memory(addressing_mode);
        let (target_addr, _) = self.get_operand_address(addressing_mode);
        let data = self.mem_read_u8(target_addr) as i8;
        self.add_to_acc(data.wrapping_neg().wrapping_sub(1) as u8);
    }

    fn inclusive_or(&mut self, addressing_mode: &AddressingMode) {

        let (target_addr, _) = self.get_operand_address(addressing_mode);
        let data = self.mem_read_u8(target_addr);

        self.acc |= data;
        self.set_negative_and_zero_flags(self.acc);

    }

    fn exclusive_or(&mut self, addressing_mode: &AddressingMode) {

        let (target_addr, _) = self.get_operand_address(addressing_mode);
        let data = self.mem_read_u8(target_addr);

        self.acc ^= data;
        self.set_negative_and_zero_flags(self.acc);

    }

    fn get_stack_pointer_addr(&self) -> u16 {
        0x0100 | self.sp as u16
    }

    fn stack_push_u8(&mut self, value: u8) {
        let stack_addr = self.get_stack_pointer_addr();
        self.mem_write_u8(stack_addr, value);
        self.sp = self.sp.wrapping_sub(1);
    }

    fn stack_pop_u8(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        let stack_addr = self.get_stack_pointer_addr();
        self.mem_read_u8(stack_addr)
    }

    fn stack_push_u16(&mut self, addr: u16) {

        let msb = (addr >> 8) as u8;
        let lsb = (addr & 0xFF) as u8;

        self.stack_push_u8(msb);
        self.stack_push_u8(lsb);

    }

    fn stack_pop_u16(&mut self) -> u16 {
        let lsb = self.stack_pop_u8() as u16;
        let msb = self.stack_pop_u8() as u16;
        (msb << 8) | lsb
    }

    fn stack_pop_acc(&mut self) {
        self.acc = self.stack_pop_u8();
        self.set_negative_and_zero_flags(self.acc);
    }


    /// This needs its own method because of how the CPU uses those
    /// strange bits 4-5 in the status. More info at the link below
    /// <http://wiki.nesdev.com/w/index.php/CPU_status_flag_behavior>
    fn stack_push_status(&mut self) {
        let mut status = self.status;
        status.insert(CPUFlags::BREAK_COMMAND_4);
        status.insert(CPUFlags::BREAK_COMMAND_5);
        self.stack_push_u8(status.bits());
    }

    /// This needs its own method because of how the CPU uses those
    /// strange bits 4-5 in the status. More info at the link below
    /// <http://wiki.nesdev.com/w/index.php/CPU_status_flag_behavior>
    fn stack_pop_status(&mut self) {
        self.status = CPUFlags::from_bits_truncate(self.stack_pop_u8());
        self.clear_flag(CPUFlags::BREAK_COMMAND_4);
        self.set_flag(CPUFlags::BREAK_COMMAND_5);
    }

    fn return_from_interrupt(&mut self) {
        self.status = CPUFlags::from_bits_truncate(self.stack_pop_u8());
        self.pc = self.stack_pop_u16();
        self.set_flag(CPUFlags::BREAK_COMMAND_5);
    }

    /// 6502 has a bug when the indirect vector is on a page boundary
    /// <https://www.nesdev.org/obelisk-6502-guide/reference.html#JMP>
    fn jump(&mut self, addressing_mode: &AddressingMode) {
        let (target_addr, _) = self.get_operand_address(addressing_mode);
        self.pc = target_addr;
    }

    fn jump_to_subroutine(&mut self, addressing_mode: &AddressingMode) {

        // We want to return to the instruction AFTER this
        // because otherwise we'll just come back to the
        // JSR instruction and loop
        // We're doing +2 (because we read 2 bytes after the instruction)
        // and -1 because we want to store the target return-1
        self.stack_push_u16(self.pc + 2 - 1);
        let (target_addr, _) = self.get_operand_address(addressing_mode);
        self.pc = target_addr;

    }

    fn return_from_subroutine(&mut self) {
        let target_addr = self.stack_pop_u16();
        self.pc = target_addr + 1;
    }

    // ! Really wanted to do a guardian clause instead, but tarpaulin wasn't covering the early return
    fn branch_if(&mut self, condition: bool) {
        if condition {
            let (jump_addr, page_crossed) = self.get_operand_address(&AddressingMode::Relative);
            self.pc = jump_addr;
            self.bus.tick();
            self.tick_if_page_crossed(page_crossed);
        }
    }

    fn add_to_acc(&mut self, data: u8) {
        let mut sum = self.acc as u16 + data as u16;

        if self.is_flag_set(CPUFlags::CARRY) {
            sum += 1;
        }

        let carry = sum > 0xff;
        let has_overflow = (data ^ sum as u8) & (sum as u8 ^ self.acc) & 0x80 != 0;

        self.conditional_flag_set(carry, CPUFlags::CARRY);
        self.conditional_flag_set(has_overflow, CPUFlags::OVERFLOW);

        self.acc = sum as u8;
        self.set_negative_and_zero_flags(self.acc);
    }

    fn add_with_carry(&mut self, addressing_mode: &AddressingMode) {
        let (target_addr, _) = self.get_operand_address(addressing_mode);
        let data = self.mem_read_u8(target_addr);
        self.add_to_acc(data);
    }

    fn subtract_with_carry(&mut self, addressing_mode: &AddressingMode) {
        let (target_addr, _) = self.get_operand_address(addressing_mode);
        let data = self.mem_read_u8(target_addr) as i8;
        self.add_to_acc(data.wrapping_neg().wrapping_sub(1) as u8);
    }

    fn acc_shift_left(&mut self) {
        self.conditional_flag_set(self.acc & 0b1000_0000 > 0, CPUFlags::CARRY);
        self.acc <<= 1;
        self.set_negative_and_zero_flags(self.acc);
    }

    fn acc_shift_right(&mut self) {
        self.conditional_flag_set(self.acc & 1 == 1, CPUFlags::CARRY);
        self.acc >>= 1;
        self.set_negative_and_zero_flags(self.acc);
    }

    fn mem_shift_left(&mut self, addressing_mode: &AddressingMode) {

        let (target_addr, _) = self.get_operand_address(addressing_mode);
        let mut data = self.mem_read_u8(target_addr);
        
        self.conditional_flag_set(data & 0b1000_0000 > 0, CPUFlags::CARRY);
        data <<= 1;

        self.set_negative_and_zero_flags(data);
        self.mem_write_u8(target_addr, data);

    }

    fn mem_shift_right(&mut self, addressing_mode: &AddressingMode) {

        let (target_addr, _) = self.get_operand_address(addressing_mode);
        let mut data = self.mem_read_u8(target_addr);
        
        self.conditional_flag_set(data & 1 == 1, CPUFlags::CARRY);
        data >>= 1;

        self.set_negative_and_zero_flags(data);
        self.mem_write_u8(target_addr, data);

    }

    fn rotate_acc_left(&mut self) {

        let carry_enabled = self.is_flag_set(CPUFlags::CARRY);
        self.conditional_flag_set(self.acc >> 7 == 1, CPUFlags::CARRY);

        self.acc <<= 1;

        if carry_enabled {
            self.acc |= 1;
        }

        self.set_negative_and_zero_flags(self.acc);

    }

    fn rotate_acc_right(&mut self) {

        let carry_enabled = self.is_flag_set(CPUFlags::CARRY);
        self.conditional_flag_set(self.acc & 1 == 1, CPUFlags::CARRY);

        self.acc >>= 1;

        if carry_enabled {
            self.acc |= 0b1000_0000;
        }

        self.set_negative_and_zero_flags(self.acc);

    }

    fn rotate_mem_left(&mut self, addressing_mode: &AddressingMode) {

        let (target_addr, _) = self.get_operand_address(addressing_mode);
        let mut data = self.mem_read_u8(target_addr);
        let carry_enabled = self.is_flag_set(CPUFlags::CARRY);

        self.conditional_flag_set(data >> 7 == 1, CPUFlags::CARRY);
        data <<= 1;

        if carry_enabled {
            data |= 1;
        }

        self.set_negative_and_zero_flags(data);
        self.mem_write_u8(target_addr, data);

    }

    fn rotate_mem_right(&mut self, addressing_mode: &AddressingMode) {

        let (target_addr, _) = self.get_operand_address(addressing_mode);
        let mut data = self.mem_read_u8(target_addr);
        let carry_enabled = self.is_flag_set(CPUFlags::CARRY);

        self.conditional_flag_set(data & 1 == 1, CPUFlags::CARRY);
        data >>= 1;

        if carry_enabled {
            data |= 0b1000_0000;
        }

        self.set_negative_and_zero_flags(data);
        self.mem_write_u8(target_addr, data);

    }

    fn arithmetic_shift_left_and_or_with_acc(&mut self, addressing_mode: &AddressingMode) {
        self.mem_shift_left(addressing_mode);
        self.inclusive_or(addressing_mode)
    }

    fn logical_shift_right_and_xor_with_acc(&mut self, addressing_mode: &AddressingMode) {
        self.mem_shift_right(addressing_mode);
        self.exclusive_or(addressing_mode);
    }

    fn rotate_left_and_and_with_acc(&mut self, addressing_mode: &AddressingMode) {
        self.rotate_mem_left(addressing_mode);
        self.and(addressing_mode)
    }

    fn rotate_right_and_add_to_acc(&mut self, addressing_mode: &AddressingMode) {
        self.rotate_mem_right(addressing_mode);
        let (target_addr, _) = self.get_operand_address(addressing_mode);
        let data = self.mem_read_u8(target_addr);
        self.add_to_acc(data);
    }

    fn set_flag(&mut self, flag_alias: CPUFlags) {
        self.status.insert(flag_alias);
    }

    fn clear_flag(&mut self, flag_alias: CPUFlags) {
        self.status.remove(flag_alias);
    }

    /// If `condition` is true, the specified flag is set.
    /// If `condition` is false, the specified flag is cleared.
    fn conditional_flag_set(&mut self, condition: bool, flag_alias: CPUFlags) {
        self.status.set(flag_alias, condition);
    }

    fn is_flag_set(&self, flag_alias: CPUFlags) -> bool {
        self.status.contains(flag_alias)
    }

    fn set_negative_and_zero_flags(&mut self, value: u8) {
        self.conditional_flag_set(value == 0, CPUFlags::ZERO);
        self.conditional_flag_set(value & CPUFlags::NEGATIVE.bits() > 0, CPUFlags::NEGATIVE);
    }

    fn page_crossed(&self, base: u16, target: u16) -> bool {
        (base & 0xFF00) != (target & 0xFF00)
    }

    fn tick_if_page_crossed(&mut self, page_crossed: bool) {
        if page_crossed {
            self.bus.tick();
        }
    }

    fn load_register(&mut self, addressing_mode: &AddressingMode, target_register: &RegisterID) {

        let (target_addr, page_crossed) = self.get_operand_address(addressing_mode);
        let data = self.mem_read_u8(target_addr);
        let register_ref = match target_register {
            RegisterID::ACC => &mut self.acc,
            RegisterID::X => &mut self.x,
            RegisterID::Y => &mut self.y,
            RegisterID::SP => panic!("Stack pointer should not be a target for loading")
        };
        
        *register_ref = data;
        self.set_negative_and_zero_flags(data);
        self.tick_if_page_crossed(page_crossed);

    }

    fn store_register(&mut self, addressing_mode: &AddressingMode, target_register: &RegisterID) {

        let register_value = match target_register {
            RegisterID::ACC => self.acc,
            RegisterID::X => self.x,
            RegisterID::Y => self.y,
            RegisterID::SP => panic!("Stack pointer should not be a target for storing")
        };

        let (target_addr, _) = self.get_operand_address(addressing_mode);
        self.mem_write_u8(target_addr, register_value);

    }

    fn load_acc_and_x(&mut self, addressing_mode: &AddressingMode) {

        let (target_addr, page_crossed) = self.get_operand_address(addressing_mode);
        let data = self.mem_read_u8(target_addr);
        self.acc = data;
        self.x = data;

        self.set_negative_and_zero_flags(data);
        self.tick_if_page_crossed(page_crossed);
        
    }

    fn store_registers(&mut self, addressing_mode: &AddressingMode, reg_a: &RegisterID, reg_b: &RegisterID) {
        
        let reg_a_value = match reg_a {
            RegisterID::ACC => self.acc,
            RegisterID::X => self.x,
            RegisterID::Y => self.y,
            RegisterID::SP => panic!("Stack pointer should not be a target for storing")
        };

        let reg_b_value = match reg_b {
            RegisterID::ACC => self.acc,
            RegisterID::X => self.x,
            RegisterID::Y => self.y,
            RegisterID::SP => panic!("Stack pointer should not be a target for storing")
        };

        let (target_addr, _) = self.get_operand_address(addressing_mode);
        self.mem_write_u8(target_addr, reg_a_value & reg_b_value);

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
            self.set_negative_and_zero_flags(source_value);
        }

    }

    fn compare_register(&mut self, addressing_mode: &AddressingMode, target_register: &RegisterID) {

        let (target_addr, _) = self.get_operand_address(addressing_mode);
        let data = self.mem_read_u8(target_addr);
        let register_value = match target_register {
            RegisterID::ACC => self.acc,
            RegisterID::X => self.x,
            RegisterID::Y => self.y,
            RegisterID::SP => panic!("Stack pointer should not be a target for comparing")
        };

        let result = register_value.wrapping_sub(data);
        self.set_negative_and_zero_flags(result);
        self.conditional_flag_set(register_value >= data, CPUFlags::CARRY);
    }

    fn and(&mut self, addressing_mode: &AddressingMode) {
        let (target_addr, _) = self.get_operand_address(addressing_mode);
        let data = self.mem_read_u8(target_addr);
        self.acc &= data;
        self.set_negative_and_zero_flags(self.acc);
    }

    fn bit(&mut self, addressing_mode: &AddressingMode) {

        let (target_addr, _) = self.get_operand_address(addressing_mode);
        let data = self.mem_read_u8(target_addr);
        let result = self.acc & data;

        self.conditional_flag_set(data & CPUFlags::OVERFLOW.bits() > 0, CPUFlags::OVERFLOW);
        self.conditional_flag_set(data & CPUFlags::NEGATIVE.bits() > 0, CPUFlags::NEGATIVE);
        self.conditional_flag_set(result == 0, CPUFlags::ZERO);

    }

    fn nop_read(&mut self, addressing_mode: &AddressingMode) {
        let (addr, page_crossed) = self.get_operand_address(addressing_mode);
        self.mem_read_u8(addr);
        self.tick_if_page_crossed(page_crossed);
    }

}

#[cfg(test)]
mod tests {

    use std::vec;

    use crate::cpu::*;
    use crate::rom::tests::test_rom;

    fn init_test_cpu() -> CPU {
        CPU::new(Bus::new(test_rom()))
    }

    #[test]
    fn test_cpu_init() {

        let cpu = init_test_cpu();

        assert_eq!(cpu.pc, 0x0000);
        assert_eq!(cpu.sp, 0xFD);
        assert_eq!(cpu.acc, 0);
        assert_eq!(cpu.x, 0);
        assert_eq!(cpu.y, 0);
        assert_eq!(cpu.status.bits(), 0x24);

    }

    #[test]
    fn test_cpu_reset() {

        let mut cpu = init_test_cpu();

        cpu.acc = 52;
        cpu.sp = 124;
        cpu.pc = 1892;
        cpu.x = 15;
        cpu.y = 16;
        cpu.status = CPUFlags::from_bits_truncate(0b10010000);

        cpu.reset();

        assert_eq!(cpu.pc, 0x0101);
        assert_eq!(cpu.sp, 0xFF);
        assert_eq!(cpu.acc, 0);
        assert_eq!(cpu.x, 0);
        assert_eq!(cpu.y, 0);
        assert_eq!(cpu.status.bits(), 0);

    }

    #[test]
    fn test_set_negative_and_zero_flags() {

        let mut cpu = init_test_cpu();

        cpu.set_negative_and_zero_flags(cpu.acc);
        assert!(cpu.is_flag_set(CPUFlags::ZERO));

        cpu.acc = 130;
        cpu.set_negative_and_zero_flags(cpu.acc);
        assert!(cpu.is_flag_set(CPUFlags::NEGATIVE));

        cpu.acc = 16;
        cpu.set_negative_and_zero_flags(cpu.acc);
        assert!(!cpu.is_flag_set(CPUFlags::NEGATIVE));
        assert!(!cpu.is_flag_set(CPUFlags::ZERO));

    }

    #[test]
    fn test_increment_register () {

        let mut cpu = init_test_cpu();
        cpu.x = 0xFE;

        cpu.increment_register(&RegisterID::X);
        assert_eq!(cpu.x, 0xFF);

        cpu.increment_register(&RegisterID::X);
        assert_eq!(cpu.x, 0);

    }

    #[test]
    fn test_decrement_register () {

        let mut cpu = init_test_cpu();
        cpu.x = 1;

        cpu.decrement_register(&RegisterID::X);
        assert_eq!(cpu.x, 0);

        cpu.decrement_register(&RegisterID::X);
        assert_eq!(cpu.x, 255);

    }

    #[test]
    fn test_mem_read_u8 () {

        let mut cpu = init_test_cpu();
        cpu.mem_write_u8(162, 0xAF);

        assert_eq!(cpu.mem_read_u8(162), 0xAF);

    }

    #[test]
    fn test_mem_read_u16 () {

        let mut cpu = init_test_cpu();
        cpu.mem_write_u8(162, 0x80);
        cpu.mem_write_u8(163, 0x08);

        assert_eq!(cpu.mem_read_u16(162), 0x0880);

    }

    #[test]
    fn test_mem_write_u8 () {

        let mut cpu = init_test_cpu();
        let data: u8 = 0x12;
        cpu.mem_write_u8(162, data);

        assert_eq!(cpu.mem_read_u8(162), 0x12);

    }

    #[test]
    fn test_mem_write_u16 () {

        let mut cpu = init_test_cpu();
        let data: u16 = 0x1234;
        cpu.mem_write_u16(162, data);

        assert_eq!(cpu.mem_read_u8(162), 0x34);
        assert_eq!(cpu.mem_read_u8(163), 0x12);

    }

    #[test]
    fn test_clear_flag() {

        let mut cpu = init_test_cpu();
        cpu.status = CPUFlags::from_bits_truncate(0b1111_1111);

        cpu.clear_flag(CPUFlags::ZERO);
        assert!(!cpu.is_flag_set(CPUFlags::ZERO));

    }

    #[test]
    fn test_get_operand_address_immediate() {

        let mut cpu = init_test_cpu();
        cpu.acc = 0x10;
        cpu.x = 0x11;
        cpu.y = 0x12;
        cpu.sp = 0x13;
        cpu.pc = 0xF0;

        let (addr, _) = cpu.get_operand_address(&AddressingMode::Immediate);
        assert_eq!(addr, 0xF0);

    }

    #[test]
    fn test_get_operand_address_absolute() {

        let mut cpu = init_test_cpu();
        cpu.acc = 0x10;
        cpu.x = 0x11;
        cpu.y = 0x12;
        cpu.sp = 0x13;
        cpu.pc = 0xF0;

        cpu.mem_write_u16(0xF0, 0x8088);

        let (addr, _) = cpu.get_operand_address(&AddressingMode::Absolute);
        assert_eq!(addr, 0x8088);

        let (addr, _) = cpu.get_operand_address(&AddressingMode::AbsoluteX);
        assert_eq!(addr, 0x8099);

        let (addr, _) = cpu.get_operand_address(&AddressingMode::AbsoluteY);
        assert_eq!(addr, 0x809A);

        cpu.mem_write_u16(0xF0, 0xFFF0);

        // Absolute addressing wrap around
        let (addr, _) = cpu.get_operand_address(&AddressingMode::AbsoluteX);
        assert_eq!(addr, 0x01);

        let (addr, _) = cpu.get_operand_address(&AddressingMode::AbsoluteY);
        assert_eq!(addr, 0x02);

    }

    #[test]
    fn test_get_operand_address_zero_page() {

        let mut cpu = init_test_cpu();
        cpu.acc = 0x10;
        cpu.x = 0x11;
        cpu.y = 0x12;
        cpu.sp = 0x13;
        cpu.pc = 0xF0;

        cpu.mem_write_u16(0xF0, 0x8088);

        let (addr, _) = cpu.get_operand_address(&AddressingMode::ZeroPage);
        assert_eq!(addr, 0x88);

        let (addr, _) = cpu.get_operand_address(&AddressingMode::ZeroPageX);
        assert_eq!(addr, 0x99);

        let (addr, _) = cpu.get_operand_address(&AddressingMode::ZeroPageY);
        assert_eq!(addr, 0x9A);

        cpu.mem_write_u16(0xF0, 0xFFF0);

        // Zero page addressing wrap around
        let (addr, _) = cpu.get_operand_address(&AddressingMode::ZeroPageX);
        assert_eq!(addr, 0x01);

        let (addr, _) = cpu.get_operand_address(&AddressingMode::ZeroPageY);
        assert_eq!(addr, 0x02);

    }

    #[test]
    fn test_get_operand_address_indirect() {

        let mut cpu = init_test_cpu();
        cpu.acc = 0x10;
        cpu.x = 0x11;
        cpu.y = 0x12;
        cpu.sp = 0x13;
        cpu.pc = 0xF0;

        cpu.mem_write_u16(0xF0, 0x80);
        cpu.mem_write_u16(0x80, 0x1234);
        cpu.mem_write_u16(0x91, 0x6789);

        let (addr, _) = cpu.get_operand_address(&AddressingMode::Indirect);
        assert_eq!(addr, 0x1234);

        let (addr, _) = cpu.get_operand_address(&AddressingMode::IndirectX);
        assert_eq!(addr, 0x6789);

        let (addr, _) = cpu.get_operand_address(&AddressingMode::IndirectY);
        assert_eq!(addr, 0x1246);

    }

    #[test]
    fn test_get_operand_address_relative() {

        let mut cpu = init_test_cpu();
        cpu.acc = 0x10;
        cpu.x = 0x11;
        cpu.y = 0x12;
        cpu.sp = 0x13;
        cpu.pc = 0xF0;

        cpu.mem_write_u16(0xF0, 0x8001);
        let (addr, _) = cpu.get_operand_address(&AddressingMode::Relative);
        assert_eq!(addr, 0xF2);

        cpu.mem_write_u8(0xF0, 0b1111_1100);
        let (addr, _) = cpu.get_operand_address(&AddressingMode::Relative);
        assert_eq!(addr, 0b1110_1101);

    }

    #[test]
    #[should_panic]
    fn test_get_operand_address_implied_panics() {
        let mut cpu = init_test_cpu();
        cpu.get_operand_address(&AddressingMode::Implied);
    }

    #[test]
    #[should_panic]
    fn test_load_register_sp_panics() {
        let mut cpu = init_test_cpu();
        cpu.load_register(&AddressingMode::Immediate, &RegisterID::SP);
    }

    #[test]
    #[should_panic]
    fn test_store_register_sp_panics() {
        let mut cpu = init_test_cpu();
        cpu.store_register(&AddressingMode::Immediate, &RegisterID::SP);
    }

    #[test]
    #[should_panic]
    fn test_compare_register_sp_panics() {
        let mut cpu = init_test_cpu();
        cpu.compare_register(&AddressingMode::Immediate, &RegisterID::SP);
    }

    #[test]
    #[should_panic]
    fn test_increment_register_panics() {
        let mut cpu = init_test_cpu();
        cpu.increment_register(&RegisterID::SP);
    }

    #[test]
    #[should_panic]
    fn test_decrement_register_panics() {
        let mut cpu = init_test_cpu();
        cpu.decrement_register(&RegisterID::SP);
    }

    #[test]
    fn test_adc() {

        let mut cpu = init_test_cpu();
        let program = vec![0xA9, 0xF0, 0x69, 0x0F, 0x00];
        cpu.load_and_run(program);

        assert_eq!(cpu.acc, 0xFF);

        let program = vec![0xA9, 0xF0, 0x69, 0x10, 0x00];
        cpu.load_and_run(program);

        assert!(cpu.is_flag_set(CPUFlags::CARRY));

    }

    #[test]
    fn test_and() {
        
        let mut cpu = init_test_cpu();
        let program = vec![0xA9, 0b1010_1010, 0x29, 0b1111_0000, 0x00];
        cpu.load_and_run(program);

        assert_eq!(cpu.acc, 0b1010_0000);

    }

    #[test]
    fn test_asl_acc() {
        
        let mut cpu = init_test_cpu();
        let program = vec![0xA9, 0b0101_0101, 0x0A, 0x00];
        cpu.load_and_run(program);

        assert_eq!(cpu.acc, 0b1010_1010);
        assert!(!cpu.is_flag_set(CPUFlags::CARRY));

        let program = vec![0xA9, 0b1010_1010, 0x0A, 0x00];
        cpu.load_and_run(program);

        assert_eq!(cpu.acc, 0b0101_0100);
        assert!(cpu.is_flag_set(CPUFlags::CARRY));

    }

    #[test]
    fn test_asl_mem() {
        
        let mut cpu = init_test_cpu();
        let program = vec![0xA9, 0b0101_0101, 0x0E, 0x01, 0x06];
        cpu.load_and_run(program);

        assert_eq!(cpu.mem_read_u8(0x0601), 0b1010_1010);
        assert!(!cpu.is_flag_set(CPUFlags::CARRY));

        cpu.reset();
        let program = vec![0xA9, 0b1010_1010, 0x0E, 0x01, 0x06];
        cpu.load_and_run(program);

        assert_eq!(cpu.mem_read_u8(0x0601), 0b0101_0100);
        assert!(cpu.is_flag_set(CPUFlags::CARRY));

    }

    #[test]
    fn test_bcc() {

        let mut cpu = init_test_cpu();

        // Branch condition is met
        let program = vec![0x90, 0b1111_1101];
        cpu.load_and_run(program);

        assert_eq!(cpu.pc, 0x0600);

        // Branch condition is NOT met
        let program = vec![0x90, 0b1111_1101];
        cpu.load(program);
        cpu.set_flag(CPUFlags::CARRY);
        cpu.run();

        assert_eq!(cpu.pc, 0x0603);

    }

    #[test]
    fn test_bcs() {

        let mut cpu = init_test_cpu();

        // Branch condition is met
        let program = vec![0xB0, 0b1111_1101];

        cpu.load(program);
        cpu.set_flag(CPUFlags::CARRY);
        cpu.run();

        assert_eq!(cpu.pc, 0x0600);

        // Branch condition is NOT met
        let program = vec![0xB0, 0b1111_1110];
        cpu.load(program);
        cpu.clear_flag(CPUFlags::CARRY);
        cpu.run();

        assert_eq!(cpu.pc, 0x0603);

    }

    #[test]
    fn test_beq() {

        let mut cpu = init_test_cpu();

        // Branch condition is met
        let program = vec![0xF0, 0b1111_1101];

        cpu.load(program);
        cpu.set_flag(CPUFlags::ZERO);
        cpu.run();

        assert_eq!(cpu.pc, 0x0600);

        // Branch condition is NOT met
        let program = vec![0xF0, 0b1111_1101];
        cpu.load(program);
        cpu.clear_flag(CPUFlags::ZERO);
        cpu.run();

        assert_eq!(cpu.pc, 0x0603);

    }

    #[test]
    fn test_bne() {

        let mut cpu = init_test_cpu();

        // Branch condition is met
        let program = vec![0xD0, 0b1111_1101];

        cpu.load(program);
        cpu.run();

        assert_eq!(cpu.pc, 0x0600);

        // Branch condition is NOT met
        let program = vec![0xD0, 0b1111_1101];
        cpu.load(program);
        cpu.set_flag(CPUFlags::ZERO);
        cpu.run();

        assert_eq!(cpu.pc, 0x0603);

    }

    #[test]
    fn test_bmi() {

        let mut cpu = init_test_cpu();

        // Branch condition is met
        let program = vec![0x30, 0b1111_1101];

        cpu.load(program);
        cpu.set_flag(CPUFlags::NEGATIVE);
        cpu.run();

        assert_eq!(cpu.pc, 0x0600);

        // Branch condition is NOT met
        let program = vec![0x30, 0b1111_1101];
        cpu.load(program);
        cpu.clear_flag(CPUFlags::NEGATIVE);
        cpu.run();

        assert_eq!(cpu.pc, 0x0603);

    }

    #[test]
    fn test_bpl() {

        let mut cpu = init_test_cpu();

        // Branch condition is met
        let program = vec![0x10, 0b1111_1101];

        cpu.load(program);
        cpu.run();

        assert_eq!(cpu.pc, 0x0600);

        // Branch condition is NOT met
        let program = vec![0x10, 0b1111_1101];
        cpu.reset();
        cpu.load(program);
        cpu.set_flag(CPUFlags::ZERO);
        cpu.run();

        assert_eq!(cpu.pc, 0x0600);

    }

    #[test]
    fn test_bvc() {

        let mut cpu = init_test_cpu();

        // Branch condition is met
        let program = vec![0x50, 0b1111_1101];

        cpu.load(program);
        cpu.run();

        assert_eq!(cpu.pc, 0x0600);

        // Branch condition is NOT met
        let program = vec![0x50, 0b1111_1101];
        cpu.load(program);
        cpu.set_flag(CPUFlags::OVERFLOW);
        cpu.run();

        assert_eq!(cpu.pc, 0x0603);

    }

    #[test]
    fn test_bvs() {

        let mut cpu = init_test_cpu();

        // Branch condition is met
        let program = vec![0x70, 0b1111_1101];

        cpu.load(program);
        cpu.set_flag(CPUFlags::OVERFLOW);
        cpu.run();

        assert_eq!(cpu.pc, 0x0600);

        // Branch condition is NOT met
        let program = vec![0x70, 0b1111_1101];
        cpu.load(program);
        cpu.clear_flag(CPUFlags::OVERFLOW);
        cpu.run();

        assert_eq!(cpu.pc, 0x0603);

    }

    #[test]
    fn test_bit() {

        let mut cpu = init_test_cpu();
        let program = vec![0xA9, 0xF0, 0x2C, 0x06, 0x06, 0x00, 0b1110_0000];
        cpu.load_and_run(program);
        assert!(cpu.is_flag_set(CPUFlags::NEGATIVE));
        assert!(cpu.is_flag_set(CPUFlags::OVERFLOW));
        assert_eq!(cpu.acc, 0xF0);

    }

    #[test]
    fn test_clc() {

        let mut cpu = init_test_cpu();
        cpu.status = CPUFlags::from_bits_truncate(0b1111_1111);

        let program = vec![0x18, 0x00];
        cpu.load(program);
        cpu.status = CPUFlags::from_bits_truncate(0b1111_1111);
        cpu.run();

        assert!(!cpu.is_flag_set(CPUFlags::CARRY));

    }

    #[test]
    fn test_cld() {

        let mut cpu = init_test_cpu();
        cpu.status = CPUFlags::from_bits_truncate(0b1111_1111);

        let program = vec![0xD8, 0x00];
        cpu.load(program);
        cpu.status = CPUFlags::from_bits_truncate(0b1111_1111);
        cpu.run();

        assert!(!cpu.is_flag_set(CPUFlags::DECIMAL_MODE));

    }

    #[test]
    fn test_cli() {

        let mut cpu = init_test_cpu();
        cpu.status = CPUFlags::from_bits_truncate(0b1111_1111);

        let program = vec![0x58, 0x00];
        cpu.load(program);
        cpu.status = CPUFlags::from_bits_truncate(0b1111_1111);
        cpu.run();

        assert!(!cpu.is_flag_set(CPUFlags::INTERRUPT_DISABLE));

    }

    #[test]
    fn test_clv() {

        let mut cpu = init_test_cpu();

        let program = vec![0xB8, 0x00];
        cpu.load(program);
        cpu.status = CPUFlags::from_bits_truncate(0b1111_1111);
        cpu.run();

        assert!(!cpu.is_flag_set(CPUFlags::OVERFLOW));

    }

    #[test]
    fn test_cmp() {

        let mut cpu = init_test_cpu();
        let program = vec![0xA9, 0xF0, 0xC9, 0xF0, 0x00];
        cpu.load_and_run(program);

        assert!(!cpu.is_flag_set(CPUFlags::NEGATIVE));
        assert!(cpu.is_flag_set(CPUFlags::ZERO));
        assert!(cpu.is_flag_set(CPUFlags::CARRY));

        let program = vec![0xA9, 0xF0, 0xC9, 0x00, 0x00];
        cpu.load_and_run(program);

        assert!(cpu.is_flag_set(CPUFlags::NEGATIVE));
        assert!(!cpu.is_flag_set(CPUFlags::ZERO));
        assert!(cpu.is_flag_set(CPUFlags::CARRY));

    }

    #[test]
    fn test_cpx() {

        let mut cpu = init_test_cpu();
        let program = vec![0xA2, 0xF0, 0xE0, 0xF0, 0x00];
        cpu.load_and_run(program);

        assert!(!cpu.is_flag_set(CPUFlags::NEGATIVE));
        assert!(cpu.is_flag_set(CPUFlags::ZERO));
        assert!(cpu.is_flag_set(CPUFlags::CARRY));


        let program = vec![0xA2, 0xF0, 0xE0, 0x00, 0x00];
        cpu.load_and_run(program);

        assert!(cpu.is_flag_set(CPUFlags::NEGATIVE));
        assert!(!cpu.is_flag_set(CPUFlags::ZERO));
        assert!(cpu.is_flag_set(CPUFlags::CARRY));

    }

    #[test]
    fn test_cpy() {

        let mut cpu = init_test_cpu();
        let program = vec![0xA0, 0xF0, 0xC0, 0xF0, 0x00];
        cpu.load_and_run(program);

        assert!(!cpu.is_flag_set(CPUFlags::NEGATIVE));
        assert!(cpu.is_flag_set(CPUFlags::ZERO));
        assert!(cpu.is_flag_set(CPUFlags::CARRY));

        let program = vec![0xA0, 0xF0, 0xC0, 0x00, 0x00];
        cpu.load_and_run(program);

        assert!(cpu.is_flag_set(CPUFlags::NEGATIVE));
        assert!(!cpu.is_flag_set(CPUFlags::ZERO));
        assert!(cpu.is_flag_set(CPUFlags::CARRY));

    }

    #[test]
    fn test_dec() {

        let mut cpu = init_test_cpu();
        let program = vec![0xCE, 0x04, 0x06, 0x00, 0b1111_1111];
        cpu.load_and_run(program);

        assert_eq!(cpu.mem_read_u8(0x0604), 0b1111_1110);
        assert!(cpu.is_flag_set(CPUFlags::NEGATIVE));

    }

    #[test]
    fn test_eor() {

        let mut cpu = init_test_cpu();
        let program = vec![0xA9, 0b1111_1111, 0x49, 0b0101_0101];
        cpu.load_and_run(program);

        assert_eq!(cpu.acc, 0b1010_1010);
        assert!(cpu.is_flag_set(CPUFlags::NEGATIVE));

    }

    #[test]
    fn test_inc() {

        let mut cpu = init_test_cpu();
        let program = vec![0xEE, 0x04, 0x06, 0x00, 0b1111_1111];
        cpu.load_and_run(program);

        assert_eq!(cpu.mem_read_u8(0x0604), 0x00);
        assert!(cpu.is_flag_set(CPUFlags::ZERO));

    }

    #[test]
    fn test_jmp() {

        let mut cpu = init_test_cpu();
        let program = vec![0x4C, 0xEE, 0x00];
        cpu.load_and_run(program);

        // The reason that it's 0x00F0 is because
        // we jump to 0x00EF and then read the next
        // instruction which is BRK so the final state
        // is 0x00EF + 1
        assert_eq!(cpu.pc, 0x00EF);

    }

    #[test]
    fn test_jsr() {

        let mut cpu = init_test_cpu();
        let program = vec![0x20, 0xEE, 0x00];
        cpu.reset();
        cpu.load_and_run(program);

        // The reason that it's 0x00F0 is because
        // we jump to 0x00EF and then read the next
        // instruction which is BRK so the final state
        // is 0x00EF + 1
        assert_eq!(cpu.pc, 0x00EF);
        assert_eq!(cpu.sp, 0xFD);
        assert_eq!(cpu.stack_pop_u16(), 0x0602);

    }

    #[test]
    fn test_lda_immediate() {

        let mut cpu = init_test_cpu();

        // Negative bit is set
        let program = vec![0xA9, 156, 0x00];
        cpu.load_and_run(program);

        assert_eq!(cpu.acc, 156);

    }

    #[test]
    fn test_lda_zero_page() {

        let mut cpu = init_test_cpu();

        let program = vec![0xA5, 0x04, 0x00];
        cpu.load(program);
        cpu.mem_write_u8(0x04, 0x13);
        cpu.run();

        assert_eq!(cpu.acc, 0x13);

    }

    #[test]
    fn test_lda_zero_page_x() {

        let mut cpu = init_test_cpu();

        let program = vec![0xA9, 0xFA, 0xAA, 0xB5, 0x0A, 0x00];
        cpu.load(program);
        cpu.mem_write_u8(0x04, 0x13);
        cpu.run();

        assert_eq!(cpu.acc, 0x13);

    }

    #[test]
    fn test_lda_absolute() {

        let mut cpu = init_test_cpu();

        let program = vec![0xAD, 0x04, 0x06, 0x00, 0x13];
        cpu.load_and_run(program);

        assert_eq!(cpu.acc, 0x13);

    }

    #[test]
    fn test_lda_absolute_x() {

        let mut cpu = init_test_cpu();

        let program = vec![0xA9, 0x04, 0xAA, 0xBD, 0x03, 0x06, 0x00, 0x13];
        cpu.load_and_run(program);

        assert_eq!(cpu.acc, 0x13);

    }

    #[test]
    fn test_lda_absolute_y() {

        let mut cpu = init_test_cpu();

        let program = vec![0xA9, 0x04, 0xA8, 0xB9, 0x03, 0x06, 0x00, 0x13];
        cpu.load_and_run(program);

        assert_eq!(cpu.acc, 0x13);

    }

    #[test]
    fn test_lda_indirect_x() {

        let mut cpu = init_test_cpu();

        let program = vec![0xA9, 0x10, 0xAA, 0xA1, 0xEF, 0x00];
        cpu.load(program);
        cpu.mem_write_u16(0xFF, 0x0001);
        cpu.mem_write_u8(0x01, 0x13);
        cpu.run();

        assert_eq!(cpu.acc, 0x13);

    }

    #[test]
    fn test_lda_indirect_y() {

        let mut cpu = init_test_cpu();

        let program = vec![0xA9, 0x10, 0xA8, 0xB1, 0xEF, 0x00];
        cpu.load(program);
        cpu.mem_write_u16(0xEF, 0x0001);
        cpu.mem_write_u8(0x11, 0x13);
        cpu.run();

        assert_eq!(cpu.acc, 0x13);

    }

    #[test]
    fn test_ldx() {

        let mut cpu = init_test_cpu();
        let program = vec![0xA2, 0xFF, 0x00];
        cpu.load_and_run(program);

        assert_eq!(cpu.x, 0xFF);
        assert!(cpu.is_flag_set(CPUFlags::NEGATIVE));
        assert!(!cpu.is_flag_set(CPUFlags::ZERO));

    }

    #[test]
    fn test_ldy() {

        let mut cpu = init_test_cpu();
        let program = vec![0xA0, 0xFF, 0x00];
        cpu.load_and_run(program);

        assert_eq!(cpu.y, 0xFF);
        assert!(cpu.is_flag_set(CPUFlags::NEGATIVE));
        assert!(!cpu.is_flag_set(CPUFlags::ZERO));

    }

    #[test]
    fn test_lsr_acc() {
        
        let mut cpu = init_test_cpu();
        let program = vec![0xA9, 0b0101_0101, 0x4A, 0x00];
        cpu.load_and_run(program);

        assert_eq!(cpu.acc, 0b0010_1010);
        assert!(cpu.is_flag_set(CPUFlags::CARRY));

        let program = vec![0xA9, 0b1010_1010, 0x4A, 0x00];
        cpu.load_and_run(program);

        assert_eq!(cpu.acc, 0b0101_0101);
        assert!(!cpu.is_flag_set(CPUFlags::CARRY));

    }

    #[test]
    fn test_lsr_mem() {
        
        let mut cpu = init_test_cpu();
        let program = vec![0xA9, 0b0101_0101, 0x4E, 0x01, 0x06, 0xAD, 0x01, 0x06, 0x00];
        cpu.load_and_run(program);

        assert_eq!(cpu.acc, 0b0010_1010);
        assert!(cpu.is_flag_set(CPUFlags::CARRY));

        let program = vec![0xA9, 0b1010_1010, 0x4E, 0x01, 0x06, 0xAD, 0x01, 0x06, 0x00];
        cpu.load_and_run(program);

        assert_eq!(cpu.acc, 0b0101_0101);
        assert!(!cpu.is_flag_set(CPUFlags::CARRY));

    }

    #[test]
    fn test_nop_official() {

        let mut cpu = init_test_cpu();
        let program = vec![0xEA, 0x00];
        cpu.load_and_run(program);

        assert_eq!(cpu.pc, 0x0602)

    }

    #[test]
    fn test_ora() {

        let mut cpu = init_test_cpu();
        let program = vec![0x09, 0b1010_1010, 0x49, 0b0101_0101];
        cpu.load_and_run(program);

        assert_eq!(cpu.acc, 0b1111_1111);
        assert!(cpu.is_flag_set(CPUFlags::NEGATIVE));

    }

    #[test]
    fn test_pha() {

        let mut cpu = init_test_cpu();
        let program = vec![0x48];

        cpu.reset();
        cpu.load(program);
        cpu.acc = 0xFF;
        cpu.run();

        assert_eq!(cpu.sp, 0xFE); // Byte has been pushed to stack
        assert_eq!(cpu.stack_pop_u8(), 0xFF);

    }

    #[test]
    fn test_php() {

        let mut cpu = init_test_cpu();
        let program = vec![0x08];

        cpu.reset();
        cpu.load(program);
        cpu.set_flag(CPUFlags::OVERFLOW);
        cpu.run();

        assert_eq!(cpu.sp, 0xFE); // Byte has been pushed to stack
        assert!(cpu.is_flag_set(CPUFlags::OVERFLOW));

    }

    #[test]
    fn test_pla() {

        let mut cpu = init_test_cpu();
        let program = vec![0x48, 0xA9, 0x11, 0x68];

        cpu.reset();
        cpu.load(program);
        cpu.acc = 0xFF;
        cpu.run();

        assert_eq!(cpu.sp, 0xFF); // Byte has been popped from stack
        assert_eq!(cpu.acc, 0xFF);
        assert!(cpu.is_flag_set(CPUFlags::NEGATIVE));

    }

    #[test]
    fn test_plp() {

        let mut cpu = init_test_cpu();
        let program = vec![0x08, 0x38, 0x28];

        cpu.reset();
        cpu.load(program);
        cpu.set_flag(CPUFlags::OVERFLOW);
        cpu.run();

        assert_eq!(cpu.sp, 0xFF); // Byte has been popped from stack
        assert!(cpu.is_flag_set(CPUFlags::OVERFLOW));
        assert!(!cpu.is_flag_set(CPUFlags::CARRY));

    }

    #[test]
    fn test_rol_acc() {

        let mut cpu = init_test_cpu();
        let program = vec![0xA9, 0b1010_1010, 0x2A];
        cpu.load_and_run(program);

        assert_eq!(cpu.acc, 0b0101_0100);
        assert!(cpu.is_flag_set(CPUFlags::CARRY));

        cpu.reset();

        let program = vec![0xA9, 0b0000_1111, 0x2A];
        cpu.load(program);
        cpu.set_flag(CPUFlags::CARRY);
        cpu.run();

        assert_eq!(cpu.acc, 0b0001_1111);
        assert!(!cpu.is_flag_set(CPUFlags::CARRY));

    }

    #[test]
    fn test_ror_acc() {

        let mut cpu = init_test_cpu();
        let program = vec![0xA9, 0b0101_0101, 0x6A];
        cpu.load(program);
        cpu.set_flag(CPUFlags::CARRY);
        cpu.run();

        assert_eq!(cpu.acc, 0b1010_1010);
        assert!(cpu.is_flag_set(CPUFlags::CARRY));

        cpu.reset();
        let program = vec![0xA9, 0b0101_0100, 0x6A];
        cpu.load_and_run(program);

        assert_eq!(cpu.acc, 0b0010_1010);
        assert!(!cpu.is_flag_set(CPUFlags::CARRY));

    }

    #[test]
    fn test_rol_mem() {

        let mut cpu = init_test_cpu();
        let program = vec![0x2E, 0x04, 0x06, 0x00, 0x10];
        cpu.load(program);
        cpu.set_flag(CPUFlags::CARRY);
        cpu.run();

        assert_eq!(cpu.mem_read_u8(0x0604), 0b0010_0001);
        assert!(!cpu.is_flag_set(CPUFlags::CARRY));

        let program = vec![0x2E, 0x04, 0x06, 0x00, 0b1000_1010];
        cpu.load_and_run(program);

        assert_eq!(cpu.mem_read_u8(0x0604), 0b0001_0100);
        assert!(cpu.is_flag_set(CPUFlags::CARRY));

    }

    #[test]
    fn test_ror_mem() {

        let mut cpu = init_test_cpu();
        let program = vec![0x6E, 0x04, 0x06, 0x00, 0x10];
        cpu.load(program);
        cpu.set_flag(CPUFlags::CARRY);
        cpu.run();

        assert_eq!(cpu.mem_read_u8(0x0604), 0b1000_1000);
        assert!(!cpu.is_flag_set(CPUFlags::CARRY));

        let program = vec![0x6E, 0x04, 0x06, 0x00, 0b0000_1011];
        cpu.load_and_run(program);

        assert_eq!(cpu.mem_read_u8(0x0604), 0b000_0101);
        assert!(cpu.is_flag_set(CPUFlags::CARRY));

    }

    #[test]
    fn test_run_sample_prog_1() {

        /*
            This program does the following:
            Load 0xC0 into the accumulator
            Transfer to the X register
            Increment X
         */

        let mut cpu = init_test_cpu();
        let program = vec![0xA9, 0xC0, 0xAA, 0xE8, 0x00];

        cpu.load_and_run(program);

        assert_eq!(cpu.acc, 0xC0);
        assert_eq!(cpu.x, 0xC1);

    }

    #[test]
    fn test_rti() {

        let mut cpu = init_test_cpu();
        let program = vec![0x40];
        cpu.reset();
        cpu.load(program);
        cpu.stack_push_u16(0x0F0F);
        cpu.stack_push_u8(0b1000_0001);
        cpu.run();

        assert_eq!(cpu.pc, 0x0F10);
        assert_eq!(cpu.sp, 0xFF);
        assert!(cpu.is_flag_set(CPUFlags::NEGATIVE));
        assert!(cpu.is_flag_set(CPUFlags::CARRY));

    }
    
    #[test]
    fn test_rts() {

        let mut cpu = init_test_cpu();
        let program = vec![0x60, 0xEF, 0xFE];
        cpu.reset();
        cpu.load(program);
        cpu.stack_push_u16(0x0602);
        cpu.run();

        assert_eq!(cpu.pc, 0x0604);
        assert_eq!(cpu.sp, 0xFF); // Stack should be empty now

    }

    #[test]
    fn test_sbc() {

        let mut cpu = init_test_cpu();
        let program = vec![0xA9, 0xF0, 0xE9, 0x0F, 0x00];
        cpu.load_and_run(program);

        assert_eq!(cpu.acc, 0xE0);

        cpu.reset();
        let program = vec![0xA9, 0x00, 0xE9, 0x01, 0x00];
        cpu.load_and_run(program);

    }

    #[test]
    fn test_sec() {

        let mut cpu = init_test_cpu();
        let program = vec![0x38];
        cpu.load_and_run(program);

        assert!(cpu.is_flag_set(CPUFlags::CARRY));

    }

    #[test]
    fn test_sed() {

        let mut cpu = init_test_cpu();
        let program = vec![0xF8];
        cpu.load_and_run(program);
        
        assert!(cpu.is_flag_set(CPUFlags::DECIMAL_MODE));

    }

    #[test]
    fn test_sei() {

        let mut cpu = init_test_cpu();
        let program = vec![0x78];
        cpu.load_and_run(program);
        
        assert!(cpu.is_flag_set(CPUFlags::INTERRUPT_DISABLE));

    }

    #[test]
    fn test_sta() {

        let mut cpu = init_test_cpu();
        let program = vec![0xA9, 0x13, 0x8D, 0xFF, 0x06];
        cpu.load_and_run(program);
    
        assert_eq!(cpu.mem_read_u8(0x06FF), 0x13);

    }

    #[test]
    fn test_stx() {

        let mut cpu = init_test_cpu();
        let program = vec![0xA2, 0x13, 0x8E, 0xFF, 0x06];
        cpu.load_and_run(program);
    
        assert_eq!(cpu.mem_read_u8(0x06FF), 0x13);

    }

    #[test]
    fn test_sty() {

        let mut cpu = init_test_cpu();
        let program = vec![0xA0, 0x13, 0x8C, 0xFF, 0x06];
        cpu.load_and_run(program);
    
        assert_eq!(cpu.mem_read_u8(0x06FF), 0x13);

    }

    #[test]
    fn test_tax () {

        let mut cpu = init_test_cpu();
        cpu.acc = 156;

        let program = vec![0xAA, 0x00];
        cpu.load(program);
        cpu.run();

        assert_eq!(cpu.x, 156);
        assert!(cpu.is_flag_set(CPUFlags::NEGATIVE));

    }

    #[test]
    fn test_tay () {

        let mut cpu = init_test_cpu();
        cpu.acc = 156;

        let program = vec![0xA8, 0x00];
        cpu.load(program);
        cpu.run();

        assert_eq!(cpu.y, 156);
        assert!(cpu.is_flag_set(CPUFlags::NEGATIVE));

    }

    #[test]
    fn test_tsx () {

        let mut cpu = init_test_cpu();
        cpu.sp = 156;

        let program = vec![0xBA, 0x00];
        cpu.load(program);
        cpu.run();

        assert_eq!(cpu.x, 156);
        assert!(cpu.is_flag_set(CPUFlags::NEGATIVE));

    }

    #[test]
    fn test_txa () {

        let mut cpu = init_test_cpu();
        cpu.x = 156;

        let program = vec![0x8A, 0x00];
        cpu.load(program);
        cpu.run();

        assert_eq!(cpu.acc, 156);
        assert!(cpu.is_flag_set(CPUFlags::NEGATIVE));

    }

    #[test]
    fn test_txs () {

        let mut cpu = init_test_cpu();
        cpu.x = 156;

        let program = vec![0x9A, 0x00];
        cpu.load(program);
        cpu.run();

        assert_eq!(cpu.sp, 156);

    }

    #[test]
    fn test_tya () {

        let mut cpu = init_test_cpu();
        cpu.y = 156;

        let program = vec![0x98, 0x00];
        cpu.load(program);
        cpu.run();

        assert_eq!(cpu.acc, 156);
        assert!(cpu.is_flag_set(CPUFlags::NEGATIVE));

    }

    #[test]
    fn test_inx () {

        let mut cpu = init_test_cpu();
        cpu.x = 127;
        cpu.set_negative_and_zero_flags(cpu.x);
        assert!(!cpu.is_flag_set(CPUFlags::NEGATIVE));

        let program = vec![0xE8, 0x00];
        cpu.load(program);
        cpu.run();

        assert_eq!(cpu.x, 128);
        assert!(cpu.is_flag_set(CPUFlags::NEGATIVE));

    }

    #[test]
    fn test_iny () {

        let mut cpu = init_test_cpu();
        cpu.y = 127;
        cpu.set_negative_and_zero_flags(cpu.y);
        assert!(!cpu.is_flag_set(CPUFlags::NEGATIVE));

        let program = vec![0xC8, 0x00];
        cpu.load(program);
        cpu.run();

        assert_eq!(cpu.y, 128);
        assert!(cpu.is_flag_set(CPUFlags::NEGATIVE));

    }

    #[test]
    fn test_dex () {

        let mut cpu = init_test_cpu();
        cpu.x = 128;
        cpu.set_negative_and_zero_flags(cpu.x);
        assert!(cpu.is_flag_set(CPUFlags::NEGATIVE));

        let program = vec![0xCA, 0x00];
        cpu.load(program);
        cpu.run();

        assert_eq!(cpu.x, 127);
        assert!(!cpu.is_flag_set(CPUFlags::NEGATIVE));

    }

    #[test]
    fn test_dey () {

        let mut cpu = init_test_cpu();
        cpu.y = 128;
        cpu.set_negative_and_zero_flags(cpu.y);
        assert!(cpu.is_flag_set(CPUFlags::NEGATIVE));

        let program = vec![0x88, 0x00];
        cpu.load(program);
        cpu.run();

        assert_eq!(cpu.y, 127);
        assert!(!cpu.is_flag_set(CPUFlags::NEGATIVE));

    }

}