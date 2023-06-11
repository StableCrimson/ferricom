# Ferricom

Attempting to write a NES emulator in Rust

I have never written an emulator before, and Rust is a fairly new language to me. So I am very much
a beginner. This isn't a project where finishing is the goal, the main point is to learn.
If this project helps you write your own emulator or learn Rust, then even better.

## CPU Status

57/256 instructions implemented

- [ ] Stack implemented
- [ ] Detecting if a page is crossed
- [ ] Extra cycle on crossed page for certain instructions
- [ ] Cycle accuracy tests
- [ ] Logging

## Resources

- [Rust NES Emu Blog](https://bugzmanov.github.io/nes_ebook/)
- [6502 Instruction set](https://www.nesdev.org/obelisk-6502-guide/instructions.html)
- [6502 Status flags](https://www.nesdev.org/wiki/Status_flags)
- [6502 Addressing modes](https://www.nesdev.org/obelisk-6502-guide/addressing.html)
- [iNES File Format Spec](https://www.nesdev.org/wiki/INES#Flags_6)
