use crate::palette;
use crate::Frame;

use super::PPU;

fn bg_pallette(ppu: &PPU, tile_column: usize, tile_row : usize) -> [u8;4] {
  let attr_table_idx = tile_row / 4 * 8 +  tile_column / 4;
  let attr_byte = ppu.vram[0x3c0 + attr_table_idx];  // note: still using hardcoded first nametable

  let pallet_idx = match (tile_column %4 / 2, tile_row % 4 / 2) {
      (0,0) => attr_byte & 0b11,
      (1,0) => (attr_byte >> 2) & 0b11,
      (0,1) => (attr_byte >> 4) & 0b11,
      (1,1) => (attr_byte >> 6) & 0b11,
      (_,_) => panic!("should not happen"),
  };

  let pallete_start: usize = 1 + (pallet_idx as usize)*4;
  [ppu.palette_table[0], ppu.palette_table[pallete_start], ppu.palette_table[pallete_start+1], ppu.palette_table[pallete_start+2]]
}

fn sprite_palette(ppu: &PPU, pallete_idx: u8) -> [u8; 4] {
  let start = 0x11 + (pallete_idx * 4) as usize;
  [
      0,
      ppu.palette_table[start],
      ppu.palette_table[start + 1],
      ppu.palette_table[start + 2],
  ]
}

pub fn render(ppu: &PPU, frame: &mut Frame) {

  let bank = ppu.control.background_pattern_address();

  for i in 0..0x3C0 {

    let tile = ppu.vram[i] as u16;
    let tile_x = i % 32;
    let tile_y = i / 32;
    let tile = &ppu.chr_rom[(bank+tile*16) as usize..=(bank+tile*16+15) as usize];
    let palette = bg_pallette(ppu, tile_x, tile_y);
    // println!("{:0X}: {:0X}", i, ppu.vram[i]);

    for y in 0..=7 {

      let mut upper = tile[y];
      let mut lower = tile[y+8];
  
      for x in (0..=7).rev() {
  
        let value = (1&lower) << 1 | (1&upper);
  
        upper >>= 1;
        lower >>= 1;
  
        let rgb = match value {
          0 => palette::SYSTEM_PALLETE[palette[0] as usize],
          1 => palette::SYSTEM_PALLETE[palette[1] as usize],
          2 => palette::SYSTEM_PALLETE[palette[2] as usize],
          3 => palette::SYSTEM_PALLETE[palette[3] as usize],
          _ => panic!(""),
        };
  
        frame.set_pixel(tile_x * 8 + x, tile_y * 8 + y, rgb)
      }
    }
  }

  // Sprites
  for i in (0..ppu.oam_data.len()).step_by(4).rev() {
    let tile_idx = ppu.oam_data[i + 1] as u16;
    let tile_x = ppu.oam_data[i + 3] as usize;
    let tile_y = ppu.oam_data[i] as usize;

    let flip_vertical = if ppu.oam_data[i + 2] >> 7 & 1 == 1 {
        true
    } else {
        false
    };
    let flip_horizontal = if ppu.oam_data[i + 2] >> 6 & 1 == 1 {
        true
    } else {
        false
    };
    let pallette_idx = ppu.oam_data[i + 2] & 0b11;
    let sprite_palette = sprite_palette(ppu, pallette_idx);
   
    let bank: u16 = ppu.control.sprite_pattern_address();

    let tile = &ppu.chr_rom[(bank + tile_idx * 16) as usize..=(bank + tile_idx * 16 + 15) as usize];


    for y in 0..=7 {
      let mut upper = tile[y];
      let mut lower = tile[y + 8];
      'ololo: for x in (0..=7).rev() {
        let value = (1 & lower) << 1 | (1 & upper);
        upper = upper >> 1;
        lower = lower >> 1;
        let rgb = match value {
          0 => continue 'ololo, // skip coloring the pixel
          1 => palette::SYSTEM_PALLETE[sprite_palette[1] as usize],
          2 => palette::SYSTEM_PALLETE[sprite_palette[2] as usize],
          3 => palette::SYSTEM_PALLETE[sprite_palette[3] as usize],
          _ => panic!("can't be"),
        };
        match (flip_horizontal, flip_vertical) {
          (false, false) => frame.set_pixel(tile_x + x, tile_y + y, rgb),
          (true, false) => frame.set_pixel(tile_x + 7 - x, tile_y + y, rgb),
          (false, true) => frame.set_pixel(tile_x + x, tile_y + 7 - y, rgb),
          (true, true) => frame.set_pixel(tile_x + 7 - x, tile_y + 7 - y, rgb),
        }
      }
    }
  }
}

pub fn show_tile(chr_rom: &Vec<u8>, bank: usize, tile_num: usize) -> Frame {

  let mut frame = Frame::new();
  let bank = (bank * 0x1000) as usize;

  let tile = &chr_rom[(bank+tile_num*16)..=(bank+tile_num*16+15)];

  for y in 0..=7 {

    let mut upper = tile[y];
    let mut lower = tile[y+8];

    for x in (0..=7).rev() {

      let value = (1&upper) << 1 | (1&lower);

      upper >>= 1;
      lower >>= 1;

      let rgb = match value {
        0 => palette::SYSTEM_PALLETE[0x01],
        1 => palette::SYSTEM_PALLETE[0x23],
        2 => palette::SYSTEM_PALLETE[0x27],
        3 => palette::SYSTEM_PALLETE[0x30],
        _ => panic!(""),
      };

      frame.set_pixel(x, y, rgb);

    }
  }
  frame
}

pub fn show_tile_bank(chr_rom: &Vec<u8>, bank: usize) -> Frame {
  let mut frame = Frame::new();
    let mut tile_y = 0;
    let mut tile_x = 0;
    let bank = (bank * 0x1000) as usize;

    for tile_n in 0..255 {
        if tile_n != 0 && tile_n % 20 == 0 {
            tile_y += 10;
            tile_x = 0;
        }
        let tile = &chr_rom[(bank + tile_n * 16)..=(bank + tile_n * 16 + 15)];

        for y in 0..=7 {
            let mut upper = tile[y];
            let mut lower = tile[y + 8];

            for x in (0..=7).rev() {
                let value = (1 & upper) << 1 | (1 & lower);
                upper = upper >> 1;
                lower = lower >> 1;
                let rgb = match value {
                    0 => palette::SYSTEM_PALLETE[0x01],
                    1 => palette::SYSTEM_PALLETE[0x23],
                    2 => palette::SYSTEM_PALLETE[0x27],
                    3 => palette::SYSTEM_PALLETE[0x30],
                    _ => panic!("can't be"),
                };
                frame.set_pixel(tile_x + x, tile_y + y, rgb)
            }
        }
        tile_x += 10;
    }
    frame
}