use crate::opencan::tx::*;

#[no_mangle]
extern "C" fn CANTX_populate_TESTER_GpioCmd(m: &mut CAN_Message_TESTER_GpioCmd) {
    m.TESTER_currentGpio = CAN_TESTER_currentGpio::CAN_TESTER_CURRENTGPIO_NONE as _;
}
