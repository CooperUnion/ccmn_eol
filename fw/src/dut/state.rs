// use crate::opencan;

#[no_mangle]
extern "C" fn ember_can_callback_notify_lost_can() {}

// static mut counter: u8 = 0;

// #[no_mangle]
// extern "C" fn CANTX_populate_HOST_Status(m: &mut opencan::tx::CAN_Message_HOST_Status) {
//     // Testing (for my understanding, this will be deleted)
//     unsafe {
//         m.HOST_counter = counter;
//         counter += 1;
//     }
// }
