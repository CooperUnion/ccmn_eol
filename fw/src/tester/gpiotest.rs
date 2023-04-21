use std::{thread::sleep, time::Duration};

use anyhow::anyhow;
use atomic::Atomic;
use ccmn_eol_shared::{atomics::*, gpiotest::EolGpios};

use crate::{canrx, canrx_is_node_ok, opencan::tx::*};

struct _G {
    gpio_cmd: Atomic<Option<u8>>,
    test_active: Atomic<bool>,
}

static _G: _G = _G {
    gpio_cmd: Atomic::<_>::new(None),
    test_active: Atomic::<_>::new(false),
};

pub fn do_gpio_test() -> anyhow::Result<()> {
    // for each PLAIN_GPIO, send a CAN command to turn only that one on,
    // and then check that we see the same thing ourselves.
    let gpios = EolGpios::new();
    gpios.init();
    gpios.set_all_to_input();

    glo_w!(gpio_cmd, None);
    glo_w!(test_active, true);

    // wait for a while for DUT to be up
    sleep(Duration::from_secs(1));

    for pin in gpios.pins {
        if !canrx_is_node_ok!(DUT) {
            dbg!(canrx!(DUT_testStatus));
            return Err(anyhow!("Lost DUT while testing gpio!"));
        }

        let pad = pin.pad();
        println!("testing pad {pad}");
        glo_w!(gpio_cmd, Some(pad));
        sleep(Duration::from_millis(20));

        let state = gpios.read_all();
        let desired_state = 1u64 << pad;
        if state != desired_state {
            return Err(anyhow!("GPIO state mismatch on pin {pad}:\n desired {desired_state:064b}\n actual  {state:064b}"));
        }
        println!("pad {pad} ok");
    }

    glo_w!(test_active, false);

    Ok(())
}

#[no_mangle]
extern "C" fn CANTX_populate_TESTER_GpioCmd(m: &mut CAN_Message_TESTER_GpioCmd) {
    m.TESTER_currentGpio = match glo!(gpio_cmd) {
        None => CAN_TESTER_currentGpio::CAN_TESTER_CURRENTGPIO_NONE,
        Some(g) => g.into(),
    };
    m.TESTER_testActive = glo!(test_active) as _;
}
