use std::{thread::sleep, time::Duration};

use anyhow::anyhow;
use ccmn_eol_shared::{gpiotest::EolGpios, with_interrupts_disabled};
use esp_idf_sys::{
    esp, ledc_channel_config, ledc_channel_config_t, ledc_clk_cfg_t_LEDC_AUTO_CLK,
    ledc_mode_t_LEDC_LOW_SPEED_MODE, ledc_timer_config, ledc_timer_config_t,
    ledc_timer_t_LEDC_TIMER_0, ledc_channel_t_LEDC_CHANNEL_0, ledc_intr_type_t_LEDC_INTR_DISABLE, ledc_timer_bit_t_LEDC_TIMER_6_BIT,
};

use crate::{opencan::tx::*, canrx, imports::opencan::rx::CAN_DUT_adcUniqueness};

const ADC_PINS: &[u32] = &[
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18,
];

pub fn do_adc_test() -> anyhow::Result<(u32, i32)> {
    println!("# ADC Test Start");
    let gpios = EolGpios::new();
    gpios.init();
    gpios.set_all_to_output();
    gpios.write_all(0);

    // okay, now let's do some LEDC stuff..
    esp!(unsafe {
        ledc_timer_config(&ledc_timer_config_t {
            speed_mode: ledc_mode_t_LEDC_LOW_SPEED_MODE,
            duty_resolution: ledc_timer_bit_t_LEDC_TIMER_6_BIT,
            timer_num: ledc_timer_t_LEDC_TIMER_0,
            freq_hz: 1250000,
            clk_cfg: ledc_clk_cfg_t_LEDC_AUTO_CLK,
        })
    }).unwrap();

                                // pin,      value
    let mut largest_tolerance: (Option<u32>, Option<i32>) = (None, None);

    for &pin in ADC_PINS {
        println!("#  testing ADC pin {pin}");
        gpios.init();
        gpios.write_all(0);

        esp!(unsafe {
            ledc_channel_config(&ledc_channel_config_t {
                gpio_num: pin as _,
                speed_mode: ledc_mode_t_LEDC_LOW_SPEED_MODE,
                channel: ledc_channel_t_LEDC_CHANNEL_0,
                intr_type: ledc_intr_type_t_LEDC_INTR_DISABLE,
                timer_sel: ledc_timer_t_LEDC_TIMER_0,
                duty: 6,
                hpoint: 0,
                flags: Default::default()
            })
        }).unwrap();

        // ok, the pin should be PWMing now.
        // wait for a little bit for the value to stabilize...
        // the RC filtering on the tester unit is a little silly.
        sleep(Duration::from_millis(400));

        let (uniqueness, active_pin, millivolts) = with_interrupts_disabled! {(
            canrx!(DUT_adcUniqueness),
            canrx!(DUT_adcActivePin),
            canrx!(DUT_adcActiveMillivolts),
        )};

        match uniqueness {
            CAN_DUT_adcUniqueness::CAN_DUT_ADCUNIQUENESS_NONE => return Err(anyhow!("ADC uniquness result for pin {pin} was NONE: is there a disconnected pin?")),
            CAN_DUT_adcUniqueness::CAN_DUT_ADCUNIQUENESS_NOT_UNIQUE => return Err(anyhow!("ADC uniquness result for pin {pin} was NOT_UNIQUE: are there bridged pins?")),
            CAN_DUT_adcUniqueness::CAN_DUT_ADCUNIQUENESS_UNIQUE => {},
            _ => panic!("Invalid ADC uniqueness value from CAN"),
        }

        if active_pin != pin {
            return Err(anyhow!("ADC active pin was unexpectedly {active_pin}, but should have been {pin}. Are there bridged/disconnected pins?"));
        }

        const EXPECTED_RESULT_MV: i32 = 306;
        const ACCEPTABLE_ADC_TOLERANCE_MV: i32 = 15;
        let tolerance = millivolts as i32 - EXPECTED_RESULT_MV;

        if tolerance.abs() > ACCEPTABLE_ADC_TOLERANCE_MV {
            return Err(anyhow!("ADC result was out of spec for pin {pin}: needed {EXPECTED_RESULT_MV}+-{ACCEPTABLE_ADC_TOLERANCE_MV} mV, but got {millivolts} mV."));
        }

        if largest_tolerance.0.is_none() {
            largest_tolerance = (Some(pin), Some(tolerance));
        }

        if let (_, Some(prev_tolerance)) = largest_tolerance {
            if tolerance.abs() > prev_tolerance.abs() {
                largest_tolerance = (Some(pin), Some(tolerance));
            }
        }
        println!("#  ADC pin {pin} ok; tolerance = {tolerance} mV");
    }

    println!("# ADC Test End");

    Ok((largest_tolerance.0.unwrap(), largest_tolerance.1.unwrap()))
}

#[no_mangle]
extern "C" fn CANTX_populate_TESTER_AdcCmd(m: &mut CAN_Message_TESTER_AdcCmd) {}
