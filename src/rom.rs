use std::fs;
use std::path::Path;
use log::{debug, warn};

const HEADER_SIZE: usize = 16;
const TRAINER_SIZE: usize = 512;
const PRG_ROM_PAGE_SIZE: usize = 16384;
const CHR_ROM_PAGE_SIZE: usize = 8192;
const PRG_RAM_PAGE_SIZE: usize = 8192;
const CHR_RAM_PAGE_SIZE: usize = 4096;

/// `iNESVersion::Indeterminate` means that the file is either `iNES` 0.7 or `iNES` Archaic.
/// Right now I do not dileniate between the two because ferricom currently only supports `iNES` 1.
/// While `iNES` 2 is not natively supported, its backwards compatibility with `iNES` 1 allows it to be
/// used. As such, it will load, but you will recieve a warning when loading the ROM that the unique features
/// of `iNES` 2 will not be used until specific support for it is added.
#[allow(non_camel_case_types)]
#[derive(PartialEq, Debug)]
enum iNESVersion {
  iNES_Archaic,
  iNES_1,
  iNES_2,
  Indeterminate
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum ScreenMirroring {
  Horizontal,
  Vertical,
  FourScreen
}

pub enum Region {
  NSTC,
  PAL
}

pub struct ROM {
  pub name: String,
  pub region: Region,
  pub prg_rom: Vec<u8>,
  pub prg_ram: Vec<u8>,
  pub chr_rom: Vec<u8>,
  pub chr_ram: Vec<u8>,
  pub mirroring: ScreenMirroring,
  pub mapper: u8
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

    let header = match ROM::retrieve_and_verify_header(byte_code) {
      Ok(header) => header,
      Err(msg) => return Err(msg)
    };

    let mut has_chr_ram = false;
    let mut chr_ram_size = 0;

    let prg_rom_size = header[4] as usize * PRG_ROM_PAGE_SIZE;
    let chr_rom_size = header[5] as usize * CHR_ROM_PAGE_SIZE;
    let prg_ram_size = header[8] as usize * PRG_RAM_PAGE_SIZE;
    let version = ROM::get_ines_version(header);
    let region = match header[9] & 1 {
      0 => Region::NSTC,
      1 => Region::PAL,
      _ => return Err("NES region unrecognizable".to_string()),
    };

    if header[6] & 2 == 2 {
      return Err("Cartridge contains battery-backed RAM, which is unsupported".to_string());
    }

    if chr_rom_size == 0 {
      warn!("ROM has no CHR_ROM, uses CHR_RAM instead, which is unsupported");
      chr_ram_size = CHR_RAM_PAGE_SIZE; // Unsure if we need any more than one page of mem
    }

    debug!("iNES Version: {:?}", version);
  
    if version != iNESVersion::iNES_1 {
      if version != iNESVersion::iNES_2 {
        return Err("ROM must be either iNES_1 or iNES_2!".to_string());
      }
      warn!("WARNING: iNES V2 is not officially supported, but will work because of backwards compatibility with iNES V1");
    }
  
    let mapper = header[7] & 0b1111_0000 | header[6] >> 4;
    debug!("Mapper 0x{:0X}", mapper);

    let trainer_present = ROM::has_trainer(header);

    if trainer_present {
      warn!("ROM contains a 512 trainer, this will not be used and has no planned support.");
    }
    
    debug!("PRG ROM is 0x{:0X} bytes", prg_rom_size);
    debug!("CHR ROM is 0x{:0X} bytes", chr_rom_size);
  
    let prg_rom_offset = HEADER_SIZE + if trainer_present { TRAINER_SIZE } else { 0 };
    let chr_rom_offset = prg_rom_offset + prg_rom_size;
  
    let mirroring = ROM::get_screen_mirroring(header);

    debug!("Screen mapping: {:?}", mirroring);

    Ok(ROM {
      name: name.to_string(),
      region,
      prg_rom: byte_code[prg_rom_offset..(prg_rom_offset+prg_rom_size)].to_vec(),
      prg_ram: vec![0x00; prg_ram_size],
      chr_rom: byte_code[chr_rom_offset..(chr_rom_offset+chr_rom_size)].to_vec(),
      chr_ram: vec![0x00; chr_ram_size],
      mirroring,
      mapper
    })
  
  }

  fn retrieve_and_verify_header(byte_code: &[u8]) -> Result<&[u8], String> {

    let header = match byte_code.get(0..HEADER_SIZE) {
      Some(header) => header,
      None => return Err("Error reading ROM header. ROM may be malformed".to_string())
    };

    let nes_signature = vec![0x4E, 0x45, 0x53, 0x1A];

    if header[0..4] != nes_signature {
      return Err("Header does not contain signature bytes. ROM may be malformed or invalid".to_string());
    }

    Ok(header)

  }

