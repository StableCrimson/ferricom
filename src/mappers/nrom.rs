use crate::rom::{ScreenMirroring, ROM};

use super::{Mapper, Map, MappedRead, MappedWrite};

const CHR_ROM_BANK_START: u16 =   0x0000;
const CHR_ROM_BANK_END: u16 =     0x1FFF;
const PRG_RAM_START: u16 =        0x6000;
const PRG_RAM_END: u16 =          0x7FFF;
const PRG_ROM_BANK_1_START: u16 = 0x8000;
const PRG_ROM_BANK_1_END: u16 =   0xBFFF;
const PRG_ROM_BANK_2_START: u16 = 0xC000;
const PRG_ROM_BANK_2_END: u16 =   0xFFFF;

#[derive(Debug)]
pub struct NROM {
  mirroring: ScreenMirroring,
  mirror_prg_rom: bool,
}

impl NROM {

  pub fn load(rom: &mut ROM) -> Mapper {
    
    if !rom.has_chr_rom() {
      rom.chr_ram = vec![0; 0x2000]
    }

    let nrom = Self {
      mirroring: rom.header.mirroring,
      mirror_prg_rom: rom.prg_rom.len() <= 0x4000,
    };

    nrom.into()

  }

}

impl Map for NROM {

  fn mirroring(&self) -> ScreenMirroring { self.mirroring }

  fn set_mirroring(&mut self, _mirroring: ScreenMirroring) {
      self.mirroring = _mirroring;
  }

  fn map_read(&self, addr: u16) -> MappedRead {
    match addr {
      CHR_ROM_BANK_START..=CHR_ROM_BANK_END => MappedRead::Chr(addr.into()),
      PRG_RAM_START..=PRG_RAM_END => MappedRead::PrgRAM((addr & 0x1FFF).into()),
      PRG_ROM_BANK_1_START..=PRG_ROM_BANK_1_END => MappedRead::PrgROM((addr & 0x3FFF).into()),
      PRG_ROM_BANK_2_START..=PRG_ROM_BANK_2_END => {
        let mirror_mask = if self.mirror_prg_rom { 0x3FFF } else { 0x7FFF };
        MappedRead::PrgROM((addr & mirror_mask).into()) 
      }
      _ => MappedRead::None
    }
  }

  fn map_write(&self, addr: u16, data: u8) -> MappedWrite {
    match addr {
      CHR_ROM_BANK_START..=CHR_ROM_BANK_END => MappedWrite::Chr(addr.into(), data),
      PRG_RAM_START..=PRG_RAM_END => MappedWrite::PrgRAM((addr & 0x1FFF).into(), data),
      _ => MappedWrite::None,
    }
  }
}