use esp_idf_sys::{
    gpio_config, gpio_config_t, gpio_get_level, gpio_set_level,
    gpio_mode_t_GPIO_MODE_INPUT, gpio_mode_t_GPIO_MODE_OUTPUT, gpio_mode_t_GPIO_MODE_INPUT_OUTPUT, gpio_set_direction,
};

use crate::util::bit64;

#[derive(Clone, Copy, Debug)]
#[repr(u32)]
pub enum GpioMode {
    Input = gpio_mode_t_GPIO_MODE_INPUT,
    Output = gpio_mode_t_GPIO_MODE_OUTPUT,
    InputOutput = gpio_mode_t_GPIO_MODE_INPUT_OUTPUT,
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

    pub fn set_dir(&self, mode: GpioMode) {
        unsafe {
            gpio_set_direction(self.pad as i32, mode as u32);
        }
    }

    pub fn set(&self, level: bool) {
        unsafe {
            gpio_set_level(self.pad as i32, level as u32);
        }
    }

    pub fn get(&self) -> bool {
        unsafe { gpio_get_level(self.pad as i32) != 0 }
    }

    pub fn pad(&self) -> u32 {
        self.pad
    }
}

impl GpioPin {
    pub fn init(&self) {
        unsafe {
            gpio_config(&gpio_config_t {
                pin_bit_mask: bit64!(self.pad),
                mode: self.mode as u32,
                ..Default::default()
            });
        }
    }
}

#[macro_export]
macro_rules! gpio {
    ($pad: expr, $mode: ident) => {
        crate::gpio::GpioPin::new($pad, crate::gpio::GpioMode::$mode)
    };
}

pub use gpio;


#[macro_export]
macro_rules! static_gpio {
    ($name: ident, $pad: expr, $mode: ident) => {
        static $name: ccmn_eol_shared::gpio::GpioPin =
            ccmn_eol_shared::gpio::GpioPin::new($pad, ccmn_eol_shared::gpio::GpioMode::$mode);
    };
}

pub use static_gpio;
