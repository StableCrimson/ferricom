use bitflags::bitflags;

bitflags!{
  pub struct StatusRegister: u8 {
    const NOTUSED          = 0b0000_0001;
    const NOTUSED2         = 0b0000_0010;
    const NOTUSED3         = 0b0000_0100;
    const NOTUSED4         = 0b0000_1000;
    const NOTUSED5         = 0b0001_0000;
    const SPRITE_OVERFLOW  = 0b0010_0000;
    const SPRITE_ZERO_HIT  = 0b0100_0000;
    const VBLANK_STARTED   = 0b1000_0000;
  }
}

impl StatusRegister {

  pub fn new() -> Self {
    StatusRegister::from_bits_truncate(0b0000_0000)
  }

  pub fn set_vblank_status(&mut self, value: bool) {
    self.set(StatusRegister::VBLANK_STARTED, value);
  }

  pub fn reset_vblank_status(&mut self) {
    self.remove(StatusRegister::VBLANK_STARTED);
  }

  pub fn is_in_vblank(&self) -> bool {
    self.contains(StatusRegister::VBLANK_STARTED)
  }

}