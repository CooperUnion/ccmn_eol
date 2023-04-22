use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct TestResults {
    pub gpio_result: bool,
    pub adc_result: Option<(u32, i32)>,
    pub eeprom_result: u8,
}

pub const TEST_RESULT_START_MAGIC: &'static str = "$#$#$";
