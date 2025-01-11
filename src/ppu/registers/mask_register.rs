use bitflags::bitflags;

bitflags! {
  #[derive(Default)]
  pub struct MaskRegister: u8 {
    const GRAYSCALE =       0b0000_0001;
    const SHOW_LEFT_BG =    0b0000_0010;
    const SHOW_LEFT_SPR =   0b0000_0100;
    const SHOW_BG =         0b0000_1000;
    const SHOW_SPR =        0b0001_0000;
    const EMPH_RED =        0b0010_0000;
    const EMPH_GREEN =      0b0100_0000;
    const EMPH_BLUE =       0b1000_0000;
  }
}

impl MaskRegister {

  pub fn new() -> Self {
    MaskRegister::from_bits_truncate(0)
  }

  // ? There has to be a better way to do this
  // ? Probably somewhere in the bitflags documentation
  pub fn update(&mut self, data: u8) {
    self.remove(MaskRegister::from_bits_truncate(self.bits()));
    self.insert(MaskRegister::from_bits_truncate(data));
  }

  pub fn grayscale(&self) -> bool { self.contains(MaskRegister::GRAYSCALE) }

}