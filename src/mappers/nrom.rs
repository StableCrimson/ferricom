use crate::rom::{ScreenMirroring, ROM};

use super::{Mapper, Map};

const PRG_RAM_START: u16 =        0x6000;
const PRG_RAM_END: u16 =          0xCFFF;
const PRG_ROM_BANK_1_START: u16 = 0x8000;
const PRG_ROM_BANK_1_END: u16 =   0xBFFF;
const PRG_ROM_BANK_2_START: u16 = 0xC000;
const PRG_ROM_BANK_2_END: u16 =   0xFFFF;

pub struct NROM {
  mirroring: ScreenMirroring,
  mirror_prg_rom: bool,
}

impl NROM {

  pub fn load(rom: &mut ROM) -> Mapper {
    
    let nrom = Self {
      mirroring: rom.header.mirroring,
      mirror_prg_rom: rom.prg_rom.len() <= 0x4000,
    };

    nrom.into()

  }

}

impl Map for NROM {

  fn mirroring(&self) -> ScreenMirroring {
      self.mirroring
  }

}