use std::{thread::sleep, time::Duration};

use anyhow::anyhow;
use atomic::Atomic;
use ccmn_eol_shared::{adc::*, atomics::*};
use esp_idf_sys::{
    adc_channel_t, adc_channel_t_ADC_CHANNEL_0, adc_channel_t_ADC_CHANNEL_1,
    adc_channel_t_ADC_CHANNEL_2, adc_channel_t_ADC_CHANNEL_3, adc_channel_t_ADC_CHANNEL_4,
    adc_channel_t_ADC_CHANNEL_5, adc_channel_t_ADC_CHANNEL_6, adc_channel_t_ADC_CHANNEL_7,
    adc_channel_t_ADC_CHANNEL_8, adc_channel_t_ADC_CHANNEL_9, adc_unit_t, adc_unit_t_ADC_UNIT_1,
    adc_unit_t_ADC_UNIT_2,
};

use crate::{
    canrx, canrx_is_node_ok, imports::opencan::tx::CAN_Message_DUT_AdcTestStatus, opencan::rx::*,
};

const ADC_TOLERANCE_MV: i16 = 10;

const ADC_PINS: &[u32] = &[
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18,
];

pub const fn pin_to_adc_channel(pin: u32) -> (adc_unit_t, adc_channel_t) {
    match pin {
        1 => (adc_unit_t_ADC_UNIT_1, adc_channel_t_ADC_CHANNEL_0),
        2 => (adc_unit_t_ADC_UNIT_1, adc_channel_t_ADC_CHANNEL_1),
        3 => (adc_unit_t_ADC_UNIT_1, adc_channel_t_ADC_CHANNEL_2),
        4 => (adc_unit_t_ADC_UNIT_1, adc_channel_t_ADC_CHANNEL_3),
        5 => (adc_unit_t_ADC_UNIT_1, adc_channel_t_ADC_CHANNEL_4),
        6 => (adc_unit_t_ADC_UNIT_1, adc_channel_t_ADC_CHANNEL_5),
        7 => (adc_unit_t_ADC_UNIT_1, adc_channel_t_ADC_CHANNEL_6),
        8 => (adc_unit_t_ADC_UNIT_1, adc_channel_t_ADC_CHANNEL_7),
        9 => (adc_unit_t_ADC_UNIT_1, adc_channel_t_ADC_CHANNEL_8),
        10 => (adc_unit_t_ADC_UNIT_1, adc_channel_t_ADC_CHANNEL_9),
        11 => (adc_unit_t_ADC_UNIT_2, adc_channel_t_ADC_CHANNEL_0),
        12 => (adc_unit_t_ADC_UNIT_2, adc_channel_t_ADC_CHANNEL_1),
        13 => (adc_unit_t_ADC_UNIT_2, adc_channel_t_ADC_CHANNEL_2),
        14 => (adc_unit_t_ADC_UNIT_2, adc_channel_t_ADC_CHANNEL_3),
        15 => (adc_unit_t_ADC_UNIT_2, adc_channel_t_ADC_CHANNEL_4),
        16 => (adc_unit_t_ADC_UNIT_2, adc_channel_t_ADC_CHANNEL_5),
        17 => (adc_unit_t_ADC_UNIT_2, adc_channel_t_ADC_CHANNEL_6),
        18 => (adc_unit_t_ADC_UNIT_2, adc_channel_t_ADC_CHANNEL_7),
        19 | 20 => panic!("Don't test pins 19-20!"),
        _ => panic!("Pin does not have mapping!"),
    }
}

struct _G {
    adc_value: Atomic<(u32, i16)>,
}

static _G: _G = _G {
    adc_value: Atomic::<_>::new((0, 0)),
};

pub fn do_adc_test() -> anyhow::Result<()> {
    let adc1_channels: Vec<u32> = ADC_PINS
        .iter()
        .filter_map(|&p| {
            let (unit, channel) = pin_to_adc_channel(p);
            if unit == adc_unit_t_ADC_UNIT_1 {
                Some(channel)
            } else {
                None
            }
        })
        .collect();

    let adc2_channels: Vec<u32> = ADC_PINS
        .iter()
        .filter_map(|&p| {
            let (unit, channel) = pin_to_adc_channel(p);
            if unit == adc_unit_t_ADC_UNIT_2 {
                Some(channel)
            } else {
                None
            }
        })
        .collect();

    let adc1 = Adc::new_and_init(&adc1_channels, adc_unit_t_ADC_UNIT_1)?;

    let adc2 = Adc::new_and_init(&adc2_channels, adc_unit_t_ADC_UNIT_2)?;

    loop {
        if !canrx_is_node_ok!(TESTER) {
            return Err(anyhow!("Lost TESTER while running adc test"));
        }

        if canrx!(TESTER_currentTest) != CAN_TESTER_currentTest::CAN_TESTER_CURRENTTEST_ADC_TEST {
            break;
        }

        dbg!("adc test loop!");

        match canrx!(TESTER_currentGpio) {
            CAN_TESTER_currentGpio::CAN_TESTER_CURRENTGPIO_NONE => glo_w!(adc_value, (0, 0)),
            g => glo_w!(adc_value, (g, 0)),
        };

        sleep(Duration::from_millis(10));
    }

    Ok(())
}

#[no_mangle]
extern "C" fn CANTX_populate_DUT_AdcTestStatus(m: &mut CAN_Message_DUT_AdcTestStatus) {
    m.DUT_adcTestStatus = 0;
}
