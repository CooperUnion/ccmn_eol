use anyhow::{anyhow, Result};
use atomic::Atomic;
use ccmn_eol_shared::atomics::*;

use crate::libeeprom;
use crate::opencan::tx::*;

static EEPROM_ADDR: u16 = 0xDEAD;
static TEST_DATA: [u8; 8] = [68, 69, 65, 68, 66, 69, 69, 70];

struct _G {
    test_status: Atomic<Option<bool>>,
}

static _G: _G = _G {
    test_status: Atomic::<_>::new(None),
};

/// Write and read from the eeprom
pub fn eeprom_eol_test() -> Result<()> {
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
        glo_w!(test_status, Some(true));
        Ok(())
    } else {
        println!("Warning: eeprom is not working!");
        glo_w!(test_status, Some(false));
        Err(anyhow!("EEPROM test fail"))
    }
}

#[no_mangle]
extern "C" fn CANTX_populate_DUT_EepromTestStatus(m: &mut CAN_Message_DUT_EepromTestStatus) {
    m.DUT_testStatus = match glo!(test_status) {
        Some(true) => CAN_DUT_testStatus::CAN_DUT_TESTSTATUS_TEST_PASSED,
        Some(false) => CAN_DUT_testStatus::CAN_DUT_TESTSTATUS_TEST_FAILED,
        None => CAN_DUT_testStatus::CAN_DUT_TESTSTATUS_NOT_RUN,
    };
}
