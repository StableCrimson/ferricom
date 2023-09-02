use std::collections::HashMap;
use log::error;

use crate::cpu::{CPU, Mem, AddressingMode};
use crate::instructions::{Instruction, CPU_INSTRUCTION_SET};

pub fn trace(cpu: &CPU) -> String {
        
  let opcodes: &HashMap<u8, &'static Instruction> = &CPU_INSTRUCTION_SET;

  // TODO: Remove the match statement once all 256 opcodes are implemented
  let code = cpu.mem_read_u8(cpu.pc);
  let opcode = match opcodes.get(&code) {
    Some(ins) => ins,
    None => {
        error!("Instruction 0x{:0X} is invalid or unimplemented", code);
        panic!("Instruction 0x{:0X} is invalid or unimplemented", code);
    }
  };

  let begin = cpu.pc;
  let mut hex_dump = vec![];
  hex_dump.push(code);

  let (mem_addr, stored_value) = match opcode.addressing_mode {
      AddressingMode::Immediate | AddressingMode::None | AddressingMode::Implied | AddressingMode::Relative => (0, 0),
      _ => {
          let (addr, _) = cpu.get_absolute_address(&opcode.addressing_mode, begin+1);
          (addr, cpu.mem_read_u8(addr))
      }
  };

  let tmp = match opcode.bytes {
      1 => match opcode.opcode {
          0x0a | 0x4a | 0x2a | 0x6a => "A ".to_string(),
          _ => String::from(""),
      },
      2 => {
          let address: u8 = cpu.mem_read_u8(begin + 1);
          hex_dump.push(address);

          match opcode.addressing_mode {
              AddressingMode::Immediate => format!("#${:02x}", address),
              AddressingMode::ZeroPage => format!("${:02x} = {:02x}", mem_addr, stored_value),
              AddressingMode::ZeroPageX => format!(
                  "${:02x},X @ {:02x} = {:02x}",
                  address, mem_addr, stored_value
              ),
              AddressingMode::ZeroPageY => format!(
                  "${:02x},Y @ {:02x} = {:02x}",
                  address, mem_addr, stored_value
              ),
              AddressingMode::IndirectX => format!(
                  "(${:02x},X) @ {:02x} = {:04x} = {:02x}",
                  address,
                  (address.wrapping_add(cpu.x)),
                  mem_addr,
                  stored_value
              ),
              AddressingMode::IndirectY => format!(
                  "(${:02x}),Y = {:04x} @ {:04x} = {:02x}",
                  address,
                  (mem_addr.wrapping_sub(cpu.y as u16)),
                  mem_addr,
                  stored_value
              ),
              AddressingMode::None | AddressingMode::Implied | AddressingMode::Relative => {
                  // assuming local jumps: BNE, BVS, etc....
                  let address: usize =
                      (begin as usize + 2).wrapping_add((address as i8) as usize);
                  format!("${:04x}", address)
              }

              _ => panic!(
                  "unexpected addressing mode {:?} has opcode-len 2. code {:02x}",
                  opcode.addressing_mode, opcode.opcode
              ),
          }
      }
      3 => {
          let address_lo = cpu.mem_read_u8(begin + 1);
          let address_hi = cpu.mem_read_u8(begin + 2);
          hex_dump.push(address_lo);
          hex_dump.push(address_hi);

          let address = cpu.mem_read_u16(begin + 1);

          match opcode.addressing_mode {
              AddressingMode::None | AddressingMode::Implied | AddressingMode::Relative | AddressingMode::Indirect => {
                  if opcode.opcode == 0x6c {
                      //jmp indirect
                      let jmp_addr = if address & 0x00FF == 0x00FF {
                          let lo = cpu.mem_read_u8(address);
                          let hi = cpu.mem_read_u8(address & 0xFF00);
                          (hi as u16) << 8 | (lo as u16)
                      } else {
                          cpu.mem_read_u16(address)
                      };

                      // let jmp_addr = cpu.mem_read_u16(address);
                      format!("(${:04x}) = {:04x}", address, jmp_addr)
                  } else {
                      format!("${:04x}", address)
                  }
              }
              AddressingMode::Absolute => {
              
                if (opcode.opcode == 0x4C) | (opcode.opcode == 0x20) {
                  format!("${:04x}", mem_addr)
                } else {
                  format!("${:04x} = {:02x}", mem_addr, stored_value)
                }
              },
              AddressingMode::AbsoluteX => format!(
                  "${:04x},X @ {:04x} = {:02x}",
                  address, mem_addr, stored_value
              ),
              AddressingMode::AbsoluteY => format!(
                  "${:04x},Y @ {:04x} = {:02x}",
                  address, mem_addr, stored_value
              ),
              _ => panic!(
                  "unexpected addressing mode {:?} has opcode-len 3. code {:02x}",
                  opcode.addressing_mode, opcode.opcode
              ),
          }
      }
      _ => String::from(""),
  };

  let hex_str = hex_dump
      .iter()
      .map(|z| format!("{:02x}", z))
      .collect::<Vec<String>>()
      .join(" ");
  let asm_str = format!("{:04x}  {:8} {: >4} {}", begin, hex_str, opcode.ins, tmp)
      .trim()
      .to_string();

  format!(
      "{:47} A:{:02x} X:{:02x} Y:{:02x} P:{:02x} SP:{:02x}",
      asm_str, cpu.acc, cpu.x, cpu.y, cpu.status, cpu.sp
  )
  .to_ascii_uppercase()
}

#[cfg(test)]
mod test {
   use super::*;
   use crate::bus::Bus;
   use crate::rom::tests::test_rom;

   #[test]
   fn test_format_trace() {
       let mut bus = Bus::new(test_rom());
       bus.mem_write_u8(100, 0xa2);
       bus.mem_write_u8(101, 0x01);
       bus.mem_write_u8(102, 0xca);
       bus.mem_write_u8(103, 0x88);
       bus.mem_write_u8(104, 0x00);

       let mut cpu = CPU::new(bus);
       cpu.pc = 0x64;
       cpu.acc = 1;
       cpu.x = 2;
       cpu.y = 3;
       let mut result: Vec<String> = vec![];
       cpu.run_with_callback(|cpu| {
           result.push(trace(cpu));
       });
       assert_eq!(
           "0064  A2 01     LDX #$01                        A:01 X:02 Y:03 P:24 SP:FD",
           result[0]
       );
       assert_eq!(
           "0066  CA        DEX                             A:01 X:01 Y:03 P:24 SP:FD",
           result[1]
       );
       assert_eq!(
           "0067  88        DEY                             A:01 X:00 Y:03 P:26 SP:FD",
           result[2]
       );
   }

   #[test]
   fn test_format_mem_access() {
       let mut bus = Bus::new(test_rom());
       // ORA ($33), Y
       bus.mem_write_u8(100, 0x11);
       bus.mem_write_u8(101, 0x33);


       //data
       bus.mem_write_u8(0x33, 00);
       bus.mem_write_u8(0x34, 04);

       //target cell
       bus.mem_write_u8(0x400, 0xAA);

       let mut cpu = CPU::new(bus);
       cpu.pc = 0x64;
       cpu.y = 0;
       let mut result: Vec<String> = vec![];
       cpu.run_with_callback(|cpu| {
           result.push(trace(cpu));
       });
       assert_eq!(
           "0064  11 33     ORA ($33),Y = 0400 @ 0400 = AA  A:00 X:00 Y:00 P:24 SP:FD",
           result[0]
       );
   }
}