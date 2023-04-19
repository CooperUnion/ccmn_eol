use anyhow::anyhow;
use ccmn_eol_shared::gpio::{GpioPin, gpio, GpioMode};

use crate::opencan::tx::*;

const GPIOS: &[GpioPin] = &[
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
    gpio!(19, InputOutput),
    gpio!(33, InputOutput),
    gpio!(34, InputOutput),
    gpio!(35, InputOutput),
    gpio!(36, InputOutput),
    gpio!(37, InputOutput),
    gpio!(40, InputOutput),
    gpio!(47, InputOutput),
    gpio!(48, InputOutput),
];

/// Get a bitmask representing the state of all the GPIOs.
pub fn gpio_state() -> u64 {
    let mut mask = 0;

    for pin in GPIOS {
        pin.init();
        pin.set_dir(GpioMode::Input);
        mask |= (pin.get() as u64) << pin.pad();
    }

    mask
}

pub fn do_gpio_test() -> anyhow::Result<()> {
    // for each PLAIN_GPIO, send a CAN command to turn only that one on,
    // and then check that we see the same thing ourselves.
    let state = gpio_state();
    if state != 0 {
        return Err(anyhow!("Starting GPIO state is not 0, it's: {state:0b}!"));
    }

    Ok(())
}

#[no_mangle]
extern "C" fn CANTX_populate_TESTER_GpioCmd(m: &mut CAN_Message_TESTER_GpioCmd) {
    m.TESTER_currentGpio = CAN_TESTER_currentGpio::CAN_TESTER_CURRENTGPIO_NONE as _;
}
