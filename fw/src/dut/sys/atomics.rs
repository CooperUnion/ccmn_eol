pub use core::sync::atomic::{
    AtomicBool, AtomicI16, AtomicI32, AtomicI64, AtomicI8, AtomicU16, AtomicU32, AtomicU64,
    AtomicU8,
};

macro_rules! glo {
    ($field:ident) => {
        paste::paste! {
            _G.[<$field>].load(core::sync::atomic::Ordering::Relaxed)
        }
    };
}

pub(crate) use glo;

macro_rules! glo_w {
    ($field:ident, $val:expr) => {
        paste::paste! {
            _G.[<$field>].store($val, core::sync::atomic::Ordering::Relaxed)
        }
    };
}

pub(crate) use glo_w;
