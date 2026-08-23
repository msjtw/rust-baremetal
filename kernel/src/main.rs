#![no_std]
#![no_main]
#![feature(allocator_api)]
#![allow(static_mut_refs)]

pub mod allocator;
mod csr;
mod kernel;
pub mod lock;
mod process;
mod trap;
pub mod virtmemory;

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use spin::Once;

use core::arch::{asm, global_asm};
use core::panic::PanicInfo;
use core::ptr::write_volatile;

use crate::kernel::{Cpu, Kernel};
use crate::trap::init_trap;
use crate::trap::trampoline::{userret, uservec};
use crate::virtmemory::RAMEND;

const PRIME: &[u8] = include_bytes!("../../user/_prime.bin");
const INIT: &[u8] = include_bytes!("../../user/_init.bin");

#[global_allocator]
static HEAP_ALLOCATOR: allocator::LockedHeap<32> = allocator::LockedHeap::<32>::new();

static FRAME_ALLOCATOR: allocator::FrameAllocator = allocator::FrameAllocator {};

static mut CPU: Cpu = Cpu::new();
static KERNEL: Once<lock::IntMutex<Kernel>> = Once::new();

global_asm!(
    "
    .global _entry
    .extern _STACK_PTR
    .extern stack

    .section .text.boot

    _entry:
        la sp, _STACK_PTR
        call main

    park:
        j park
    "
);

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        $crate::uart_print(&alloc::format!($($arg)*));
    }};
}

#[macro_export]
macro_rules! println {
    () => {{
        $crate::uart_print("\n");
    }};
    ($($arg:tt)*) => {{
        $crate::uart_print(&alloc::format!("{}\n", alloc::format!($($arg)*)));
    }};
}

pub const DEBUG: bool = false;

#[macro_export]
macro_rules! debug {
    () => {{
        if $crate::DEBUG {
            $crate::uart_print("\n");
        }
    }};
    ($($arg:tt)*) => {{
        if $crate::DEBUG {
            $crate::uart_print(&alloc::format!("{}\n", alloc::format!($($arg)*)));
        }
    }};
}

pub fn uart_print(message: &str) {
    let uart = virtmemory::UART as *mut u8;
    for c in message.bytes() {
        unsafe {
            write_volatile(uart, c);
        }
    }
}

// FIX: Stack guard pages don't work,
// stack-overflow causes infinite trapping.

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    // NOTE: without this they are optimized away
    let _ = uservec as *const () as usize;
    let _ = userret as *const () as usize;

    // TODO: How to implement memory so all accesses don't have to be unsafe.
    //       Can I map a slice [u8] over whole available ram?

    // Init physical memory allocator.
    unsafe {
        let ekernel = &virtmemory::ekernel as *const usize as usize;
        HEAP_ALLOCATOR
            .lock()
            .init(ekernel, RAMEND as usize - ekernel);
    }


    init_trap();
    KERNEL.call_once(|| lock::IntMutex::new(Kernel::default()));
    {
        let mut kernel = KERNEL.get().unwrap().lock();

        debug!("Hello world\n");

        kernel.init().expect("Kernel init fail");

        kernel.initproc(4).unwrap();
        kernel
            .kvm
            .as_mut()
            .expect("KVM not initialized")
            .start_kvm();
        debug!("Virt started\n");

        // Start init
        let user_p0 = kernel.allocproc().unwrap();
        user_p0.kexec(String::from("init"), vec!["10"]).unwrap();
        user_p0.state = process::ProcState::Runnable;
    }

    process::scheduler();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print!("Something went wrong. {:?}\n", info);
    loop {}
}
