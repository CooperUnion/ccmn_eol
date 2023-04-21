use std::{thread::sleep, time::Duration};

use ccmn_eol_shared::gpiotest::EolGpios;

use crate::opencan::tx::*;

pub fn do_adc_test() -> anyhow::Result<()> {
    let gpios = EolGpios::new();
    gpios.init();
    gpios.set_all_to_output();
    gpios.write_all(0);

    sleep(Duration::from_secs(1));

    Ok(())
}

#[no_mangle]
extern "C" fn CANTX_populate_TESTER_AdcCmd(m: &mut CAN_Message_TESTER_AdcCmd) {}
