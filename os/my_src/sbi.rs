//! SBI call wrappers

/*
use core::arch::asm;

const SBI_CONSOLE_PUTCHAR: usize = 1;

/// general sbi call
#[inline(always)]
fn sbi_call(which: usize, arg0: usize, arg1: usize, arg2: usize) -> usize {
    let mut ret;
    unsafe {
        asm!(
            "li x16, 0",
            "ecall",
            inlateout("x10") arg0 => ret,
            in("x11") arg1,
            in("x12") arg2,
            in("x17") which,
        );
    }
    ret
}

/// use sbi call to putchar in console (qemu uart handler)
pub fn console_putchar(c: usize) {
    sbi_call(SBI_CONSOLE_PUTCHAR, c, 0, 0);
}

use crate::board::QEMUExit;
/// use sbi call to shutdown the kernel
pub fn shutdown() -> ! {
    crate::board::QEMU_EXIT_HANDLE.exit_failure();
}
*/

// my code

/// general sbi call
#[inline(always)]
pub fn sbi_call(which: usize, args: [usize; 3]) -> usize {
    let mut ret;
    unsafe {
        core::arch::asm!(
            "ecall",
            inlateout("x10") args[0] => ret,
            in("x11") args[1],
            in("x12") args[2],
            in("x17") which,
        );
    }
    ret
}

