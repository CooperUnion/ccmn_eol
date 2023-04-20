use anyhow::anyhow;
use ccmn_eol_shared::gpiotest::EolGpios;

use crate::opencan::tx::*;

pub fn do_gpio_test() -> anyhow::Result<()> {
    // for each PLAIN_GPIO, send a CAN command to turn only that one on,
    // and then check that we see the same thing ourselves.
    let gpios = EolGpios::new();
    gpios.init();
    gpios.set_all_to_input();

    let state = gpios.read_all();

    for pin in gpios.pins {
        let pad = pin.pad();
        if (state >> pad & 1) != 1 {
            return Err(anyhow!("GPIO state mismatch on pin {pad}: {state:064b}!"));
        }
    }

    Ok(())
}

#[no_mangle]
extern "C" fn CANTX_populate_TESTER_GpioCmd(m: &mut CAN_Message_TESTER_GpioCmd) {
    m.TESTER_currentGpio = CAN_TESTER_currentGpio::CAN_TESTER_CURRENTGPIO_NONE as _;
}
