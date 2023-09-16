pub mod gamepad_register;

use gamepad_register::JoypadButton;

#[derive(Default)]
pub struct Gamepad {
  strobe: bool,
  button_index: u8,
  button_status: JoypadButton,
}

impl Gamepad {

  pub fn new() -> Self {
    Gamepad {
      strobe: false,
      button_index: 0,
      button_status: JoypadButton::from_bits_truncate(0),
    }
  }

  pub fn write(&mut self, data: u8) {
    self.strobe = data & 1 == 1;
    if self.strobe {
        self.button_index = 0
    }
  }

  pub fn read(&mut self) -> u8 {
      if self.button_index > 7 {
          return 1;
      }
      let response = (self.button_status.bits() & (1 << self.button_index)) >> self.button_index;
      if !self.strobe && self.button_index <= 7 {
          self.button_index += 1;
      }
      response
  }

  pub fn set_button_pressed_status(&mut self, button: JoypadButton, pressed: bool) {
    self.button_status.set(button, pressed);
}

}