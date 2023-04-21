use std::ptr;

use esp_idf_sys::{
    adc_atten_t_ADC_ATTEN_DB_0, adc_bitwidth_t_ADC_BITWIDTH_DEFAULT,
    adc_cali_create_scheme_curve_fitting, adc_cali_curve_fitting_config_t, adc_cali_handle_t,
    adc_cali_raw_to_voltage, adc_channel_t, adc_oneshot_chan_cfg_t, adc_oneshot_config_channel,
    adc_oneshot_new_unit, adc_oneshot_read, adc_oneshot_unit_handle_t, adc_oneshot_unit_init_cfg_t,
    adc_ulp_mode_t_ADC_ULP_MODE_DISABLE, adc_unit_t, esp, EspError,
    ADC_CALI_SCHEME_CURVE_FITTING_SUPPORTED,
};
use static_assertions::const_assert_eq;

#[derive(Debug)]
pub struct Adc {
    handle: adc_oneshot_unit_handle_t,
    cali_handle: adc_cali_handle_t,
}

impl Adc {
    /// New ADC.
    ///
    /// Initializes, configures, and makes calibration scheme.
    pub fn new_and_init(channels: &[adc_channel_t], unit: adc_unit_t) -> Result<Adc, EspError> {
        // ADC init
        let init_cfg = adc_oneshot_unit_init_cfg_t {
            unit_id: unit,
            ulp_mode: adc_ulp_mode_t_ADC_ULP_MODE_DISABLE,
        };

        let mut handle: adc_oneshot_unit_handle_t = ptr::null_mut();
        esp!(unsafe { adc_oneshot_new_unit(&init_cfg, &mut handle) })?;
        let handle = handle;

        // ADC config
        let atten = adc_atten_t_ADC_ATTEN_DB_0;
        for &channel in channels {
            esp!(unsafe {
                adc_oneshot_config_channel(
                    handle,
                    channel,
                    &adc_oneshot_chan_cfg_t {
                        atten,
                        bitwidth: adc_bitwidth_t_ADC_BITWIDTH_DEFAULT,
                    },
                )
            })?;
        }

        // ADC calibration init
        // check that we use curve fittin
        const_assert_eq!(ADC_CALI_SCHEME_CURVE_FITTING_SUPPORTED, 1);

        let mut cali_handle: adc_cali_handle_t = ptr::null_mut();
        esp!(unsafe {
            adc_cali_create_scheme_curve_fitting(
                &adc_cali_curve_fitting_config_t {
                    unit_id: unit,
                    atten,
                    bitwidth: adc_bitwidth_t_ADC_BITWIDTH_DEFAULT,
                },
                &mut cali_handle,
            )
        })?;

        Ok(Self {
            handle,
            cali_handle,
        })
    }

    /// Get calibrated ADC reading for given channel in mV.
    pub fn read(&self, channel: adc_channel_t) -> Result<i16, EspError> {
        let mut raw = -1;
        esp!(unsafe { adc_oneshot_read(self.handle, channel, &mut raw) })?;

        let mut cali_mv = -1;
        esp!(unsafe { adc_cali_raw_to_voltage(self.cali_handle, raw, &mut cali_mv) })?;

        Ok(cali_mv as _)
    }
}
