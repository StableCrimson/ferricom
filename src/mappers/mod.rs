use crate::rom::ScreenMirroring;

pub mod nrom;

use enum_dispatch::enum_dispatch;
use nrom::NROM;

pub enum MappedRead {
  None,
  Chr(u16),
  Data(u8),
  PrgROM(u16),
  PrgRAM(u16),
}

pub enum MappedWrite {
  None,
  Chr(u16, u8),
  PrgRAM(u16, u8),
}

#[enum_dispatch]
pub enum Mapper {
  Empty,
  NROM
}


impl Mapper {

  pub fn none() -> Self {
    Empty.into()
  }

}

#[enum_dispatch(Mapper)]
pub trait Map {

  fn mirroring(&self) -> ScreenMirroring { ScreenMirroring::FourScreen }
  fn map_read(&self, _addr: u16) -> MappedRead { self.map_peak(_addr) }
  fn map_peak(&self, _addr: u16) -> MappedRead { MappedRead::None }
  fn map_write(&self, _addr: u16, _data: u8) -> MappedWrite { MappedWrite::None }

}

#[derive(Debug)]
pub struct Empty;
impl Map for Empty {}