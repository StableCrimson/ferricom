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

    Instruction::new(0xA9, "LDA", 2, 2, AddressingMode::Immediate),
    Instruction::new(0xA5, "LDA", 2, 3, AddressingMode::ZeroPage), // TODO: Write test
    Instruction::new(0xB5, "LDA", 2, 4, AddressingMode::ZeroPageX), // TODO: Write test
    Instruction::new(0xAD, "LDA", 3, 4, AddressingMode::Absolute),
    Instruction::new(0xBD, "LDA", 3, 4, AddressingMode::AbsoluteX), // TODO: Write test
    Instruction::new(0xB9, "LDA", 3, 4, AddressingMode::AbsoluteY), // TODO: Write test
    Instruction::new(0xA1, "LDA", 2, 6, AddressingMode::IndirectX), // TODO: Write test
    Instruction::new(0xB1, "LDA", 2, 5, AddressingMode::IndirectY), // TODO: Write test

    Instruction::new(0x18, "CLC", 1, 2, AddressingMode::Implied),
    Instruction::new(0xD8, "CLD", 1, 2, AddressingMode::Implied),
    Instruction::new(0x58, "CLI", 1, 2, AddressingMode::Implied),
    Instruction::new(0xB8, "CLV", 1, 2, AddressingMode::Implied),

    Instruction::new(0xCA, "DEX", 1, 2, AddressingMode::Implied),
    Instruction::new(0x88, "DEY", 1, 2, AddressingMode::Implied),

    Instruction::new(0xE8, "INX", 1, 2, AddressingMode::Implied),
    Instruction::new(0xC8, "INY", 1, 2, AddressingMode::Implied),

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