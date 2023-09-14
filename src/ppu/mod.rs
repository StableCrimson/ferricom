pub mod registers;
pub mod palette;
pub mod frame;
pub mod render;

use log::warn;

use crate::mappers::{Mapper, Map, Empty, MappedWrite};
use crate::rom::ScreenMirroring;
use crate::ppu::registers::address_register::AddressRegister;
use crate::ppu::registers::control_register::ControlRegister;
use crate::ppu::registers::status_register::StatusRegister;

use self::registers::mask_register::MaskRegister;

const CHR_ROM_BEGIN: u16 =        0;
const CHR_ROM_END: u16 =          0x1FFF;
const VRAM_NAMETABLES_BEGIN: u16 =    0x2000;
const VRAM_NAMETABLES_END: u16 =      0x2FFF;
const VRAM_MIRROR_BEGIN: u16 =        0x3000;
const VRAM_MIRROR_END: u16 =          0x3EFF;
const PALETTE_TABLE_BEGIN: u16 =  0x3F00;
const PALETTE_TABLE_END: u16 =    0x3FFF;

pub struct PPU {
  pub chr_rom: Vec<u8>,
  pub chr_ram: Vec<u8>,
  pub ex_ram: Vec<u8>,
  pub mapper: Mapper,
  palette_table: [u8; 32],
  oam_addr: u8,
  oam_data: [u8; 256],
  pub vram: [u8; 2048],
  addr: AddressRegister,
  control: ControlRegister,
  status: StatusRegister,
  mask: MaskRegister,
  pub internal_data_buffer: u8,
  pub scanline: u16,
  pub cycles: usize,
  should_reset: bool,
  nmi: Option<u8>,
}

impl PPU {

  pub fn new() -> PPU {
    PPU {
      chr_rom: vec![],
      chr_ram: vec![],
      ex_ram: vec![],
      mapper: Mapper::Empty(Empty),
      palette_table: [0; 32],
      oam_addr: 0,
      oam_data: [0; 256],
      vram: [0; 2048],
      addr: AddressRegister::new(),
      control: ControlRegister::new(),
      status: StatusRegister::new(),
      mask: MaskRegister::new(),
      internal_data_buffer: 0,
      scanline: 0,
      cycles: 0,
      should_reset: false,
      nmi: None,
    }
  }

  pub fn load_chr_rom(&mut self, chr_rom: Vec<u8>) { self.chr_rom = chr_rom; }
  
  pub fn load_chr_ram(&mut self, chr_ram: Vec<u8>) { self.chr_ram = chr_ram; }
  
  pub fn load_ex_ram(&mut self, ex_ram: Vec<u8>) { self.ex_ram = ex_ram; }
  
  pub fn load_mapper(&mut self, mapper: Mapper) { self.mapper = mapper; }

  pub fn should_reset(&self) -> bool { self.should_reset }

  pub fn set_should_reset(&mut self, val: bool) { self.should_reset = val; }

  pub fn tick(&mut self, cycles: u8) -> bool {

    self.cycles += cycles as usize;

    if self.cycles >= 341 {

      self.cycles -= 341;
      self.scanline += 1;

      if self.scanline == 241 {
        self.status.set_vblank_status(true);
        if self.control.should_generate_vblank_nmi() {
          self.nmi = Some(1);
        }
      }

      if self.scanline >= 262 {
        self.scanline = 0;
        self.status.reset_vblank_status();
        self.internal_data_buffer = 0;
        self.nmi = None;
        return true;
      }
    }
    false
  }

  pub fn write_to_ppu_address(&mut self, data: u8) {
    self.internal_data_buffer = data;
    self.addr.update(data);
  }

  pub fn read_status(&mut self) -> u8 {
    let data = self.status.bits();
    self.internal_data_buffer |= self.status.bits() & 0xE0;
    self.status.reset_vblank_status();
    self.addr.reset_latch();
    data
  }

  pub fn update_ctrl_register(&mut self, data: u8) {

    self.internal_data_buffer = data;
    let before_nmi = self.control.should_generate_vblank_nmi();
    self.control.update(data);

    if !before_nmi && self.control.should_generate_vblank_nmi() && self.status.is_in_vblank() {
      self.nmi = Some(1);
    }

  }

  fn increment_vram_addr(&mut self) {
    self.addr.increment(self.control.get_vram_addr_increment());
  }

  pub fn poll_nmi(&mut self) -> Option<u8> {
    self.nmi.take()
  }

  pub fn read_data(&mut self) -> u8 {

    let mut addr = self.addr.get();
    self.increment_vram_addr();

    // Mirror down to 0x2000->0x2EFF
    if (VRAM_MIRROR_BEGIN..=VRAM_MIRROR_END).contains(&addr) {
      addr -= 0x1000;
    }

    match addr {
      0..=0x1FFF => {

        let result = self.internal_data_buffer;

        self.internal_data_buffer = if !self.chr_ram.is_empty() {
          self.chr_ram[addr as usize]
        } else {
          self.chr_rom[addr as usize]
        };

        result
      },
      0x2000..=0x2FFF => {
        let result = self.internal_data_buffer;
        self.internal_data_buffer = self.vram[self.mirror_vram_addr(addr) as usize];
        result
      },
      0x3F00..=0x3FFF => self.palette_table[(addr-0x3F00) as usize],
      _ => panic!("Unexpected access to mirrored adddress space")
    }

  }

