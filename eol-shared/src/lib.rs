use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct TestResults {
    pub gpio_result: bool,
    pub adc_result: Option<(u32, i32)>,
    pub eeprom_result: u8,
}
