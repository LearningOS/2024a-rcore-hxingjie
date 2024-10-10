

// my code

use core::panic::PanicInfo;
use crate::shutdown;
use crate::print;
use crate::println;

// #[panic_handler] 是一种编译指导属性，用于标记核心库core中的 panic! 宏要对接的函数（该函数实现对致命错误的具体处理）。
// 该编译指导属性所标记的函数需要具有 fn(&PanicInfo) -> ! 函数签名，函数可通过 PanicInfo 数据结构获取致命错误的相关信息。
// 这样Rust编译器就可以把核心库core中的 panic! 宏定义与 #[panic_handler] 指向的panic函数实现合并在一起，
// 使得no_std程序具有类似std库的应对致命错误的功能。

// 实现 panic_handler 函数
#[panic_handler]
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
    shutdown();
}

