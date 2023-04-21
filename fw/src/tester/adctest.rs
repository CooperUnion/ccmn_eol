use std::{thread::sleep, time::Duration};

use ccmn_eol_shared::gpiotest::EolGpios;
use esp_idf_sys::{
    esp, ledc_channel_config, ledc_channel_config_t, ledc_clk_cfg_t_LEDC_AUTO_CLK,
    ledc_mode_t_LEDC_LOW_SPEED_MODE, ledc_timer_config, ledc_timer_config_t,
    ledc_timer_t_LEDC_TIMER_0, ledc_channel_t_LEDC_CHANNEL_0, ledc_intr_type_t_LEDC_INTR_DISABLE, ledc_timer_bit_t_LEDC_TIMER_13_BIT, ledc_timer_bit_t_LEDC_TIMER_11_BIT, ledc_timer_bit_t_LEDC_TIMER_9_BIT, ledc_timer_bit_t_LEDC_TIMER_8_BIT, ledc_timer_bit_t_LEDC_TIMER_6_BIT,
};

use crate::opencan::tx::*;

const ADC_PINS: &[u32] = &[
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18,
];

pub fn do_adc_test() -> anyhow::Result<()> {
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

    // for &pin in ADC_PINS {
        esp!(unsafe {
            ledc_channel_config(&ledc_channel_config_t {
                gpio_num: 1 as _,
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
    // }

    println!("# ADC Test Sleep");
    sleep(Duration::from_secs(5));
    println!("# ADC Test End");

    Ok(())
}

#[no_mangle]
extern "C" fn CANTX_populate_TESTER_AdcCmd(m: &mut CAN_Message_TESTER_AdcCmd) {}
