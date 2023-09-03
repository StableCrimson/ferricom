pub mod registers;

use log::{error, warn};

use crate::rom::ScreenMirroring;
use crate::ppu::registers::address_register::AddressRegister;
use crate::ppu::registers::control_register::ControlRegister;
use crate::ppu::registers::status_register::StatusRegister;

const CHR_ROM_BEGIN: u16 =        0;
const CHR_ROM_END: u16 =          0x1FFF;
const VRAM_MIRROR_BEGIN: u16 =    0x2000;
const VRAM_MIRROR_END: u16 =      0x2FFF;
const PALETTE_TABLE_BEGIN: u16 =  0x3F00;
const PALETTE_TABLE_END: u16 =    0x3FFF;

pub struct PPU {
  chr_rom: Vec<u8>,
  palette_table: [u8; 32],
  oam_data: [u8; 256],
  vram: [u8; 2048],
  screen_mirroring: ScreenMirroring,
  addr: AddressRegister,
  control: ControlRegister,
  status: StatusRegister,
  internal_data_buffer: u8,
  scanline: u16,
  cycles: usize
}

impl PPU {

  pub fn new(chr_rom: Vec<u8>, screen_mirroring: ScreenMirroring) -> PPU {
    PPU {
      chr_rom,
      palette_table: [0; 32],
      oam_data: [0; 256],
      vram: [0; 2048],
      screen_mirroring,
      addr: AddressRegister::new(),
      control: ControlRegister::new(),
      status: StatusRegister::new(),
      internal_data_buffer: 0,
      scanline: 0,
      cycles: 21
    }
  }

  pub fn tick(&mut self, cycles: u8) -> bool {

    self.cycles += cycles as usize;

    if self.cycles >= 341 {

      self.cycles -= 341;
      self.scanline += 1;

      if self.scanline == 241 {
        if self.control.should_generate_vblank_nmi() {
          self.status.set_vblank_status(true);
          todo!("Should trigger a non-maskable interrupt");
        }
      }

      if self.scanline >= 262 {
        self.scanline = 0;
        self.status.reset_vblank_status();
        return true;
      }

    }

    false

  }

  pub fn write_to_ppu_address(&mut self, data: u8) {
    self.addr.update(data);
  }

  pub fn update_ctrl_register(&mut self, data: u8) {
    self.control.update(data);
  }

  fn increment_vram_addr(&mut self) {
    self.addr.increment(self.control.get_vram_addr_increment());
  }

  pub fn read_data(&mut self) -> u8 {

    let addr = self.addr.get();
    self.increment_vram_addr();

    match addr {
      0..=0x1FFF => {
        let result = self.internal_data_buffer;
        self.internal_data_buffer = self.chr_rom[addr as usize];
        result
      },
      0x2000..=0x2FFF => {
        let result = self.internal_data_buffer;
        self.internal_data_buffer = self.vram[self.mirror_vram_addr(addr) as usize];
        result
      },
      0x3000..=0x3EFF => panic!("Address space 0x3000..0x3EFF is not expected to be used"),
      0x3F00..=0x3FFF => self.palette_table[(addr-0x3F00) as usize],
      _ => panic!("Unexpected access to mirrored adddress space")
    }

  }

  pub fn write_to_data_register(&mut self, data: u8) {
    
    let target_addr = self.addr.get();

    match target_addr {
      CHR_ROM_BEGIN..=CHR_ROM_END => {
        warn!("Attempted to write to character rom address space: 0x{:0X}", target_addr);
      },
      VRAM_MIRROR_BEGIN..=VRAM_MIRROR_END => {
        self.vram[self.mirror_vram_addr(target_addr) as usize] = data;
      },
      0x3F10 | 0x3F14 | 0x3F18 | 0x3F1C => {
        let mirror_address = target_addr - 0x10;
        self.palette_table[(mirror_address - 0x3F00) as usize] = data;
      },
      PALETTE_TABLE_BEGIN..=PALETTE_TABLE_END => {
        self.palette_table[(target_addr - 0x3F00) as usize] = data;
      }
      _ => {
        error!("Unable to access mirrored address space: 0x{:0X}", target_addr);
        panic!("Unable to access mirrored address space: 0x{:0X}", target_addr);
      }
    }

  }

  pub fn mirror_vram_addr(&self, addr: u16) -> u16 {

    let mirrored_addr = addr & 0b0010_1111_1111_1111;
    let vram_index = mirrored_addr - 0x2000;
    let name_table = vram_index / 0x0400;

    match (&self.screen_mirroring, name_table) {
      (ScreenMirroring::Vertical, 2) => vram_index - 0x0800,
      (ScreenMirroring::Vertical, 3) => vram_index - 0x0800,
      (ScreenMirroring::Horizontal, 1) => vram_index - 0x0400,
      (ScreenMirroring::Horizontal, 2) => vram_index - 0x0400,
      (ScreenMirroring::Horizontal, 3) => vram_index - 0x0800,
      _ => vram_index
    }
  }
}