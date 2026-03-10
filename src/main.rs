#![no_std]
#![no_main]
#![feature(ascii_char)]

mod csr;
mod kmemory;

extern crate alloc;
use alloc::format;
use buddy_system_allocator::LockedHeap;

use core::arch::global_asm;
use core::panic::PanicInfo;
use core::ptr::write_volatile;

const HEAPSIZE: usize = 0x10000;

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap<32> = LockedHeap::<32>::new();

static mut HEAP: [u8; HEAPSIZE] = [0; HEAPSIZE];

// #[unsafe(no_mangle)]
// unsafe extern  static STACK: [u8; 4096] = [0; 4096];

global_asm!(
    "
    .global _entry
    .extern _STACK_PTR
    .extern stack

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

struct Kernel {
    memory: kmemory::Kmem,
    kvm: kmemory::Kvm,

}

#[allow(static_mut_refs)]
#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    uart_print("Hello, world!\n");

    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(HEAP.as_mut_ptr() as usize, HEAPSIZE);
    }

    let msg = unsafe {
        format!(
            "etext: 0x{:x}, ekernel: 0x{:x}, _STACK_PTR: 0x{:x}\n",
            &kmemory::etext as *const u32 as u32,
            &kmemory::ekernel as *const u32 as u32,
            &kmemory::_STACK_PTR as *const u32 as u32
        )
    };
    let tmp = msg.as_str();
    uart_print(tmp);
    
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    uart_print("Something went wrong.\n");
    loop {}
}
