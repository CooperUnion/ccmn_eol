//! Main entrypoint to the firmware.
//! `app_main` resets the boot partition to factory and starts ember_tasking.
//! `app_main` exits and leaves behind the rate tasks to continue running.
use std::{panic, thread::sleep, time::Duration};

use esp_idf_sys::esp_restart;

use crate::{
    canrx, canrx_is_node_ok,
    ember_tasking::{ember_rate_funcs_S, ember_tasking_begin}, imports::opencan::rx::CAN_TESTER_currentTest::{CAN_TESTER_CURRENTTEST_GPIO_TEST, CAN_TESTER_CURRENTTEST_ADC_TEST},
};

// some extern declarations
extern "C" {
    // temp: skip generating bindings to ember-bltools for now
    fn ember_bltools_set_boot_partition_to_factory();
    static can_rf: ember_rate_funcs_S;
}

// ember_task_list and ember_task_count
#[no_mangle]
static ember_task_list: [&ember_rate_funcs_S; 2] = [unsafe { &can_rf }, &crate::leds::RATE_FUNCS];

#[no_mangle]
static ember_task_count: usize = ember_task_list.len();

// app_main
#[no_mangle]
extern "C" fn app_main() {
    panic::set_hook(Box::new(|info| {
        println!("eol tester panic! {info}");
    }));

    unsafe {
        ember_bltools_set_boot_partition_to_factory();

        println!("***~~~ CCMN EOL Testing DUT ~~~***");
        println!("firmware githash: {}", git_version::git_version!());
        println!("starting tasking...\n");

        ember_tasking_begin();

        while !canrx_is_node_ok!(TESTER) {
            sleep(Duration::from_millis(20));
            println!("waiting for tester... {}", canrx!(TESTER_currentTest));
        }

        dbg!(do_tests()).ok();

        sleep(Duration::from_secs(1));

        esp_restart();
    }
}

fn do_tests() -> anyhow::Result<()> {
    crate::eeprom::eeprom_eol_test()?;
    while !canrx_is_node_ok!(TESTER) || canrx!(TESTER_currentTest) != CAN_TESTER_CURRENTTEST_GPIO_TEST {
        sleep(Duration::from_millis(10));
    }
    crate::gpiotest::do_gpio_output_test()?;

    while !canrx_is_node_ok!(TESTER) || canrx!(TESTER_currentTest) != CAN_TESTER_CURRENTTEST_ADC_TEST {
        sleep(Duration::from_millis(10));
    }
    crate::adctest::do_adc_test()?;

    Ok(())
}
