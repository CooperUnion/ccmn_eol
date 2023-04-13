#![feature(asm_experimental_arch)]
#![feature(asm_const)]

mod imports {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]

    pub mod ember_tasking {
        include!(concat!(env!("OUT_DIR"), "/ember_tasking.rs"));
    }

    pub mod libeeprom {
        include!(concat!(env!("OUT_DIR"), "/libeeprom.rs"));
    }
}

use imports::{ember_tasking, libeeprom};

mod sys;
pub use sys::*;
mod util;

mod eeprom;
