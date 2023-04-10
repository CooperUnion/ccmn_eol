use crate::opencan;

#[no_mangle]
extern "C" fn ember_can_callback_notify_lost_can() {

}

#[no_mangle]
extern "C" fn CANTX_populate_HOST_Status(_m: &mut opencan::tx::CAN_Message_HOST_Status) {
}
