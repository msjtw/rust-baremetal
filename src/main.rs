#![no_std]
#![no_main]
#![feature(ascii_char)]

extern crate alloc;
use allocator::Heap;

use alloc::format;
use core::arch::global_asm;
use core::panic::PanicInfo;
use core::ptr::write_volatile;

const HEAPSIZE: usize = 0x10000;

#[global_allocator]
static mut HEAP_ALLOCATOR: Heap = Heap::empty();

static mut HEAP: [u8; HEAPSIZE] = [0; HEAPSIZE];

global_asm!(include_str!("entry.s"));

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
pub extern "C" fn main() {
    uart_print("Hello, world!\n");

    // SAFETY: `HEAP` is only used here and `entry` is only called once.
    unsafe {
        // Give the allocator some memory to allocate.
        HEAP_ALLOCATOR.init(HEAP.as_mut_ptr(), HEAPSIZE);
    }

    // Now we can do things that require heap allocation.
    for ctr in 1..10 {
        let message = format!("Ticks: {}\n", ctr);
        let temp_str = message.as_str();
        uart_print(temp_str);
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    uart_print("Something went wrong.\n");
    loop {}
}
