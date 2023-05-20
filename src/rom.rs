use std::process;

const PRG_ROM_PAGE_SIZE: usize = 16384;
const CHR_ROM_PAGE_SIZE: usize = 8192;
const TRAINER_SIZE: usize = 512;
const HEADER_SIZE: usize = 16;

/// iNES Intedeterminate means that the version of iNES the file uses
/// is either iNES 0.7 or iNES Archaic. Right now I do not dileniate between
/// the two because ferricom currently only supports iNES 1
#[allow(non_camel_case_types)]
#[derive(PartialEq, Debug)]
enum iNESVersion {
  iNES_Archaic,
  iNES_1,
  iNES_2,
  Indeterminate
}

#[derive(PartialEq, Debug)]
pub enum ScreenMirroring {
  Horizontal,
  Vertical,
  FourScreen
}

pub struct ROM {
  pub prg_rom: Vec<u8>,
  pub chr_rom: Vec<u8>,
  pub mirroring: ScreenMirroring
}

impl ROM {

  pub fn new(byte_code: &Vec<u8>) -> ROM {

    let header = retrieve_and_verify_header(byte_code).unwrap_or_else(|msg| {
      eprintln!("ERROR: {msg}");
      process::exit(1);
    });
  
    let prg_rom_size = header[4] as usize * PRG_ROM_PAGE_SIZE;
    let chr_rom_size = header[5] as usize * CHR_ROM_PAGE_SIZE;
    let version = get_ines_version(header);
  
    println!("iNES Version: {:?}", version);
  
    if version != iNESVersion::iNES_1 {
      if version != iNESVersion::iNES_2 {
        eprintln!("ERROR: Currently only iNES V1 is supported.");
        process::exit(1);
      }

      println!("WARNING: iNES V2 is not officially supported, but will work as V1 because of backwards compatibility");

    }
  
    let trainer_present = has_trainer(header);
    println!("Trainer is present: {trainer_present}");
  
    println!("PRG ROM is {prg_rom_size} bytes");
    println!("CHR ROM is {chr_rom_size} bytes");
  
    let prg_rom_offset = HEADER_SIZE + if trainer_present { TRAINER_SIZE } else { 0 };
    let chr_rom_offset = prg_rom_offset + prg_rom_size;
  
    let screen_mapping = get_screen_mapping(header);
    println!("Screen mapping: {:?}", screen_mapping);
  
    ROM {
      prg_rom: byte_code[prg_rom_offset..(prg_rom_offset+prg_rom_size)].to_vec(),
      chr_rom: byte_code[chr_rom_offset..(chr_rom_offset+chr_rom_size)].to_vec(),
      mirroring: screen_mapping
    }
  
  }

}

fn retrieve_and_verify_header(byte_code: &Vec<u8>) -> Result<&[u8], &str> {

  let header = match byte_code.get(0..HEADER_SIZE as usize) {
    Some(header) => header,
    None => {
      return Err("Error reading ROM header. ROM may be malformed");
    }
  };

  let nes_signature = vec![0x4E, 0x45, 0x53, 0x1A];

  if header[0..4] != nes_signature {
    return Err("Header does not contain signature bytes. ROM may be malformed or invalid");
  }

  println!("Header validated");
  println!("ROM header is {:0X?}", header);

  Ok(header)

}

fn get_ines_version(header: &[u8]) -> iNESVersion {

    match header[0x07] & 0x0C {

      0x08 => return iNESVersion::iNES_2,
      0x04 => return iNESVersion::iNES_Archaic,
      0x00 => {
        if header[12..=15] == vec![0, 0, 0, 0] {
          return iNESVersion::iNES_1
        }
        return iNESVersion::Indeterminate
      }
      _ => return iNESVersion::Indeterminate

    }
}

fn has_trainer(header: &[u8]) -> bool {
  header[6] & 0b100 != 0
}

fn get_screen_mapping(header: &[u8]) -> ScreenMirroring {

  let control_byte = header[6];

  if control_byte & 0b1000 > 0 {
    return ScreenMirroring::FourScreen;
  }

  return if control_byte & 0b1 > 0 { ScreenMirroring::Vertical } else { ScreenMirroring::Horizontal }

}