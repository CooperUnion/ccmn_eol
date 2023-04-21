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
    canrx, canrx_is_node_ok,
    imports::opencan::tx::{
        CAN_DUT_adcActiveMillivolts, CAN_DUT_adcActivePin, CAN_DUT_adcUniqueness,
        CAN_Message_DUT_AdcTestStatus,
    },
    opencan::rx::*,
};

const ADC_MIN_MV: i16 = 15;

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

#[derive(Clone, Copy, Debug)]
struct AdcData {
    unique: bool,
    pin: u32,
    value: i16,
}

struct _G {
    adc_data: Atomic<Option<AdcData>>,
}

static _G: _G = _G {
    adc_data: Atomic::<_>::new(None),
};

pub fn do_adc_test() -> anyhow::Result<()> {
    println!("# DUT ADC Test Start");
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

        let mut data: Option<AdcData> = None;

        for &pin in ADC_PINS {
            let (unit, channel) = pin_to_adc_channel(pin);

            #[allow(non_upper_case_globals)]
            let val = match unit {
                adc_unit_t_ADC_UNIT_1 => adc1.read(channel),
                adc_unit_t_ADC_UNIT_2 => adc2.read(channel),
                _ => panic!("Invalid ADC unit!"),
            }
            .unwrap();

            // nonzero ADC reading
            if val.abs() > ADC_TOLERANCE_MV {
                data = Some(AdcData {
                    unique: data.is_none(),
                    pin,
                    value: val,
                });
            }
        }
        glo_w!(adc_data, data);

        sleep(Duration::from_millis(5));
    }

    println!("# DUT ADC Test End");

    Ok(())
}

#[no_mangle]
extern "C" fn CANTX_populate_DUT_AdcTestStatus(m: &mut CAN_Message_DUT_AdcTestStatus) {
    let adc_data = glo!(adc_data);

    *m = match adc_data {
        None => CAN_Message_DUT_AdcTestStatus {
            DUT_adcUniqueness: CAN_DUT_adcUniqueness::CAN_DUT_ADCUNIQUENESS_NONE,
            DUT_adcActivePin: CAN_DUT_adcActivePin::CAN_DUT_ADCACTIVEPIN_NONE,
            DUT_adcActiveMillivolts: CAN_DUT_adcActiveMillivolts::CAN_DUT_ADCACTIVEMILLIVOLTS_NONE,
        },
        Some(d) => CAN_Message_DUT_AdcTestStatus {
            DUT_adcUniqueness: if d.unique {
                CAN_DUT_adcUniqueness::CAN_DUT_ADCUNIQUENESS_UNIQUE
            } else {
                CAN_DUT_adcUniqueness::CAN_DUT_ADCUNIQUENESS_NOT_UNIQUE
            },
            DUT_adcActivePin: d.pin as _,
            DUT_adcActiveMillivolts: d.value as _,
        },
    };
}
