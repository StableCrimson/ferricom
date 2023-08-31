use crate::rom::ScreenMirroring;

pub struct PPU {
  chr_rom: Vec<u8>,
  palette_table: [u8; 32],
  oam_data: [u8; 256],
  vram: [u8; 2048],
  screen_mirroring: ScreenMirroring
}

impl PPU {
  pub fn new(chr_rom: Vec<u8>, screen_mirroring: ScreenMirroring) -> PPU {
    PPU {
      chr_rom,
      palette_table: [0; 32],
      oam_data: [0; 256],
      vram: [0; 2048],
      screen_mirroring
    }
  }
}