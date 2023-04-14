use ccmn_eol_shared::static_gpio;

use crate::ember_tasking::ember_rate_funcs_S;

use crate::pins;
use ccmn_eol_shared::atomics::*;

pub static RATE_FUNCS: ember_rate_funcs_S = ember_rate_funcs_S {
    call_init: Some(leds_init),
    call_1Hz: None,
    call_10Hz: Some(leds_10hz),
    call_100Hz: None,
    call_1kHz: None,
};

struct G {
    counter_10hz: AtomicU8,
}

static _G: G = G {
    counter_10hz: AtomicU8::new(0),
};

static_gpio!(LED1, pins::NODE_BOARD_PIN_LED1, Output);
static_gpio!(LED2, pins::NODE_BOARD_PIN_LED2, Output);

extern "C" fn leds_init() {
    LED1.init();
    LED2.init();
}

extern "C" fn leds_10hz() {
    let count = glo!(counter_10hz) % 10;
    glo_w!(counter_10hz, count + 1);

    LED1.set(count >= 5); // flip every half second
    LED2.set((count % 2) == 0);

    println!("hello.. {}", glo!(counter_10hz));
}
