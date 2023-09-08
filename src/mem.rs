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