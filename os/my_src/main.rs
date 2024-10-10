
// my code

#![no_std] // 使用核心库 core
#![no_main] // #![no_main] 告诉编译器我们没有一般意义上的 main 函数
#![feature(panic_info_message)]

mod lang_items;
mod sbi;
use sbi::sbi_call;

const SBI_SHUTDOWN: usize = 8;
fn shutdown() -> ! {
    sbi_call(SBI_SHUTDOWN, [0, 0, 0]);
    panic!("It should shutdown!");
}

/// use sbi call to putchar in console (qemu uart handler)
const SBI_CONSOLE_PUTCHAR: usize = 1;
fn console_putchar(c: usize) {
    sbi_call(SBI_CONSOLE_PUTCHAR, [c, 0, 0]);
}

use core::fmt::{self, Write};
struct Stdout;
impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            console_putchar(c as usize);
        }
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

/// Print! to the host console using the format string and arguments.
#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        print(format_args!($fmt $(, $($arg)+)?))
    }
}

/// Println! to the host console using the format string and arguments.
#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?))
    }
}

// core::arch::global_asm!：这是一个宏，允许将汇编代码嵌入到Rust程序中。
// include_str!("entry.asm")：这是一个函数，它返回一个包含文件 entry.asm 内容的字符串切片。
// "entry.asm" 是你汇编代码文件相对于当前Rust源文件的位置。
core::arch::global_asm!(include_str!("entry.asm"));

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| {
        unsafe {
            (a as *mut u8).write_volatile(0)
        }
    })
}

#[no_mangle]
pub fn rust_main() -> ! {
    println!("Hello, world!");
    print!("Hello, ");
    println!("rCore!");
    clear_bss();
    shutdown();
}

