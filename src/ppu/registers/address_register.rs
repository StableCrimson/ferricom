#[derive(Default)]
pub struct AddressRegister {
  value: (u8, u8),
  hi_ptr: bool
}

impl AddressRegister {

  pub fn new() -> Self {
    AddressRegister { value: (0, 0), hi_ptr: true }
  }

  fn set(&mut self, data: u16) {
    self.value.0 = (data >> 8) as u8;
    self.value.1 = (data & 0xFF) as u8;
  }

  pub fn get(&self) -> u16 {
    (self.value.0 as u16) << 8 | (self.value.1 as u16)
  }

  pub fn update(&mut self, data: u8) {
    
    if self.hi_ptr {
      self.value.0 = data;
    } else {
      self.value.1 = data;
    }

    self.mirror_down();
    self.hi_ptr = !self.hi_ptr;
    
  }

  pub fn increment(&mut self, value: u8) {

    let lsb = self.value.1;
    self.value.1 = self.value.1.wrapping_add(value);
    
    if lsb > self.value.1 {
      self.value.0 = self.value.0.wrapping_add(1);
    }

    self.mirror_down();

  }

  fn mirror_down(&mut self) {
    if self.get() > 0x3FFF {
      self.set(self.get() & 0b0011_1111_1111_1111);
    }
  }

  pub fn reset_latch(&mut self) {
    self.hi_ptr = true;
  }

}

#[cfg(test)]
mod tests {

  use super::*;

  #[test]
  fn test_set() {

    let mut reg = AddressRegister::default();
    let data = 0xFFAA;

    reg.set(data);
    assert_eq!(reg.value.0, 0xFF);
    assert_eq!(reg.value.1, 0xAA);

  }

  #[test]
  fn test_write_hi_ptr_behavior() {

    let mut reg = AddressRegister::default();
    
    reg.update(0xFF);
    assert_eq!(reg.value.1, 0xFF);
    assert!(reg.hi_ptr);

    // Mirrored down
    reg.update(0xAB);
    assert_eq!(reg.value.0, 0x2B);
    assert!(!reg.hi_ptr);

  }

  #[test]
  fn test_reset_latch() {
    let mut reg = AddressRegister::default();
    reg.reset_latch();
    assert!(reg.hi_ptr)
  }

}