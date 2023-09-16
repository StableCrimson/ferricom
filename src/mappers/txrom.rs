use crate::{rom::{ROM, ScreenMirroring}, mem::Membank};

use super::{Map, Mapper, MappedRead, MappedWrite};

const PRG_RAM_SIZE: u16 = 0x2000;
const CHR_RAM_SIZE: u16 = 0x2000;

const CHR_RAM_START: usize = 0x0000;
const CHR_RAM_END: usize = 0x1FFF;
const PRG_RAM_START: usize = 0x6000;
const PRG_RAM_END: usize = 0x7FFF;
const PRG_ROM_START: usize = 0x8000;
const PRG_ROM_END: usize = 0xFFFF;

#[derive(Debug, Clone)]
struct TxRegs {
    bank_select: u8,
    bank_values: [u8; 8],
    irq_latch: u8,
    // irq_counter: u8,
    irq_enabled: bool,
    irq_reload: bool,
    // last_clock: u16,
}

impl TxRegs {
  const fn new() -> Self {
    Self {
      bank_select: 0x00,
      bank_values: [0x00; 8],
      irq_latch: 0x00,
      // irq_counter: 0x00,
      irq_enabled: false,
      irq_reload: false,
      // last_clock: 0x0000,
    }
  }
}

pub struct TXROM {
  mirroring: ScreenMirroring,
  regs: TxRegs,
  prg_rom_banks: Membank,
  prg_ram_banks: Membank,
  chr_banks: Membank,
  irq_pending: bool,
}

impl TXROM {

  pub fn load(rom: &mut ROM) -> Mapper {

    rom.prg_ram = vec![0; PRG_RAM_SIZE as usize];

    if !rom.has_chr_rom() {
      rom.chr_ram = vec![0; CHR_RAM_SIZE as usize];
    }

    if rom.header.mirroring == ScreenMirroring::FourScreen {
      rom.ex_ram = vec![0; 0x1000];
    }

    let mut txrom = Self {
      mirroring: rom.header.mirroring,
      regs: TxRegs::new(),
      chr_banks: Membank::new(CHR_RAM_START, CHR_RAM_END, rom.chr_ram.len(), 0x400),
      prg_rom_banks: Membank::new(PRG_ROM_START, PRG_ROM_END, rom.prg_rom.len(), 0x2000),
      prg_ram_banks: Membank::new(PRG_RAM_START, PRG_RAM_END, rom.prg_ram.len(), 0x2000),
      irq_pending: false,
    };

    let last_bank = txrom.prg_rom_banks.last();
    txrom.prg_rom_banks.set(2, last_bank-1);
    txrom.prg_rom_banks.set(3, last_bank);
    txrom.into()

  }

  fn update_banks(&mut self) {

    let prg_last = self.prg_rom_banks.last();
    let prg_lo = self.regs.bank_values[6] as usize;
    let prg_hi = self.regs.bank_values[7] as usize;

    if self.regs.bank_select & 0x40 == 0x40 {
        self.prg_rom_banks.set(0, prg_last - 1);
        self.prg_rom_banks.set(1, prg_hi);
        self.prg_rom_banks.set(2, prg_lo);
    } else {
        self.prg_rom_banks.set(0, prg_lo);
        self.prg_rom_banks.set(1, prg_hi);
        self.prg_rom_banks.set(2, prg_last - 1);
    }
    self.prg_rom_banks.set(3, prg_last);

    let chr = self.regs.bank_values;

    if self.regs.bank_select & 0x80 == 0x80 {
        self.chr_banks.set(0, chr[2] as usize);
        self.chr_banks.set(1, chr[3] as usize);
        self.chr_banks.set(2, chr[4] as usize);
        self.chr_banks.set(3, chr[5] as usize);
        self.chr_banks.set_range(4, 5, (chr[0] & 0xFE) as usize);
        self.chr_banks.set_range(6, 7, (chr[1] & 0xFE) as usize);
    } else {
        self.chr_banks.set_range(0, 1, (chr[0] & 0xFE) as usize);
        self.chr_banks.set_range(2, 3, (chr[1] & 0xFE) as usize);
        self.chr_banks.set(4, chr[2] as usize);
        self.chr_banks.set(5, chr[3] as usize);
        self.chr_banks.set(6, chr[4] as usize);
        self.chr_banks.set(7, chr[5] as usize);
    }
  }

}

impl Map for TXROM {

  fn map_read(&mut self, _addr: u16) -> MappedRead { self.map_peak(_addr) }

  fn map_peak(&self, _addr: u16) -> MappedRead { 
    match _addr as usize {
      CHR_RAM_START..=CHR_RAM_END => MappedRead::Chr(self.chr_banks.translate(_addr)),
      PRG_RAM_START..=PRG_RAM_END => MappedRead::PrgRAM(self.prg_ram_banks.translate(_addr)),
      PRG_ROM_START..=PRG_ROM_END => MappedRead::PrgROM(self.prg_rom_banks.translate(_addr)),
      _ => MappedRead::None
    }
  }

  fn map_write(&mut self, _addr: u16, _data: u8) -> MappedWrite {

    match _addr as usize {

      CHR_RAM_START..=CHR_RAM_END => MappedWrite::Chr(self.chr_banks.translate(_addr), _data),
      PRG_RAM_START..=PRG_RAM_END => MappedWrite::PrgRAM(self.prg_ram_banks.translate(_addr), _data),
      PRG_ROM_START..=PRG_ROM_END => {

        match _addr & 0xE001 {

          0x8000 => {
            self.regs.bank_select = _data;
            self.update_banks();
          },
          0x8001 => {
            let bank = self.regs.bank_select & 0x07;
            self.regs.bank_values[bank as usize] = _data;
            self.update_banks();
          },
          0xA000 => {
            if self.mirroring != ScreenMirroring::FourScreen {
              self.mirroring = match _data & 0x01 {
                0 => ScreenMirroring::Vertical,
                1 => ScreenMirroring::Horizontal,
                _ => unreachable!("You shouldn't be here")
              };
              self.update_banks();
            }
          },
          0xC000 => self.regs.irq_latch = _data,
          0xC001 => self.regs.irq_reload = true,
          0xE000 => {
            self.irq_pending = false;
            self.regs.irq_enabled = true;
          },
          0xE001 => self.regs.irq_enabled = false,
          _ => unimplemented!("You shouldn't be here")

        }
        MappedWrite::None

      }
      _ => MappedWrite::None

    }

  }

}