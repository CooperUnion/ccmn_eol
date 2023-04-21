use crate::opencan::tx::*;

#[no_mangle]
extern "C" fn CANTX_populate_TESTER_AdcCmd(m: &mut CAN_Message_TESTER_AdcCmd) {}