  pub fn read_oam_data(&mut self) -> u8 {
    self.internal_data_buffer = self.oam_data[self.oam_addr as usize];
    self.internal_data_buffer
  }

  pub fn write_oam_data(&mut self, data: u8) {
    self.internal_data_buffer = data;
    self.oam_data[self.oam_addr as usize] = data;
    self.oam_addr = self.oam_addr.wrapping_add(1);
  }

  pub fn write_oam_addr(&mut self, addr: u8) {
    self.internal_data_buffer = addr;
    self.oam_addr = addr;
  }

  pub fn write_oam_dma(&mut self, data: &[u8; 256]) {
    for x in data.iter() {
      self.write_oam_data(*x);
    }
  }

  pub fn write_to_data_register(&mut self, data: u8) {
    
    let mut target_addr = self.addr.get();
    self.internal_data_buffer = data;

    // Mirror down to 0x2000->0x2EFF
    if (VRAM_MIRROR_BEGIN..=VRAM_MIRROR_END).contains(&target_addr) {
      target_addr -= 0x1000;
    }

    match target_addr {
      CHR_ROM_BEGIN..=CHR_ROM_END => {

        if !self.chr_ram.is_empty() {
          if let MappedWrite::Chr(target_addr, data) = self.mapper.map_write(target_addr, data) {
            self.chr_ram[target_addr as usize] = data;
          }
        }
        warn!("Attempted to write to character rom address space: 0x{:0X}", target_addr);

      },
      VRAM_NAMETABLES_BEGIN..=VRAM_NAMETABLES_END => {
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
        // error!("Unable to access mirrored address space: 0x{:0X}", target_addr);
        // panic!("Unable to access mirrored address space: 0x{:0X}", target_addr);
      }
    }

    self.increment_vram_addr();

  }

  pub fn write_to_mask_register(&mut self, data: u8) {
    self.internal_data_buffer = data;
    self.mask.update(data);
  }

  pub fn mirror_vram_addr(&self, addr: u16) -> u16 {

    let mirrored_addr = addr & 0x2FFF;
    let vram_index = mirrored_addr - 0x2000;
    let name_table = vram_index / 0x0400;

    match (self.mapper.mirroring(), name_table) {
      (ScreenMirroring::Vertical, 2) => vram_index - 0x0800,
      (ScreenMirroring::Vertical, 3) => vram_index - 0x0800,
      (ScreenMirroring::Horizontal, 1) => vram_index - 0x0400,
      (ScreenMirroring::Horizontal, 2) => vram_index - 0x0400,
      (ScreenMirroring::Horizontal, 3) => vram_index - 0x0800,
      _ => vram_index
    }
  }
}

// #[cfg(test)]
// pub mod test {
//     use crate::mappers::Empty;

//     use super::*;

//     pub fn new_empty_rom() -> PPU {
//       PPU::new(vec![0; 2048], Mapper::Empty(Empty))
//   }

//     #[test]
//     fn test_ppu_vram_writes() {
//         let mut ppu = new_empty_rom();
//         ppu.write_to_ppu_address(0x23);
//         ppu.write_to_ppu_address(0x05);
//         ppu.write_to_data_register(0x66);

//         assert_eq!(ppu.vram[0x0305], 0x66);
//     }

//     #[test]
//     fn test_ppu_vram_reads() {
//         let mut ppu = new_empty_rom();
//         ppu.update_ctrl_register(0);
//         ppu.vram[0x0305] = 0x66;

//         ppu.write_to_ppu_address(0x23);
//         ppu.write_to_ppu_address(0x05);

//         ppu.read_data(); //load_into_buffer
//         assert_eq!(ppu.addr.get(), 0x2306);
//         assert_eq!(ppu.read_data(), 0x66);
//     }

//     #[test]
//     fn test_ppu_vram_reads_cross_page() {
//         let mut ppu = new_empty_rom();
//         ppu.update_ctrl_register(0);
//         ppu.vram[0x01ff] = 0x66;
//         ppu.vram[0x0200] = 0x77;

//         ppu.write_to_ppu_address(0x21);
//         ppu.write_to_ppu_address(0xff);

//         ppu.read_data(); //load_into_buffer
//         assert_eq!(ppu.read_data(), 0x66);
//         assert_eq!(ppu.read_data(), 0x77);
//     }

//     #[test]
//     fn test_ppu_vram_reads_step_32() {
//         let mut ppu = new_empty_rom();
//         ppu.update_ctrl_register(0b100);
//         ppu.vram[0x01ff] = 0x66;
//         ppu.vram[0x01ff + 32] = 0x77;
//         ppu.vram[0x01ff + 64] = 0x88;

//         ppu.write_to_ppu_address(0x21);
//         ppu.write_to_ppu_address(0xff);

