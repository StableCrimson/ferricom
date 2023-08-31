use crate::{cpu::Mem, ppu::PPU};
use crate::rom::ROM;
use log::{debug, warn, error};

const RAM_START: u16 = 0;
const RAM_MIRROR_END: u16 = 0x1FFF;
const PPU_REGISTER_START: u16 = 0x2000;
const PPU_REGISTER_MIRROR_END: u16 = 0x3FFF;
const ROM_SPACE_START: u16 = 0x8000;
const ROM_SPACE_END: u16 = 0xFFFF;
pub struct Bus {
  cpu_vram: [u8; 2048],
  prg_rom: Vec<u8>,
  ppu: PPU
}

impl Bus {

  pub fn new(rom: ROM) -> Self {
    Bus {
      cpu_vram: [0; 2048],
      prg_rom: rom.prg_rom,
      ppu: PPU::new(rom.chr_rom, rom.mirroring)
    }
  }

  pub fn read_prg_rom(&self, mut addr: u16) -> u8 {

    addr -= 0x8000;

    if self.prg_rom.len() == 0x4000 && addr >= 0x4000 {
      addr %= 0x4000;
    }

    self.prg_rom[addr as usize]

  }

}

impl Mem for Bus {

  fn mem_read_u8(&self, addr: u16) -> u8 {
    match addr {
      RAM_START..=RAM_MIRROR_END => {
        let mirred_addr = addr & 0b0000_0111_1111_1111;
        self.cpu_vram[mirred_addr as usize]
      },
      PPU_REGISTER_START..=PPU_REGISTER_MIRROR_END => {
        warn!("PPU hasn't been implemented");
        todo!("PPU hasn't been implemented");
      },
      ROM_SPACE_START..=ROM_SPACE_END => {
        self.read_prg_rom(addr)
      },
      _ => {
        debug!("Ignoring memory read at 0x{:0X}", addr);
        0
      }
    }
  }

  fn mem_write_u8(&mut self, addr: u16, data: u8) {
    match addr {
      RAM_START..=RAM_MIRROR_END => {
        let mirrored_addr = addr & 0b0000_0111_1111_1111;
        self.cpu_vram[mirrored_addr as usize] = data;
      },
      PPU_REGISTER_START..=PPU_REGISTER_MIRROR_END => {
        let _mirrored_addr = addr & 0b0010_0000_0000_0111;
        todo!("PPU hasn't been implemented");
      },
      ROM_SPACE_START..=ROM_SPACE_END => {
        let msg = "Attempted to write to ROM address space!";
        error!("{msg}");
        panic!("{msg}");
      },
      _ => {
        debug!("Ignoring memory access at 0x{:0X}", addr);
      }
    }    
  }
}