use super::{iNESVersion, Region, ScreenMirroring};

pub const HEADER_SIZE: usize = 16;

#[allow(non_camel_case_types)]
pub struct iNESHeader {
  pub ines_version: iNESVersion,
  pub region: Region,
  pub mirroring: ScreenMirroring,
  pub prg_rom_banks: u16,
  pub chr_rom_banks: u16,
  pub mapper_id: u16,
  pub has_trainer: bool,
  pub has_battery_backed_ram: bool,
}

impl iNESHeader {

  pub fn from_bytes(bytecode: &[u8]) -> Result<iNESHeader, String> {

    let header = match iNESHeader::retrieve_and_verify_header(bytecode) {
      Ok(header) => header,
      Err(msg) => return Err(msg)
    };

    let region = match header[9] & 1 {
      0 => Region::NSTC,
      1 => Region::PAL,
      _ => return Err("NES region unrecognizable".to_string()),
    };

    // V1
    let has_battery_backed_ram = header[6] & 2 == 2;
    let ines_version = iNESHeader::get_ines_version(header);
    let mirroring = iNESHeader::get_screen_mirroring(header);
    let mut mapper_id = (header[7] & 0b1111_0000 | header[6] >> 4) as u16;
    let has_trainer = header[6] & 0b100 != 0;
    let mut prg_rom_banks = header[4] as u16;
    let mut chr_rom_banks = header[5] as u16;

    // Have to do some things differently with the iNES_2 header
    if ines_version == iNESVersion::iNES_2 {
      mapper_id |= ((header[8] & 0x0F) as u16) << 8;
      prg_rom_banks |= ((header[9] & 0x0F) as u16) << 8;
      chr_rom_banks |= ((header[9] & 0xF0) as u16) << 8;
    }

    Ok(
      iNESHeader { 
        ines_version,
        region,
        mirroring,
        prg_rom_banks,
        chr_rom_banks,
        mapper_id,
        has_trainer,
        has_battery_backed_ram
      }
    )

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

}

#[cfg(test)]
mod tests {

  use super::*;

  #[test]
  fn test_retrieve_and_verify_header() {

    let header_clear = vec![0x4E, 0x45, 0x53, 0x1A, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    let header_bad_signature = vec![0x89, 0x45, 0x53, 0x1A, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    let header_bad_read: Vec<u8> = vec![];

    assert_eq!(iNESHeader::retrieve_and_verify_header(&header_clear).unwrap(), header_clear);
    assert_eq!(iNESHeader::retrieve_and_verify_header(&header_bad_signature).unwrap_err(), "Header does not contain signature bytes. ROM may be malformed or invalid");
    assert_eq!(iNESHeader::retrieve_and_verify_header(&header_bad_read).unwrap_err(), "Error reading ROM header. ROM may be malformed");

  }

  #[test]
  fn test_get_ines_version() {

    let mut header = vec![0x4E, 0x45, 0x53, 0x1A, 0x01, 0x01, 0x00, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

    assert_eq!(iNESHeader::get_ines_version(&header), iNESVersion::iNES_2);

    header[7] = 0x04;
    assert_eq!(iNESHeader::get_ines_version(&header), iNESVersion::iNES_Archaic);

    header[7] = 0x00;
    assert_eq!(iNESHeader::get_ines_version(&header), iNESVersion::iNES_1);

    header[12] = 0x01;
    assert_eq!(iNESHeader::get_ines_version(&header), iNESVersion::Indeterminate);

    header[7] = 0x0C;
    assert_eq!(iNESHeader::get_ines_version(&header), iNESVersion::Indeterminate);

  }

  #[test]
  fn test_get_screen_mirroring() {

    let mut header = vec![0x4E, 0x45, 0x53, 0x1A, 0x01, 0x01, 0x08, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

    assert_eq!(iNESHeader::get_screen_mirroring(&header), ScreenMirroring::FourScreen);

    header[6] = 0x01;
    assert_eq!(iNESHeader::get_screen_mirroring(&header), ScreenMirroring::Vertical);

    header[6] = 0x00;
    assert_eq!(iNESHeader::get_screen_mirroring(&header), ScreenMirroring::Horizontal);

  }

}