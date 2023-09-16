pub enum InterruptType {
  NMI,
}

pub(super) struct Interrupt {
  pub(super) vector_address: u16,
  pub(super) interrupt_flag_mask: u8,
  pub(super) cycles: u8,
}

pub(super) const NMI: Interrupt = Interrupt {
  vector_address: 0xFFFA,
  interrupt_flag_mask: 0b0010_0000,
  cycles: 2
};