  fn get_ines_version(header: &[u8]) -> iNESVersion {

      match header[0x07] & 0x0C {

        0x08 => iNESVersion::iNES_2,
        0x04 => iNESVersion::iNES_Archaic,
        0x00 => if header[12..=15] == vec![0, 0, 0, 0] { 
          iNESVersion::iNES_1 
        } else { 
          iNESVersion::Indeterminate 
        },
        _ => iNESVersion::Indeterminate

      }
  }

  fn has_trainer(header: &[u8]) -> bool {
    header[6] & 0b100 != 0
  }

  fn get_screen_mirroring(header: &[u8]) -> ScreenMirroring {

    let control_byte = header[6];

    if control_byte & 0b1000 > 0 {
      return ScreenMirroring::FourScreen;
    }

    if control_byte & 1 > 0 {
      ScreenMirroring::Vertical
    } else {
      ScreenMirroring::Horizontal
    }

  }

  fn has_chr_ram(&self) -> bool {
    self.chr_rom.len() == 0
  }

}

#[cfg(test)]
pub mod tests {

  use super::*;

  struct TestRom {
    header: Vec<u8>,
    trainer: Option<Vec<u8>>,
    pgp_rom: Vec<u8>,
    chr_rom: Vec<u8>,
  }

  fn create_rom(rom: TestRom) -> Vec<u8> {
      let mut result = Vec::with_capacity(
          rom.header.len()
              + rom.trainer.as_ref().map_or(0, |t| t.len())
              + rom.pgp_rom.len()
              + rom.chr_rom.len(),
      );

      result.extend(&rom.header);
      if let Some(t) = rom.trainer {
          result.extend(t);
      }
      result.extend(&rom.pgp_rom);
      result.extend(&rom.chr_rom);

      result
  }

  pub fn test_rom() -> ROM {
      let test_rom = create_rom(TestRom {
          header: vec![
              0x4E, 0x45, 0x53, 0x1A, 0x02, 0x01, 0x31, 00, 00, 00, 00, 00, 00, 00, 00, 00,
          ],
          trainer: None,
          pgp_rom: vec![1; 2 * PRG_ROM_PAGE_SIZE],
          chr_rom: vec![2; 1 * CHR_ROM_PAGE_SIZE],
      });

      ROM::from_bytes("".to_string(), &test_rom).unwrap()
  }

  #[test]
  fn test_retrieve_and_verify_header() {

    let header_clear = vec![0x4E, 0x45, 0x53, 0x1A, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    let header_bad_signature = vec![0x89, 0x45, 0x53, 0x1A, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    let header_bad_read: Vec<u8> = vec![];

    assert_eq!(ROM::retrieve_and_verify_header(&header_clear).unwrap(), header_clear);
    assert_eq!(ROM::retrieve_and_verify_header(&header_bad_signature).unwrap_err(), "Header does not contain signature bytes. ROM may be malformed or invalid");
    assert_eq!(ROM::retrieve_and_verify_header(&header_bad_read).unwrap_err(), "Error reading ROM header. ROM may be malformed");

  }

  #[test]
  fn test_get_ines_version() {

    let mut header = vec![0x4E, 0x45, 0x53, 0x1A, 0x01, 0x01, 0x00, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

    assert_eq!(ROM::get_ines_version(&header), iNESVersion::iNES_2);

    header[7] = 0x04;
    assert_eq!(ROM::get_ines_version(&header), iNESVersion::iNES_Archaic);

    header[7] = 0x00;
    assert_eq!(ROM::get_ines_version(&header), iNESVersion::iNES_1);

    header[12] = 0x01;
    assert_eq!(ROM::get_ines_version(&header), iNESVersion::Indeterminate);

    header[7] = 0x0C;
    assert_eq!(ROM::get_ines_version(&header), iNESVersion::Indeterminate);

  }

  #[test]
  fn test_has_trainer() {

    let mut header = vec![0x4E, 0x45, 0x53, 0x1A, 0x01, 0x01, 0x00, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
  
    assert!(!ROM::has_trainer(&header));

    header[6] = 0x04;
    assert!(ROM::has_trainer(&header));

  }

  #[test]
  fn test_get_screen_mirroring() {

    let mut header = vec![0x4E, 0x45, 0x53, 0x1A, 0x01, 0x01, 0x08, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

    assert_eq!(ROM::get_screen_mirroring(&header), ScreenMirroring::FourScreen);

    header[6] = 0x01;
    assert_eq!(ROM::get_screen_mirroring(&header), ScreenMirroring::Vertical);

    header[6] = 0x00;
    assert_eq!(ROM::get_screen_mirroring(&header), ScreenMirroring::Horizontal);

  }

}