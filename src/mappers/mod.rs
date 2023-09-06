use crate::rom::ScreenMirroring;

pub mod nrom;

use enum_dispatch::enum_dispatch;
use nrom::NROM;

pub struct Empty;
impl Map for Empty {}

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

  fn mirroring(&self) -> ScreenMirroring {
    ScreenMirroring::Vertical
  }

}