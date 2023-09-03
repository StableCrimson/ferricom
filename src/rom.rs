use log::{debug, warn, error};

const HEADER_SIZE: usize = 16;
const TRAINER_SIZE: usize = 512;
const PRG_ROM_PAGE_SIZE: usize = 16384;
const CHR_ROM_PAGE_SIZE: usize = 8192;

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

pub struct ROM {
  pub prg_rom: Vec<u8>,
  pub chr_rom: Vec<u8>,
  pub mirroring: ScreenMirroring,
  pub mapper: u8
}

impl ROM {

  #[cfg(not(tarpaulin_include))]
  pub fn new(byte_code: &[u8]) -> ROM {

    let header = ROM::retrieve_and_verify_header(byte_code).unwrap_or_else(|msg| {
      error!("{msg}");
      panic!("ERROR: {msg}");
    });
  
    let prg_rom_size = header[4] as usize * PRG_ROM_PAGE_SIZE;
    let chr_rom_size = header[5] as usize * CHR_ROM_PAGE_SIZE;
    let version = ROM::get_ines_version(header);
    
    debug!("iNES Version: {:?}", version);
  
    if version != iNESVersion::iNES_1 {
      if version != iNESVersion::iNES_2 {
        let msg = "ROM must be either iNES_1 or iNES_2!";
        error!("{msg}");
        panic!("ERROR: {msg}");
      }
      warn!("WARNING: iNES V2 is not officially supported, but will work as V1 because of backwards compatibility");
    }
  
    let mapper = header[7] & 0b1111_0000 | header[6] >> 4;
    debug!("Mapper 0x{:0X}", mapper);

    let trainer_present = ROM::has_trainer(header);

    debug!("Trainer is present: {trainer_present}");
    debug!("PRG ROM is 0x{:0X} bytes", prg_rom_size);
    debug!("CHR ROM is 0x{:0X} bytes", prg_rom_size);
  
    let prg_rom_offset = HEADER_SIZE + if trainer_present { TRAINER_SIZE } else { 0 };
    let chr_rom_offset = prg_rom_offset + prg_rom_size;
  
    let mirroring = ROM::get_screen_mirroring(header);

    debug!("Screen mapping: {:?}", mirroring);
  
    ROM {
      prg_rom: byte_code[prg_rom_offset..(prg_rom_offset+prg_rom_size)].to_vec(),
      chr_rom: byte_code[chr_rom_offset..(chr_rom_offset+chr_rom_size)].to_vec(),
      mirroring,
      mapper
    }
  
  }

  fn retrieve_and_verify_header(byte_code: &[u8]) -> Result<&[u8], &str> {

    let header = match byte_code.get(0..HEADER_SIZE) {
      Some(header) => header,
      None => return Err("Error reading ROM header. ROM may be malformed")
    };

    let nes_signature = vec![0x4E, 0x45, 0x53, 0x1A];

    if header[0..4] != nes_signature {
      return Err("Header does not contain signature bytes. ROM may be malformed or invalid");
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

    if control_byte & 0b1 > 0 {
      ScreenMirroring::Vertical
    } else {
      ScreenMirroring::Horizontal
    }

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

      ROM::new(&test_rom)
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