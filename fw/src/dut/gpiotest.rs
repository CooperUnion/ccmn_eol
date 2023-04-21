use std::{thread::sleep, time::Duration};

use crate::opencan::rx::*;
use anyhow::anyhow;
use ccmn_eol_shared::gpiotest::EolGpios;

use crate::{canrx, canrx_is_node_ok};

pub fn do_gpio_output_test() -> anyhow::Result<()> {
    let gpios = EolGpios::new();
    gpios.init();
    gpios.set_all_to_output();
    gpios.write_all(0);

    loop {
        if !canrx_is_node_ok!(TESTER) {
            return Err(anyhow!("Lost TESTER while running gpio test"));
        }

        if canrx!(TESTER_currentTest) != CAN_TESTER_currentTest::CAN_TESTER_CURRENTTEST_GPIO_TEST {
            break;
        }

        match canrx!(TESTER_currentGpio) {
            CAN_TESTER_currentGpio::CAN_TESTER_CURRENTGPIO_NONE => gpios.write_all(0),
            g => gpios.write_all(1u64 << g),
        };

        sleep(Duration::from_millis(1));
    }

    gpios.set_all_to_input();

    Ok(())
}
