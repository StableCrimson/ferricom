use crate::rom::ScreenMirroring;

pub mod nrom;
pub mod txrom;

use enum_dispatch::enum_dispatch;
use nrom::NROM;
use txrom::TXROM;


pub enum MappedRead {
  None,
  Chr(usize),
  Data(u8),
  PrgROM(usize),
  PrgRAM(usize),
}

pub enum MappedWrite {
  None,
  Chr(usize, u8),
  PrgRAM(usize, u8),
}

#[enum_dispatch]
pub enum Mapper {
  Empty,
  NROM,
  TXROM,
}


impl Mapper {

  pub fn none() -> Self {
    Empty.into()
  }

}

#[enum_dispatch(Mapper)]
pub trait Map {

  fn mirroring(&self) -> ScreenMirroring { ScreenMirroring::Default }
  fn set_mirroring(&mut self, _mirroring: ScreenMirroring) {}
  fn map_read(&self, _addr: u16) -> MappedRead { self.map_peak(_addr) }
  fn map_peak(&self, _addr: u16) -> MappedRead { MappedRead::None }
  fn map_write(&self, _addr: u16, _data: u8) -> MappedWrite { MappedWrite::None }

}

#[derive(Debug)]
pub struct Empty;
impl Map for Empty {}