use crate::{cpu::Mem, ppu::PPU};
use crate::rom::ROM;
use log::{debug, error};

const RAM_START: u16 =                0x0000;
const RAM_MIRROR_END: u16 =           0x1FFF;
const PPU_REGISTER_START: u16 =       0x2000;
const PPU_REGISTER_MIRROR_END: u16 =  0x3FFF;
const ROM_SPACE_START: u16 =          0x8000;
const ROM_SPACE_END: u16 =            0xFFFF;

/// Write-only registers
const PPU_CONTROL_BYTE: u16 =         0x2000;
const PPU_MASK_REGISTER: u16 =        0x2001;
const PPU_SCROLL_BYTE: u16 =          0x2005;

/// Read-only register for reporting PPU status
const PPU_STATUS_REGISTER: u16 =      0x2002;

/// Object Attribute Memory, keeps the state of sprites
const PPU_OAM_ADDRESS_REGISTER: u16 = 0x2003;
const PPU_OAM_DATA_REGISTER: u16 =    0x2004;

/// For accessing PPU memory map
const PPU_ADDRESS_REGISTER: u16 =     0x2006;
const PPU_DATA_REGISTER: u16 =        0x2007;

/// Direct-Memory Access, for quickly writing 256 bytes
/// from RAM to OAM
const PPU_DMA_ADDRESS: u16 =          0x4014;

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

  fn mem_read_u8(&mut self, addr: u16) -> u8 {
    match addr {
      RAM_START..=RAM_MIRROR_END => {
        let mirred_addr = addr & 0b0000_0111_1111_1111;
        self.cpu_vram[mirred_addr as usize]
      },
      PPU_CONTROL_BYTE | PPU_MASK_REGISTER | PPU_OAM_ADDRESS_REGISTER | PPU_SCROLL_BYTE | PPU_ADDRESS_REGISTER | PPU_DMA_ADDRESS => {
        error!("Attempted to read from write-only PPU address 0x{:0X}", addr);
        panic!("Attempted to read from write-only PPU address 0x{:0X}", addr);
      },
      PPU_DATA_REGISTER => self.ppu.read_data(),
      0x2008..=PPU_REGISTER_MIRROR_END => {
        let mirrored_addr = addr & 0b0010_0000_0000_0111;
        self.mem_read_u8(mirrored_addr)
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
      PPU_CONTROL_BYTE => self.ppu.update_ctrl_register(data),
      PPU_ADDRESS_REGISTER => self.ppu.write_to_ppu_address(data),
      PPU_DATA_REGISTER => self.ppu.write_to_data_register(data),
      0x2008..=PPU_REGISTER_MIRROR_END => {
        let mirrored_addr = addr & 0b0010_0000_0000_0111;
        self.mem_write_u8(mirrored_addr, data);
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