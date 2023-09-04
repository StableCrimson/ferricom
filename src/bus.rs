use crate::{cpu::Mem, ppu::PPU};
use crate::rom::ROM;
use crate::gamepad::Gamepad;
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

const GAMEPAD_ADDRESS: u16 =          0x4016;

pub struct Bus<'call> {
  cpu_vram: [u8; 2048],
  prg_rom: Vec<u8>,
  pub ppu: PPU,
  gamepad: Gamepad,
  cycles: usize,
  callback: Box<dyn FnMut(&PPU, &mut Gamepad) + 'call>,
}

impl<'a> Bus<'a> {

  pub fn new<'call, F>(rom: ROM, callback: F) -> Bus<'call>
  where 
      F: FnMut(&PPU, &mut Gamepad) + 'call {
    
    let ppu = PPU::new(rom.chr_rom, rom.mirroring);
    
    Bus {
      cpu_vram: [0; 2048],
      prg_rom: rom.prg_rom,
      ppu: ppu,
      gamepad: Gamepad::new(),
      cycles: 0,
      callback: Box::from(callback)
    }
  }

  pub fn read_prg_rom(&self, mut addr: u16) -> u8 {

    addr -= 0x8000;

    if self.prg_rom.len() == 0x4000 && addr >= 0x4000 {
      addr %= 0x4000;
    }

    self.prg_rom[addr as usize]

  }

  pub fn tick(&mut self) {
    self.tick_cycles(1);
  }

  pub fn tick_cycles(&mut self, cycles: u8) {

    self.cycles += cycles as usize;

    let frame = self.ppu.tick(cycles * 3);
    if frame {
      (self.callback)(&self.ppu, &mut self.gamepad);
    }

  }

  pub fn get_cycles(&self) -> usize {
    self.cycles
  }

  pub fn poll_nmi(&mut self) -> Option<u8> {
    self.ppu.poll_nmi()
  }

}

impl Mem for Bus<'_> {

  fn mem_read_u8(&mut self, addr: u16) -> u8 {
    match addr {
      RAM_START..=RAM_MIRROR_END => {
        let mirred_addr = addr & 0b0000_0111_1111_1111;
        self.cpu_vram[mirred_addr as usize]
      },
      PPU_CONTROL_BYTE | PPU_MASK_REGISTER | PPU_OAM_ADDRESS_REGISTER | PPU_SCROLL_BYTE | PPU_ADDRESS_REGISTER | PPU_DMA_ADDRESS => {
        // error!("Attempted to read from write-only PPU address 0x{:0X}", addr);
        // panic!("Attempted to read from write-only PPU address 0x{:0X}", addr);
        0
      },
      PPU_STATUS_REGISTER => self.ppu.read_status(),
      PPU_OAM_DATA_REGISTER => self.ppu.read_oam_data(),
      PPU_DATA_REGISTER => self.ppu.read_data(),
      0x4000..=0x4015 => 0,
      0x2008..=PPU_REGISTER_MIRROR_END => {
        let mirrored_addr = addr & 0b0010_0000_0000_0111;
        self.mem_read_u8(mirrored_addr)
      },
      ROM_SPACE_START..=ROM_SPACE_END => {
        self.read_prg_rom(addr)
      },
      GAMEPAD_ADDRESS => self.gamepad.read(),
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
      PPU_OAM_ADDRESS_REGISTER => self.ppu.write_oam_addr(data),
      PPU_OAM_DATA_REGISTER => self.ppu.write_oam_data(data),
      PPU_ADDRESS_REGISTER => self.ppu.write_to_ppu_address(data),
      PPU_DATA_REGISTER => self.ppu.write_to_data_register(data),
      0x2008..=PPU_REGISTER_MIRROR_END => {
        let mirrored_addr = addr & 0b0010_0000_0000_0111;
        self.mem_write_u8(mirrored_addr, data);
      },
      ROM_SPACE_START..=ROM_SPACE_END => {
        // let msg = "Attempted to write to ROM address space!";
        // error!("{msg}");
        // panic!("{msg}");
        // self.prg_rom[addr as usize -0x8000] = data;
      },
      GAMEPAD_ADDRESS => self.gamepad.write(data),
      PPU_DMA_ADDRESS => {

        let mut buffer: [u8; 256] = [0; 256];
        let msb: u16 = (data as u16) << 8;

        for i in 0..256u16 {
          buffer[i as usize] = self.mem_read_u8(msb+i);
        }

        self.ppu.write_oam_dma(&buffer);

      }
      _ => {
        debug!("Ignoring memory access at 0x{:0X}", addr);
      }
    }
  }
}