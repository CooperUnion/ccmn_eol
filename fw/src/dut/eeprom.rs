use crate::libeeprom;
use crate::opencan::tx::*;

static EEPROM_ADDR: u16 = 0xDEAD;
static TEST_DATA: [u8; 8] = [68, 69, 65, 68, 66, 69, 69, 70];

static mut EEPROM_TEST_STATUS: Option<bool> = None;

/// Write and read from the eeprom
pub fn eeprom_eol_test() {
    unsafe {
        libeeprom::eeprom_init();
        libeeprom::eeprom_write(EEPROM_ADDR, TEST_DATA.as_ptr(), TEST_DATA.len());
    }

    let mut buff: [u8; 8] = [0; 8];
    unsafe {
        libeeprom::eeprom_read(EEPROM_ADDR, buff.as_mut_ptr(), TEST_DATA.len());
    }

    if buff.eq(&TEST_DATA) {
        println!("eeprom data: {:?}", std::str::from_utf8(&buff).unwrap());
        unsafe {
            EEPROM_TEST_STATUS = Some(true);
        }
    } else {
        println!("Warning: eeprom is not working!");

        unsafe {
            EEPROM_TEST_STATUS = Some(false);
        }
    }
}

#[no_mangle]
extern "C" fn CANTX_populate_DUT_EepromTestStatus(m: &mut CAN_Message_DUT_EepromTestStatus) {
    m.DUT_testStatus = match unsafe { EEPROM_TEST_STATUS } {
        Some(true) => CAN_DUT_testStatus_CAN_DUT_TESTSTATUS_TEST_PASSED,
        Some(false) => CAN_DUT_testStatus_CAN_DUT_TESTSTATUS_TEST_FAILED,
        None => CAN_DUT_testStatus_CAN_DUT_TESTSTATUS_NOT_RUN,
    };
}
