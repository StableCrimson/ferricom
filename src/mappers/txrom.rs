use crate::{rom::{ROM, ScreenMirroring}, mem::Membank};

use super::{Map, Mapper, MappedRead, MappedWrite};

const PRG_RAM_SIZE: u16 = 0x2000;
const CHR_RAM_SIZE: u16 = 0x2000;

const CHR_RAM_START: usize = 0x0000;
const CHR_RAM_END: usize = 0x1FFF;
const PRG_RAM_START: usize = 0x6000;
const PRG_RAM_END: usize = 0x7FFF;
const PRG_ROM_START: usize = 0x8000;
const PRG_ROM_END: usize = 0xFFFF;

pub struct TXROM {
  mirroring: ScreenMirroring,
  prg_rom_banks: Membank,
  prg_ram_banks: Membank,
  chr_banks: Membank,
}

impl TXROM {

  pub fn load(rom: &mut ROM) -> Mapper {

    rom.prg_ram = vec![0; PRG_RAM_SIZE as usize];

    if !rom.has_chr_rom() {
      rom.chr_ram = vec![0; CHR_RAM_SIZE as usize];
    }

    if rom.header.mirroring == ScreenMirroring::FourScreen {
      rom.ex_ram = vec![0; 0x1000];
    }

    let mut txrom = Self {
      mirroring: rom.header.mirroring,
      chr_banks: Membank::new(CHR_RAM_START, CHR_RAM_END, rom.chr_ram.len(), 0x400),
      prg_rom_banks: Membank::new(PRG_ROM_START, PRG_ROM_END, rom.prg_rom.len(), 0x2000),
      prg_ram_banks: Membank::new(PRG_RAM_START, PRG_RAM_END, rom.prg_ram.len(), 0x2000),
    };

    let last_bank = txrom.prg_rom_banks.last();
    txrom.prg_rom_banks.set(2, last_bank-1);
    txrom.prg_rom_banks.set(3, last_bank);
    txrom.into()

  }

}

impl Map for TXROM {

  fn map_read(&self, _addr: u16) -> MappedRead { self.map_peak(_addr) }

  fn map_peak(&self, _addr: u16) -> MappedRead { 
    match _addr as usize {
      CHR_RAM_START..=CHR_RAM_END => MappedRead::Chr(self.chr_banks.translate(_addr)),
      PRG_RAM_START..=PRG_RAM_END => MappedRead::PrgRAM(self.prg_ram_banks.translate(_addr)),
      PRG_ROM_START..=PRG_ROM_END => MappedRead::PrgROM(self.prg_rom_banks.translate(_addr)),
      _ => MappedRead::None
    }
  }

  fn map_write(&self, _addr: u16, _data: u8) -> MappedWrite {

    match _addr as usize {

      CHR_RAM_START..=CHR_RAM_END => MappedWrite::Chr(self.chr_banks.translate(_addr), _data),
      _ => MappedWrite::None

    }

  }

}