//         ppu.read_data(); //load_into_buffer
//         assert_eq!(ppu.read_data(), 0x66);
//         assert_eq!(ppu.read_data(), 0x77);
//         assert_eq!(ppu.read_data(), 0x88);
//     }

//     // Horizontal: https://wiki.nesdev.com/w/index.php/Mirroring
//     //   [0x2000 A ] [0x2400 a ]
//     //   [0x2800 B ] [0x2C00 b ]
//     #[test]
//     fn test_vram_horizontal_mirror() {
//         let mut ppu = new_empty_rom();
//         ppu.write_to_ppu_address(0x24);
//         ppu.write_to_ppu_address(0x05);

//         ppu.write_to_data_register(0x66); //write to a

//         ppu.write_to_ppu_address(0x28);
//         ppu.write_to_ppu_address(0x05);

//         ppu.write_to_data_register(0x77); //write to B

//         ppu.write_to_ppu_address(0x20);
//         ppu.write_to_ppu_address(0x05);

//         ppu.read_data(); //load into buffer
//         assert_eq!(ppu.read_data(), 0x66); //read from A

//         ppu.write_to_ppu_address(0x2C);
//         ppu.write_to_ppu_address(0x05);

//         ppu.read_data(); //load into buffer
//         assert_eq!(ppu.read_data(), 0x77); //read from b
//     }

//     // Vertical: https://wiki.nesdev.com/w/index.php/Mirroring
//     //   [0x2000 A ] [0x2400 B ]
//     //   [0x2800 a ] [0x2C00 b ]
//     #[test]
//     fn test_vram_vertical_mirror() {
//         let mut ppu = PPU::new(vec![0; 2048], Mapper::Empty(Empty));

//         ppu.write_to_ppu_address(0x20);
//         ppu.write_to_ppu_address(0x05);

//         ppu.write_to_data_register(0x66); //write to A

//         ppu.write_to_ppu_address(0x2C);
//         ppu.write_to_ppu_address(0x05);

//         ppu.write_to_data_register(0x77); //write to b

//         ppu.write_to_ppu_address(0x28);
//         ppu.write_to_ppu_address(0x05);

//         ppu.read_data(); //load into buffer
//         assert_eq!(ppu.read_data(), 0x66); //read from a

//         ppu.write_to_ppu_address(0x24);
//         ppu.write_to_ppu_address(0x05);

//         ppu.read_data(); //load into buffer
//         assert_eq!(ppu.read_data(), 0x77); //read from B
//     }

//     #[test]
//     fn test_read_status_resets_latch() {
//         let mut ppu = new_empty_rom();
//         ppu.vram[0x0305] = 0x66;

//         ppu.write_to_ppu_address(0x21);
//         ppu.write_to_ppu_address(0x23);
//         ppu.write_to_ppu_address(0x05);

//         ppu.read_data(); //load_into_buffer
//         assert_ne!(ppu.read_data(), 0x66);

//         ppu.read_status();

//         ppu.write_to_ppu_address(0x23);
//         ppu.write_to_ppu_address(0x05);

//         ppu.read_data(); //load_into_buffer
//         assert_eq!(ppu.read_data(), 0x66);
//     }

//     #[test]
//     fn test_ppu_vram_mirroring() {
//         let mut ppu = new_empty_rom();
//         ppu.update_ctrl_register(0);
//         ppu.vram[0x0305] = 0x66;

//         ppu.write_to_ppu_address(0x63); //0x6305 -> 0x2305
//         ppu.write_to_ppu_address(0x05);

//         ppu.read_data(); //load into_buffer
//         assert_eq!(ppu.read_data(), 0x66);
//         // assert_eq!(ppu.addr.read(), 0x0306)
//     }

//     #[test]
//     fn test_read_status_resets_vblank() {
//         let mut ppu = new_empty_rom();
//         ppu.status.set_vblank_status(true);

//         let status = ppu.read_status();

//         assert_eq!(status >> 7, 1);
//         assert_eq!(ppu.status.bits() >> 7, 0);
//     }

//     #[test]
//     fn test_oam_read_write() {
//         let mut ppu = new_empty_rom();
//         ppu.write_oam_addr(0x10);
//         ppu.write_oam_data(0x66);
//         ppu.write_oam_data(0x77);

//         ppu.write_oam_addr(0x10);
//         assert_eq!(ppu.read_oam_data(), 0x66);

//         ppu.write_oam_addr(0x11);
//         assert_eq!(ppu.read_oam_data(), 0x77);
//     }

//     #[test]
//     fn test_oam_dma() {
//         let mut ppu = new_empty_rom();

//         let mut data = [0x66; 256];
//         data[0] = 0x77;
//         data[255] = 0x88;

//         ppu.write_oam_addr(0x10);
//         ppu.write_oam_dma(&data);

//         ppu.write_oam_addr(0xf); //wrap around
//         assert_eq!(ppu.read_oam_data(), 0x88);

//         ppu.write_oam_addr(0x10);
//         ppu.write_oam_addr(0x77);
//         ppu.write_oam_addr(0x11);
//         ppu.write_oam_addr(0x66);
//     }
// }
