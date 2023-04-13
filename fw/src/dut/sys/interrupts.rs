use core::arch::asm;

use static_assertions::const_assert_eq;

// see esp-idf/components/xtensa/include/xtensa/xtruntime.h
// see XTOS_SET_INTLEVEL, portDISABLE_INTERRUPTS, portRESTORE_INTERRUPTS, portENABLE_INTERRUPTS

// XCHAL_HAVE_XEA2 is what the implementation is gated with in esp-idf
const_assert_eq!(esp_idf_sys::XCHAL_HAVE_XEA2, 1);

pub struct PrevInterruptLevel(u32);

/// Enable interrupts (set interrupt level 0).
///
/// # Safety
/// Caller needs to not call this from ISR context.
#[inline(always)]
pub unsafe fn enable_interrupts() {
    unsafe {
        asm!(
            "rsil {}, 0",
            out(reg) _,
            options(nomem, nostack),
        );
    }
}

/// Disable interrupts.
///
/// Note: [`with_interrupts_disabled!`] is a strongly recommended alternative.
///
/// # Safety
/// Caller needs to remember to reenable interrupts and not call this from ISR context.
#[inline(always)]
pub unsafe fn disable_interrupts() -> PrevInterruptLevel {
    let prev: u32;

    unsafe {
        asm!(
            "rsil {}, {}",
            out(reg) prev,
            const esp_idf_sys::XCHAL_EXCM_LEVEL,
            options(nomem, nostack),
        );
    }

    PrevInterruptLevel(prev)
}

/// Restore interrupts to given level (returned from disable_interrupts()).
///
/// # Safety
/// Caller needs to not call this from ISR context.
#[inline(always)]
pub unsafe fn restore_interrupts(level: PrevInterruptLevel) {
    unsafe {
        asm!(
            "wsr.ps {}",
            in(reg) level.0,
            options(nomem, nostack),
        );
    }
}

/// Evaluate the given expression with interrupts disabled.
///
/// This disables interrupts, evaluates the expression, and restores the
/// previous interrupt level. It is safe to use this nested/multiple times.
///
/// This also returns the value of the expression, so you can use it like:
///
/// ```
///     let (p, q) = with_interrupts_disabled! {
///         (canrx!(PUTER_powerCmdP), canrx!(PUTER_powerCmdQ))
///     };
/// ```
///
/// # Do not call from ISR context.
#[macro_export]
macro_rules! with_interrupts_disabled {
    ($stuff:expr) => {{
        let prev = unsafe { crate::interrupts::disable_interrupts() };
        let tmp = $stuff;
        unsafe { crate::interrupts::restore_interrupts(prev) };
        tmp
    }};
}

pub use with_interrupts_disabled;
