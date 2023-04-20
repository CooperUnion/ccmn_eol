use std::{thread::sleep, time::Duration};

use crate::opencan::rx::*;
use ccmn_eol_shared::gpiotest::EolGpios;

use crate::{canrx, canrx_is_node_ok};

pub fn do_gpio_test() {
    let gpios = EolGpios::new();
    gpios.init();
    gpios.set_all_to_output();
    gpios.write_all(0);

    while !canrx_is_node_ok!(TESTER) {
        sleep(Duration::from_millis(20));
        println!("waiting for tester... {}", canrx!(TESTER_currentGpio));
    }

    loop {
        match canrx!(TESTER_currentGpio) {
            CAN_TESTER_currentGpio::CAN_TESTER_CURRENTGPIO_NONE => gpios.write_all(0),
            g => gpios.write_all(1u64 << g),
        };
        sleep(Duration::from_millis(1));
    }
}
