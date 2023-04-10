/// Get a single bit set in a 64 bit mask.
///
/// ```
/// assert_eq!(bit64!(3), 0b100 as u64);
/// ```
macro_rules! bit64 {
    ($p:expr) => {
        1u64 << $p
    };
}

pub(crate) use bit64;
