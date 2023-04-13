use esp_idf_sys::{
    gpio_config, gpio_config_t, gpio_get_level, gpio_set_level, GPIO_MODE_DEF_OUTPUT,
};

use crate::util::bit64;

pub enum GpioMode {
    Output,
}

pub struct GpioPin {
    pad: u32,
    mode: GpioMode,
}

pub struct GpioPinBuilder(GpioPin);

impl GpioPin {
    pub const fn new(pad: u32, mode: GpioMode) -> GpioPin {
        GpioPin { pad, mode }
    }

    pub fn set(&self, level: bool) {
        unsafe {
            gpio_set_level(self.pad as i32, level as u32);
        }
    }

    pub fn get(&self) -> bool {
        unsafe { gpio_get_level(self.pad as i32) != 0 }
    }
}

impl GpioPin {
    pub fn init(&self) {
        unsafe {
            gpio_config(&gpio_config_t {
                pin_bit_mask: bit64!(self.pad),
                mode: match self.mode {
                    GpioMode::Output => GPIO_MODE_DEF_OUTPUT,
                },
                ..Default::default()
            });
        }
    }
}

#[macro_export]
macro_rules! static_gpio {
    ($name: ident, $pad: expr, $mode: ident) => {
        static $name: ccmn_eol_shared::gpio::GpioPin = ccmn_eol_shared::gpio::GpioPin::new($pad, ccmn_eol_shared::gpio::GpioMode::$mode);
    };
}

pub use static_gpio;
