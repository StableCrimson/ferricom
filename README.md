# Ferricom

![Unit Tests and Coverage](https://github.com/StableCrimson/ferricom/actions/workflows/test_coverage.yml/badge.svg?event=push)

Attempting to write a NES emulator in Rust

I have never written an emulator before, and Rust is a fairly new language to me. So I am very much
a beginner. This isn't a project where finishing is the goal, the main point is to learn.
If this project helps you write your own emulator or learn Rust, then even better.

## Roadmap (WIP)

- [ ] CPU and all instructions
- [x] Memory into its own module
- [ ] ROM loading
- [ ] PPU (Basic)
- [ ] Input (Basic)
- [ ] APU
- [ ] PPU (Advanced)
- [ ] Build for multiple platforms
- [ ] GUI
- [ ] Input remapping / gamepad detection
- [ ] Save files and save states

## CPU Status

227/256 opcodes implemented

- [x] 151 Official opcodes
- [ ] 105 Illegal opcodes
- [x] Stack implemented
- [x] Detecting if a page is crossed
- [x] Branching instructions
- [ ] Extra cycle on crossed page for certain instructions
- [x] Logging (Needed at this stage for test ROMs)
- [ ] Cycle accuracy tests
- [ ] Passes test ROMs (Instruction set)
- [x] CLI arg parsing

### Misc To-Do

- [ ] Inline docs and tests for `cargo doc`
- [ ] Disassembler
- [ ] Actions also run linting and formatting (Once the emulator is actually usable)
- [ ] Semver
- [ ] Add usage docs

### Resources

- [Rust NES Emu Blog](https://bugzmanov.github.io/nes_ebook/)
- [6502 Instruction set](https://www.nesdev.org/obelisk-6502-guide/instructions.html)
- [6502 Status flags](https://www.nesdev.org/wiki/Status_flags)
- [6502 Addressing modes](https://www.nesdev.org/obelisk-6502-guide/addressing.html)
- [6502 Stack](https://www.nesdev.org/wiki/Stack)
- [iNES File Format Spec](https://www.nesdev.org/wiki/INES#Flags_6)
- [NES Screen Mirroring](https://www.nesdev.org/wiki/Mirroring)
