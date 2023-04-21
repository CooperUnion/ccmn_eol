//! Main entrypoint to the firmware.
//! `app_main` resets the boot partition to factory and starts ember_tasking.
//! `app_main` exits and leaves behind the rate tasks to continue running.
use std::{panic, thread::sleep, time::Duration};

use atomic::Atomic;
use ccmn_eol_shared::atomics::*;
use esp_idf_sys::esp_restart;

use crate::{
    adctest::do_adc_test,
    canrx_is_node_ok,
    ember_tasking::{ember_rate_funcs_S, ember_tasking_begin},
    gpiotest::do_gpio_test,
    opencan::tx::*,
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

struct _G {
    current_test: Atomic<CAN_TESTER_currentTest::Type>,
}

static _G: _G = _G {
    current_test: Atomic::<_>::new(CAN_TESTER_currentTest::CAN_TESTER_CURRENTTEST_NONE),
};

// app_main
#[no_mangle]
extern "C" fn app_main() {
    panic::set_hook(Box::new(|info| {
        println!("eol tester panic! {info}");
    }));

    unsafe {
        ember_bltools_set_boot_partition_to_factory();

        // println!("SWITCHING TO FREELUNCH CONSOLE...");
        // crate::freelunch::freelunch_init();

        println!("***~~~ CCMN EOL Testing TESTER ~~~***");
        println!("firmware githash: {}", git_version::git_version!());
        println!("starting tasking...\n");

        ember_tasking_begin();

        // wait for a DUT power cycle
        while canrx_is_node_ok!(DUT) {
            sleep(Duration::from_millis(20));
            println!("waiting for dut to reboot...");
        }
        while !canrx_is_node_ok!(DUT) {
            sleep(Duration::from_millis(20));
            println!("waiting for dut to start up again...");
        }

        glo_w!(
            current_test,
            CAN_TESTER_currentTest::CAN_TESTER_CURRENTTEST_GPIO_TEST
        );
        dbg!(do_gpio_test()).ok()

        glo_w!(
            current_test,
            CAN_TESTER_currentTest::CAN_TESTER_CURRENTTEST_ADC_TEST
        );
        dbg!(do_adc_test()).ok();
        glo_w!(
            current_test,
            CAN_TESTER_currentTest::CAN_TESTER_CURRENTTEST_NONE
        );
        sleep(Duration::from_secs(1));

        esp_restart();
    }
}

#[no_mangle]
extern "C" fn CANTX_populate_TESTER_TestCmd(m: &mut CAN_Message_TESTER_TestCmd) {
    m.TESTER_currentTest = glo!(current_test);
}
