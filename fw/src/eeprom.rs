use crate::ember_tasking::ember_rate_funcs_S;
use crate::libeeprom;

pub static RATE_FUNCS: ember_rate_funcs_S = ember_rate_funcs_S {
    call_init: Some(eeprom_init),
    call_1Hz: Some(eeprom_1hz),
    call_10Hz: None,
    call_100Hz: None,
    call_1kHz: None,
};

static EEPROM_ADDR: u16 = 0xDEAD;
static TEST_DATA: [u8; 8] = [68, 69, 65, 67, 66, 69, 69, 70];

extern "C" fn eeprom_init() {
    unsafe {
        libeeprom::eeprom_init();
        libeeprom::eeprom_write(EEPROM_ADDR, TEST_DATA.as_ptr(), TEST_DATA.len());
    }
}

extern "C" fn eeprom_1hz() {
    let mut buff: [u8; 8] = [0; 8];
    unsafe {
        libeeprom::eeprom_read(EEPROM_ADDR, buff.as_mut_ptr(), TEST_DATA.len());
    }

    if buff.eq(&TEST_DATA) {
        println!("eeprom data: {:?}", std::str::from_utf8(&buff).unwrap());
    } else {
        // array's not equal
        println!("Warning: eeprom is not working!");
    }
}
