unsafe extern "C" {
    pub static etext: u8;
    pub static ekernel: u8;
    pub static _STACK_PTR: u32;
}

const PAGESIZE: usize = 4 * 1024;
const RAMSIZE: usize = 62 * 1024 * 1024;
const RAMSTART: usize = 0x80000000;
const RAMEND: usize = RAMSTART + RAMSIZE;

struct Kmem {
    freelist: *const u8
}

fn kalloc_init() {
    let cursor = unsafe { &ekernel as *const u8 as *mut u8 };
    // TODO: align cursot to pagesize
    while (cursor as usize) < RAMEND {

        cursor.wrapping_add(PAGESIZE);
    }
}
fn kalloc() {}
fn kfree() {}

