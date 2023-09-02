use log::{error, warn};

use crate::rom::ScreenMirroring;

pub struct AddressRegister {
  value: (u8, u8),
  hi_ptr: bool
}

const CHR_ROM_BEGIN: u16 =        0;
const CHR_ROM_END: u16 =          0x1FFF;
const VRAM_MIRROR_BEGIN: u16 =    0x2000;
const VRAM_MIRROR_END: u16 =      0x2FFF;
const PALETTE_TABLE_BEGIN: u16 =  0x3F00;
const PALETTE_TABLE_END: u16 =    0x3FFF;


const NAME_TABLE_1: u8 =          0b0000_0001;
const NAME_TABLE_2: u8 =          0b0000_0010;
const VRAM_ADD_INCREMENT: u8 =    0b0000_0100;
const SPRITE_PATTERN_ADDR: u8 =   0b0000_1000;
const BG_PATTERN_ADDR: u8 =       0b0001_0000;
const SPRITE_SIZE: u8 =           0b0010_0000;
const MASTER_SLAVE_SELECT: u8 =   0b0100_0000;
const GENERATE_NMI: u8 =          0b1000_0000;

pub struct PPU {
  chr_rom: Vec<u8>,
  palette_table: [u8; 32],
  oam_data: [u8; 256],
  vram: [u8; 2048],
  screen_mirroring: ScreenMirroring,
  addr: AddressRegister,
  control: u8,
  status: u8,
  internal_data_buffer: u8,
  scanline: u8,
  cycles: usize
}

impl AddressRegister {

  pub fn new() -> Self {
    AddressRegister { value: (0, 0), hi_ptr: true }
  }

  fn set(&mut self, data: u16) {
    self.value.0 = (data >> 8) as u8;
    self.value.1 = data as u8;
  }

  fn get(&self) -> u16 {
    (self.value.0 as u16) << 8 | self.value.1 as u16
  }

  pub fn update(&mut self, data: u8) {
    
    if self.hi_ptr {
      self.value.0 = data;
    } else {
      self.value.1 = data;
    }

    self.hi_ptr = !self.hi_ptr;
    self.mirror_down();
    
  }

  pub fn increment(&mut self, value: u8) {

    let lsb = self.value.1;
    self.value.1 = self.value.1.wrapping_add(value);
    
    if lsb > self.value.1 {
      self.value.0 = self.value.0.wrapping_add(1);
    }

    self.mirror_down();

  }

  fn mirror_down(&mut self) {
    if self.get() > 0x3FFF {
      self.set(self.get() & 0b0011_1111_1111_1111);
    }
  }

  pub fn reset_latch(&mut self) {
    self.hi_ptr = true;
  }

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
      control: 0,
      status: 0,
      internal_data_buffer: 0,
      scanline: 0,
      cycles: 21
    }
  }

  pub fn tick(&mut self, cycles: u8) {

    self.cycles += cycles as usize;

    if self.cycles >= 341 {

      self.cycles -= 341;
      self.scanline += 1;

      if self.scanline == 241 {
        if self.control & GENERATE_NMI == GENERATE_NMI {
          self.status;
        }
      }

    }

  }

  pub fn write_to_ppu_address(&mut self, data: u8) {
    self.addr.update(data);
  }

  fn get_vram_addr_increment(&self) -> u8 {
    if self.control & VRAM_ADD_INCREMENT == VRAM_ADD_INCREMENT {
      32
    } else {
      1
    }
  }

  pub fn update_ctrl_register(&mut self, data: u8) {
    self.control = data;
  }

  fn increment_vram_addr(&mut self) {
    self.addr.increment(self.get_vram_addr_increment());
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
      _ => panic!("Unexpected accessto mirrored adddress space")
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