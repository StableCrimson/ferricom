use std::fs;
use std::path::Path;
use log::{debug, warn};

pub mod header;

use crate::mappers::Mapper;
use crate::mappers::nrom::NROM;
use crate::mappers::txrom::TXROM;
use crate::rom::header::HEADER_SIZE;

use self::header::iNESHeader;

const TRAINER_SIZE: usize = 512;
const PRG_ROM_PAGE_SIZE: usize = 16384;
const CHR_ROM_PAGE_SIZE: usize = 8192;
const _PRG_RAM_PAGE_SIZE: usize = 8192;
const _CHR_RAM_PAGE_SIZE: usize = 4096;

/// `iNESVersion::Indeterminate` means that the file is either `iNES` 0.7 or `iNES` Archaic.
/// Right now I do not dileniate between the two because ferricom currently only supports `iNES` 1.
/// While `iNES` 2 is not natively supported, its backwards compatibility with `iNES` 1 allows it to be
/// used. As such, it will load, but you will recieve a warning when loading the ROM that the unique features
/// of `iNES` 2 will not be used until specific support for it is added.
#[allow(non_camel_case_types)]
#[derive(PartialEq, Debug)]
pub enum iNESVersion {
  iNES_Archaic,
  iNES_1,
  iNES_2,
  Indeterminate
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum ScreenMirroring {
  Horizontal,
  Vertical,
  FourScreen,
  Default,
}

pub enum Region {
  NSTC,
  PAL
}

pub struct ROM {
  pub name: String,
  pub header: iNESHeader,
  pub mapper: Mapper,
  pub prg_rom: Vec<u8>,
  pub prg_ram: Vec<u8>,
  pub chr_rom: Vec<u8>,
  pub chr_ram: Vec<u8>,
  pub ex_ram: Vec<u8>,
}

impl ROM {

  pub fn from_path<P: AsRef<Path>>(path: P) -> Result<ROM, String> {

    let path = path.as_ref();
    let name = path.file_stem().unwrap().to_str().unwrap(); // yuck

    let Ok(byte_code) = fs::read(path) else {
      return Err(format!("Unable to read ROM \"{}\"", path.to_string_lossy()));
    };

    ROM::from_bytes(name, &byte_code)

  }

  pub fn from_bytes<S>(name: S, byte_code: &[u8]) -> Result<ROM, String> where S: ToString,  {

    let header = match iNESHeader::from_bytes(byte_code) {
      Ok(header) => header,
      Err(msg) => return Err(msg)
    };

    let prg_rom_size = header.prg_rom_banks as usize * PRG_ROM_PAGE_SIZE;
    let chr_rom_size = header.chr_rom_banks as usize * CHR_ROM_PAGE_SIZE;

    if header.chr_rom_banks == 0 {
      warn!("ROM has no CHR_ROM, uses CHR_RAM instead, which is unsupported");
    }

    debug!("iNES Version: {:?}", header.ines_version);
  
    if header.ines_version != iNESVersion::iNES_1 {
      if header.ines_version != iNESVersion::iNES_2 {
        return Err("ROM must be either iNES_1 or iNES_2!".to_string());
      }
      warn!("WARNING: iNES V2 is now functionally supported supported, but not all features may be present yet.");
    }
  
    debug!("Mapper 0x{:0X}", header.mapper_id);

    if header.has_trainer {
      warn!("ROM contains a 512 trainer, this will not be used and has no planned support.");
    }
    
    debug!("PRG ROM is 0x{:0X} bytes", prg_rom_size);
    debug!("CHR ROM is 0x{:0X} bytes", chr_rom_size);
  
    let prg_rom_offset = HEADER_SIZE + if header.has_trainer { TRAINER_SIZE } else { 0 };
    let chr_rom_offset = prg_rom_offset + prg_rom_size;
  
    debug!("Screen mapping: {:?}", header.mirroring);

    let mut rom = Self {
      name: name.to_string(),
      header,
      mapper: Mapper::none(),
      prg_rom: byte_code[prg_rom_offset..(prg_rom_offset+prg_rom_size)].to_vec(),
      prg_ram: vec![],
      chr_rom: byte_code[chr_rom_offset..(chr_rom_offset+chr_rom_size)].to_vec(),
      chr_ram: vec![],
      ex_ram: vec![],
    };

    let mapper = match rom.header.mapper_id {
      0 => NROM::load(&mut rom),
      4 => TXROM::load(&mut rom),
      _ => return Err(format!("Mapper {} not supported", rom.header.mapper_id))
    };
  
    rom.mapper = mapper;
    Ok(rom)

  }

  pub fn has_chr_rom(&self) -> bool {
    self.chr_rom.len() != 0
  }

}

#[cfg(test)]
pub mod tests {

  use super::*;

  struct TestRom {
    header: Vec<u8>,
    trainer: Option<Vec<u8>>,
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
  }

  fn create_rom(rom: TestRom) -> Vec<u8> {
      let mut result = Vec::with_capacity(
          rom.header.len()
              + rom.trainer.as_ref().map_or(0, |t| t.len())
              + rom.prg_rom.len()
              + rom.chr_rom.len(),
      );

      result.extend(&rom.header);
      if let Some(t) = rom.trainer {
          result.extend(t);
      }
      result.extend(&rom.prg_rom);
      result.extend(&rom.chr_rom);

      result
  }

  pub fn test_rom() -> ROM {
      let test_rom = create_rom(TestRom {
          header: vec![
              0x4E, 0x45, 0x53, 0x1A, 0x02, 0x01, 0x01, 00, 00, 00, 00, 00, 00, 00, 00, 00,
          ],
          trainer: None,
          prg_rom: vec![1; 2 * PRG_ROM_PAGE_SIZE],
          chr_rom: vec![2; 1 * CHR_ROM_PAGE_SIZE],
      });

      ROM::from_bytes("".to_string(), &test_rom).unwrap()
  }

}