mod linked_list;

use core::alloc::{GlobalAlloc, Layout};

use linked_list::*;

pub struct Heap {
    free_list: LinkedList,
}

impl Heap {
    pub const  fn empty() -> Self {
        Heap {
            free_list: LinkedList::new(),
        }
    }

    pub unsafe fn init(&mut self, addr: *mut u8, size: usize) {
        self.free_list.init(addr, size);
    }
}

unsafe impl GlobalAlloc for Heap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.free_list.first_fit(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        self.free_list.free(ptr, layout);
    }
}
