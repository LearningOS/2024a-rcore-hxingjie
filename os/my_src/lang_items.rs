//! The panic handler

/*
use core::panic::PanicInfo;
use crate::sbi::shutdown;

#[panic_handler]
/// panic handler
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!(
            "[kernel] Panicked at {}:{} {}",
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        println!("[kernel] Panicked: {}", info.message().unwrap());
    }
    shutdown()
}
*/

// my code

use core::panic::PanicInfo;

// 实现 panic_handler 函数
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

