// cargo rustc -- -C link-arg=-nostartfiles
// cargo rustc -- -C link-arg=/ENTRY:_start
// cargo rustc -- -C link-args="/ENTRY:_start /SUBSYSTEM:console"

#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

use core::panic::PanicInfo;
use core::arch::asm;

const SECTSIZE: usize = 512;

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}