#![no_std]
#![no_main]
#![feature(ascii_char)]

mod csr;

extern crate alloc;
use allocator::Heap;

use alloc::format;
use core::arch::{asm, global_asm};
use core::panic::PanicInfo;
use core::ptr::write_volatile;

const HEAPSIZE: usize = 0x10000;

#[global_allocator]
static mut HEAP_ALLOCATOR: Heap = Heap::empty();

static mut HEAP: [u8; HEAPSIZE] = [0; HEAPSIZE];

global_asm!(
    "
    .global _entry
    .extern _STACK_PTR

    .section .text.boot

    _entry:
        la sp, _STACK_PTR
        call main

    spin:
        j spin
    "
);

fn uart_print(message: &str) {
    const UART: *mut u8 = 0x10000000 as *mut u8;

    for c in message.bytes() {
        unsafe {
            write_volatile(UART, c);
        }
    }
}

#[allow(static_mut_refs)] // HEAP.as_mut_ptr() creates a mutable reference to mutable static
#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    uart_print("Hello, world!\n");

    // SAFETY: `HEAP` is only used here and `entry` is only called once.
    unsafe {
        // Give the allocator some memory to allocate.
        HEAP_ALLOCATOR.init(HEAP.as_mut_ptr(), HEAPSIZE);
    }

    let x = read_csr!(misa);

    let message = format!("Misa: {:b}\n", x);
    let temp_str = message.as_str();
    uart_print(temp_str);

    // Now we can do things that require heap allocation.
    // for ctr in 1..10 {
    //     let message = format!("Ticks: {}\n", ctr);
    //     let temp_str = message.as_str();
    //     uart_print(temp_str);
    // }

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    uart_print("Something went wrong.\n");
    loop {}
}
