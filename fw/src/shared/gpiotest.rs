use crate::gpio::{gpio, GpioMode, GpioPin};

pub struct EolGpios {
    pub pins: &'static [GpioPin],
}

impl Default for EolGpios {
    fn default() -> Self {
        Self::new()
    }
}

impl EolGpios {
    pub fn new() -> Self {
        const EOL_PINS: &[GpioPin] = &[
            gpio!(1, InputOutput),
            gpio!(2, InputOutput),
            gpio!(3, InputOutput),
            gpio!(4, InputOutput),
            gpio!(5, InputOutput),
            gpio!(6, InputOutput),
            gpio!(7, InputOutput),
            gpio!(8, InputOutput),
            gpio!(9, InputOutput),
            gpio!(10, InputOutput),
            gpio!(11, InputOutput),
            gpio!(12, InputOutput),
            gpio!(13, InputOutput),
            gpio!(14, InputOutput),
            gpio!(15, InputOutput),
            gpio!(16, InputOutput),
            gpio!(17, InputOutput),
            gpio!(18, InputOutput),
            gpio!(33, InputOutput),
            gpio!(34, InputOutput),
            gpio!(35, InputOutput),
            gpio!(36, InputOutput),
            gpio!(37, InputOutput),
            gpio!(40, InputOutput),
            gpio!(47, InputOutput),
            gpio!(48, InputOutput),
        ];

        EolGpios { pins: EOL_PINS }
    }

    pub fn init(&self) {
        for pin in self.pins {
            pin.init();
        }
    }

    pub fn set_all_to_input(&self) {
        for pin in self.pins {
            pin.set_dir(GpioMode::Input);
        }
    }

    pub fn set_all_to_output(&self) {
        for pin in self.pins {
            pin.set_dir(GpioMode::Output);
        }
    }

    pub fn write_all(&self, bitmask: u64) {
        // println!("# writing GPIO {bitmask:050b}");
        for pin in self.pins {
            let on = (bitmask >> pin.pad() & 1) != 0;
            pin.set(on);
        }
    }

    /// Get a bitmask representing the state of all the GPIOs.
    pub fn read_all(&self) -> u64 {
        let mut mask = 0;

        for pin in self.pins {
            mask |= (pin.get() as u64) << pin.pad();
        }

        mask
    }
}
