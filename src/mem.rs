use std::cmp::max;

pub trait Mem {

  fn mem_read_u8(&mut self, addr: u16) -> u8;

  fn mem_read_u16(&mut self, addr: u16) -> u16 {
      let lsb: u16 = self.mem_read_u8(addr) as u16;
      let msb = self.mem_read_u8(addr + 1) as u16;
      (msb << 8) | lsb
  }

  fn mem_write_u8(&mut self, addr: u16, data: u8);

  fn mem_write_u16(&mut self, addr: u16, data: u16) {
      let msb = (data >> 8) as u8;
      let lsb = (data & 0xff) as u8;
      self.mem_write_u8(addr, lsb);
      self.mem_write_u8(addr + 1, msb);
  }

}

pub struct Membank {
  start: usize,
  end: usize,
  size: usize,
  window: usize,
  shift: usize,
  mask: usize,
  banks: Vec<usize>,
  page_count: usize
}

impl Membank {

  pub fn new(start: usize, end: usize, capacity: usize, window: usize) -> Self {

    let size = end - start;
    let mut banks = vec![0; (size + 1) / window];
    for (i, bank) in banks.iter_mut().enumerate() {
      *bank = i * window;
    }

    let page_count = max(1, capacity / window);

    Self {
      start,
      end,
      size,
      window,
      shift: window.trailing_zeros() as usize,
      mask: page_count - 1,
      banks,
      page_count
    }

  }

  pub fn set(&mut self, slot: usize, bank: usize) {
    self.banks[slot] = (bank & self.mask) << self.shift;
  }

  pub fn last(&self) -> usize {
    self.page_count.saturating_sub(1)
  }

  pub fn get_bank(&self, addr: u16) -> usize {
    ((addr as usize) & self.size) >> self.shift
  }

  pub fn translate(&self, addr: u16) -> usize {
    let page = self.banks[self.get_bank(addr)];
    page | (addr as usize) & (self.window - 1)
  }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_bank() {
        let size = 128 * 1024;
        let banks = Membank::new(0x8000, 0xFFFF, size, 0x4000);
        assert_eq!(banks.get_bank(0x8000), 0);
        assert_eq!(banks.get_bank(0x9FFF), 0);
        assert_eq!(banks.get_bank(0xA000), 0);
        assert_eq!(banks.get_bank(0xBFFF), 0);
        assert_eq!(banks.get_bank(0xC000), 1);
        assert_eq!(banks.get_bank(0xDFFF), 1);
        assert_eq!(banks.get_bank(0xE000), 1);
        assert_eq!(banks.get_bank(0xFFFF), 1);
    }

    #[test]
    fn bank_translate() {
        let size = 128 * 1024;
        let mut banks = Membank::new(0x8000, 0xFFFF, size, 0x2000);

        let last_bank = banks.last();
        assert_eq!(last_bank, 15, "bank count");

        assert_eq!(banks.translate(0x8000), 0x0000);
        banks.set(0, 1);
        assert_eq!(banks.translate(0x8000), 0x2000);
        banks.set(0, 2);
        assert_eq!(banks.translate(0x8000), 0x4000);
        banks.set(0, 0);
        assert_eq!(banks.translate(0x8000), 0x0000);
        banks.set(0, banks.last());
        assert_eq!(banks.translate(0x8000), 0x1E000);
    }
}