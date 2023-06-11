use crate::cpu::AddressingMode;
use lazy_static::lazy_static;
use std::collections::HashMap;

pub struct Instruction {
  pub opcode: u8,
  pub ins: &'static str,
  pub bytes: u8,
  pub cycles: u8,
  pub addressing_mode: AddressingMode
}

impl Instruction {
  pub fn new(opcode: u8, ins: &'static str, bytes: u8, cycles: u8, addressing_mode: AddressingMode) -> Instruction {
    Instruction {
      opcode,
      ins,
      bytes,
      cycles,
      addressing_mode
    }
  }
}

lazy_static! {

  pub static ref CPU_INSTRUCTIONS: Vec<Instruction> = vec![
    Instruction::new(0x00, "BRK", 1, 7, AddressingMode::Implied),

    Instruction::new(0x29, "AND", 2, 2, AddressingMode::Immediate),
    Instruction::new(0x25, "AND", 2, 3, AddressingMode::ZeroPage),
    Instruction::new(0x35, "AND", 2, 4, AddressingMode::ZeroPageX),
    Instruction::new(0x2D, "AND", 3, 4, AddressingMode::Absolute),
    Instruction::new(0x3D, "AND", 3, 4, AddressingMode::AbsoluteX), // TODO: +1 cpu cycle if page is crossed
    Instruction::new(0x39, "AND", 3, 4, AddressingMode::AbsoluteY), // TODO: +1 cpu cycle if page is crossed
    Instruction::new(0x21, "AND", 2, 6, AddressingMode::IndirectX),
    Instruction::new(0x31, "AND", 2, 5, AddressingMode::IndirectY), // TODO: +1 cpu cycle if page is crossed

    Instruction::new(0x24, "BIT", 2, 3, AddressingMode::ZeroPage),
    Instruction::new(0x2C, "BIT", 3, 4, AddressingMode::Absolute),

    Instruction::new(0x18, "CLC", 1, 2, AddressingMode::Implied),
    Instruction::new(0xD8, "CLD", 1, 2, AddressingMode::Implied),
    Instruction::new(0x58, "CLI", 1, 2, AddressingMode::Implied),
    Instruction::new(0xB8, "CLV", 1, 2, AddressingMode::Implied),

    Instruction::new(0xC9, "CMP", 2, 2, AddressingMode::Immediate),
    Instruction::new(0xC5, "CMP", 2, 3, AddressingMode::ZeroPage),
    Instruction::new(0xD5, "CMP", 2, 4, AddressingMode::ZeroPageX),
    Instruction::new(0xCD, "CMP", 3, 4, AddressingMode::Absolute),
    Instruction::new(0xDD, "CMP", 3, 4, AddressingMode::AbsoluteX), // TODO: +1 cpu cycle if page is crossed
    Instruction::new(0xD9, "CMP", 3, 4, AddressingMode::AbsoluteY), // TODO: +1 cpu cycle if page is crossed
    Instruction::new(0xC1, "CMP", 2, 6, AddressingMode::IndirectX),
    Instruction::new(0xD1, "CMP", 2, 5, AddressingMode::IndirectY), // TODO: +1 cpu cycle if page is crossed

    Instruction::new(0xE0, "CPX", 2, 2, AddressingMode::Immediate),
    Instruction::new(0xE4, "CPX", 2, 3, AddressingMode::ZeroPage),
    Instruction::new(0xEC, "CPX", 3, 4, AddressingMode::Absolute),

    Instruction::new(0xC0, "CPY", 2, 2, AddressingMode::Immediate),
    Instruction::new(0xC4, "CPY", 2, 3, AddressingMode::ZeroPage),
    Instruction::new(0xCC, "CPY", 3, 4, AddressingMode::Absolute),

    Instruction::new(0xCA, "DEX", 1, 2, AddressingMode::Implied),
    Instruction::new(0x88, "DEY", 1, 2, AddressingMode::Implied),

    Instruction::new(0xE8, "INX", 1, 2, AddressingMode::Implied),
    Instruction::new(0xC8, "INY", 1, 2, AddressingMode::Implied),

    Instruction::new(0xA9, "LDA", 2, 2, AddressingMode::Immediate),
    Instruction::new(0xA5, "LDA", 2, 3, AddressingMode::ZeroPage),
    Instruction::new(0xB5, "LDA", 2, 4, AddressingMode::ZeroPageX),
    Instruction::new(0xAD, "LDA", 3, 4, AddressingMode::Absolute),
    Instruction::new(0xBD, "LDA", 3, 4, AddressingMode::AbsoluteX), // TODO: +1 cpu cycle if page is crossed
    Instruction::new(0xB9, "LDA", 3, 4, AddressingMode::AbsoluteY), // TODO: +1 cpu cycle if page is crossed
    Instruction::new(0xA1, "LDA", 2, 6, AddressingMode::IndirectX),
    Instruction::new(0xB1, "LDA", 2, 5, AddressingMode::IndirectY), // TODO: +1 cpu cycle if page is crossed

    Instruction::new(0xA2, "LDX", 2, 2, AddressingMode::Immediate),
    Instruction::new(0xA6, "LDX", 2, 3, AddressingMode::ZeroPage),
    Instruction::new(0xB6, "LDX", 2, 4, AddressingMode::ZeroPageY),
    Instruction::new(0xAE, "LDX", 3, 4, AddressingMode::Absolute),
    Instruction::new(0xBE, "LDX", 3, 4, AddressingMode::AbsoluteY), // TODO: +1 cpu cycle if page is crossed

    Instruction::new(0xA0, "LDY", 2, 2, AddressingMode::Immediate),
    Instruction::new(0xA4, "LDY", 2, 3, AddressingMode::ZeroPage),
    Instruction::new(0xB4, "LDY", 2, 4, AddressingMode::ZeroPageX),
    Instruction::new(0xAC, "LDY", 3, 4, AddressingMode::Absolute),
    Instruction::new(0xBC, "LDY", 3, 4, AddressingMode::AbsoluteX), // TODO: +1 cpu cycle if page is crossed

    Instruction::new(0xEA, "NOP", 1, 2, AddressingMode::Implied),

    Instruction::new(0x38, "SEC", 1, 2, AddressingMode::Implied),
    Instruction::new(0xF8, "SED", 1, 2, AddressingMode::Implied),
    Instruction::new(0x78, "SEI", 1, 2, AddressingMode::Implied),

    Instruction::new(0x85, "STA", 2, 3, AddressingMode::ZeroPage),
    Instruction::new(0x95, "STA", 2, 4, AddressingMode::ZeroPageX),
    Instruction::new(0x8D, "STA", 3, 4, AddressingMode::Absolute),
    Instruction::new(0x9D, "STA", 3, 5, AddressingMode::AbsoluteX),
    Instruction::new(0x99, "STA", 3, 5, AddressingMode::AbsoluteY),
    Instruction::new(0x81, "STA", 2, 6, AddressingMode::IndirectX),
    Instruction::new(0x91, "STA", 2, 6, AddressingMode::IndirectY),

    Instruction::new(0xAA, "TAX", 1, 2, AddressingMode::Implied),
    Instruction::new(0xA8, "TAY", 1, 2, AddressingMode::Implied),
    Instruction::new(0xBA, "TSX", 1, 2, AddressingMode::Implied),
    Instruction::new(0x8A, "TXA", 1, 2, AddressingMode::Implied),
    Instruction::new(0x98, "TYA", 1, 2, AddressingMode::Implied),
    Instruction::new(0x9A, "TXS", 1, 2, AddressingMode::Implied),
  ];

  pub static ref CPU_INSTRUCTION_SET: HashMap<u8, &'static Instruction> = {
    let mut map = HashMap::new();
    for ins in &*CPU_INSTRUCTIONS {
      map.insert(ins.opcode, ins);
    }
    map
  };

}