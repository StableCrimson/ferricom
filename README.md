# Ferricom

Attempting to write a NES emulator in Rust

I have never written an emulator before, and Rust is a fairly new language to me. So I am very much
a beginner. This isn't a project where finishing is the goal, the main point is to learn.
If this project helps you write your own emulator or learn Rust, then even better.

## Currently Implemented Instructions

| Name          | Opcode | Descrition                                                |
| :------------ | :----: | --------------------------------------------------------- |
| BRK           | `0x00` | Force Interrupt                                           |
| LDA Immediate | `0xA9` | Load value into the accumulator with immediate addressing |
| CLC           | `0x18` | Clear the carry flag                                      |
| CLD           | `0xD8` | Clear the decimal mode flag                               |
| CLI           | `0x58` | Clear the interrupt disable flag                          |
| CLV           | `0xB8` | Clear the overflow flag                                   |
| TAX           | `0xAA` | Copy value from accumulator into the X register           |
| TAY           | `0xA8` | Copy value from accumulator into the Y register           |
| TSX           | `0xBA` | Copy value from the stack pointer into the X register     |
| TXA           | `0x8A` | Copy value from the X register into the accumulator       |
| TXS           | `0x9A` | Copy value from the X register into the stack pointer     |
| TYA           | `0x98` | Copy value from the Y register into the accumulator       |
| INX           | `0xE8` | Increment the value in the X register                     |
| INY           | `0xC8` | Increment the value in the Y register                     |
| DEX           | `0xCA` | Decrement the value in the X register                     |
| DEY           | `0x88` | Decrement the value in the Y register                     |

## Resources

- [6502 Instruction set](https://www.nesdev.org/obelisk-6502-guide/instructions.html)
- [6502 Status flags](https://www.nesdev.org/wiki/Status_flags)
