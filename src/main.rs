// #![no_std] // don't link the Rust standard library
// #![no_main] // disable all Rust-level entry points

// use core::panic::PanicInfo;
// use core::arch::asm;

// const SECTSIZE: usize = 512;

// /// This function is called on panic.
// #[panic_handler]
// fn panic(_info: &PanicInfo) -> ! {
//     loop {}
// }
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

#[inline(always)]
unsafe fn inb(port: u16) -> u8 {
    let data: u8;
    asm!("inb %dx, %al", out("al") data, in("dx") port);
    data
}
#[inline(always)]
unsafe fn insl(port: u16, addr: *mut u8, count: usize) {
    asm!("cld; rep insl", in("dx") port, in("rdi") addr, in("rcx") count);
}
#[inline(always)]
unsafe fn outb(port: u16, data: u8) {
    asm!("outb %al, %dx", in("al") data, in("dx") port);
}
#[inline(always)]
unsafe fn outsl(port: u16, addr: *const u8, count: usize) {
    asm!("cld; rep outsl", in("dx") port, in("rsi") addr, in("rcx") count);
}
#[inline(always)]
unsafe fn stosb(addr: *mut u8, data: i8, count: usize) {
    asm!("cld; rep stosb", in("rdi") addr, in("al") data, in("rcx") count);
}
// 
unsafe fn waitdisk() {
    while (inb(0x1F7) & 0xC0) != 0x40 {}
}
// 
unsafe fn readsect(dst: *mut u8, offset: u32) {
    // Issue command
    waitdisk();
    outb(0x1F2, 1);  // count = 1
    outb(0x1F3, (offset & 0xFF) as u8);
    outb(0x1F4, ((offset >> 8) & 0xFF) as u8);
    outb(0x1F5, ((offset >> 16) & 0xFF) as u8);
    outb(0x1F6, (((offset >> 24) & 0xFF) as u8) | 0xE0);
    outb(0x1F7, 0x20);  // cmd 0x20 - read sectors

    // Read data
    waitdisk();
    insl(0x1F0, dst, SECTSIZE / 4);
}
// 
unsafe fn readseg(pa: *mut u8, count: u32, mut offset: u32) {
    let epa = pa.add(count as usize);

    // Round down to sector boundary
    let pa = pa.offset(-(offset as isize % SECTSIZE as isize));

    // Translate from bytes to sectors; kernel starts at sector 1
    offset = (offset / SECTSIZE as u32) + 1;

    while (pa as *mut u8) < epa {
        readsect(pa, offset);
        offset += 1;
    }
}

const COM1: u16 = 0x3f8;

static mut UART: bool = true;

fn uartputc(c: u8) {
    unsafe {
        if !UART {
            return;
        }
        for _ in 0..128 {
            if inb(COM1 + 5) & 0x20 != 0 {
                break;
            }
        }
        outb(COM1, c);
    }
}

fn uartinit() {
    unsafe {
        // Turn off the FIFO
        outb(COM1 + 2, 0);

        // 9600 baud, 8 data bits, 1 stop bit, parity off.
        outb(COM1 + 3, 0x80); // Unlock divisor
        outb(COM1, (115200 / 9600) as u8);
        outb(COM1 + 1, 0);
        outb(COM1 + 3, 0x03); // Lock divisor, 8 data bits.
        outb(COM1 + 4, 0);
        outb(COM1 + 1, 0x01); // Enable receive interrupts.

        // If status is 0xFF, no serial port.
        if inb(COM1 + 5) == 0xFF {
            UART = false;
        }
    }
}

fn consputc(c: i32) {
    if c == 8 { // BACKSPACE
        uartputc(b'\x08'); // '\b'
        uartputc(b' ');
        uartputc(b'\x08'); // '\b'
    } else {
        uartputc(c as u8);
    }
}

fn printint(xx: i32, base: i32, sign: bool) {
    const DIGITS: &[u8] = b"0123456789abcdef";
    let mut buf = [0u8; 16];
    let mut i = 0;
    let mut x = if sign && xx < 0 { -xx as u32 } else { xx as u32 };

    loop {
        buf[i] = DIGITS[(x % base as u32) as usize];
        i += 1;
        x /= base as u32;
        if x == 0 {
            break;
        }
    }

    if sign && xx < 0 {
        buf[i] = b'-';
        i += 1;
    }

    while i > 0 {
        i -= 1;
        consputc(buf[i] as i32);
    }
}

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    loop {}
}