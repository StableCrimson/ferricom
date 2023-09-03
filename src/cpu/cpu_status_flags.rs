use bitflags::bitflags;

bitflags! {

    /// Aliases for the flags in the 6502 status register.
    /// More information on these flags can be found here: <https://www.nesdev.org/wiki/Status_flags>
    #[derive(Clone, Copy, Debug)]
    pub struct CPUFlags: u8 {
        const CARRY =              0b0000_0001;
        const ZERO =               0b0000_0010;
        const INTERRUPT_DISABLE =  0b0000_0100;
        const DECIMAL_MODE =       0b0000_1000;

        /*
            Bits 4 and 5 are somewhat unused.
            They are used to represent any of 4 interrupt status types
        */
        const BREAK_COMMAND_4 =    0b0001_0000;
        const BREAK_COMMAND_5 =    0b0010_0000;

        const OVERFLOW =           0b0100_0000;
        const NEGATIVE =           0b1000_0000;
    }
}