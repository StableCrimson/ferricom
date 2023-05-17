# Ferriscom

Attempting to write a NES emulator in Rust

I have never written an emulator before, and Rust is a fairly new language to me. So I am very much
a beginner. This isn't a project where finishing is the goal, the main point is to learn.
If this project helps you write your own emulator or learn Rust, then even better.

## Currently Implemented Instructions

| Name          | Opcode | Descrition                                                |
| :------------ | :----: | --------------------------------------------------------- |
| BRK           | `0x00` | Force Interrupt                                           |
| LDA Immediate | `0xA9` | Load value into the accumulator with immediate addressing |
| TAX           | `0xAA` | Copy value from accumulator into the X register           |

## Resources

- [6502 Instruction set](https://www.nesdev.org/obelisk-6502-guide/instructions.html)
- [6502 Status flags](https://www.nesdev.org/wiki/Status_flags)
