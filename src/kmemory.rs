unsafe extern "C" {
    pub static etext: u8;
    pub static ekernel: u8;
    pub static _STACK_PTR: u32;
}

const PAGESIZE: usize = 4 * 1024;
const RAMSIZE: usize = 62 * 1024 * 1024;
const RAMSTART: usize = 0x80000000;
const RAMEND: usize = RAMSTART + RAMSIZE;

struct PTPage([u32; 1024]); // 4kB page contaning 1024 PTEs

const LEVELS: u32 = 2;
const PTESIZE: u32 = 4;

#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
struct PTE {
    ppn: u32,
    ppn1: u32,
    ppn0: u32,
    rsw: u8,
    d: bool,
    a: bool,
    g: bool,
    u: bool,
    x: bool,
    w: bool,
    r: bool,
    v: bool,
}

impl From<u32> for PTE {
    fn from(pte: u32) -> Self {
        PTE {
            ppn: (pte & 0b11111111111111111111110000000000) >> 10,
            ppn1: (pte & 0b11111111111100000000000000000000) >> 20,
            ppn0: (pte & 0b00000000000011111111110000000000) >> 10,
            rsw: ((pte & 0b00000000000000000000001100000000) >> 8) as u8,
            d: (pte & 0b00000000000000000000000010000000) >= 1,
            a: (pte & 0b00000000000000000000000001000000) >= 1,
            g: (pte & 0b00000000000000000000000000100000) >= 1,
            u: (pte & 0b00000000000000000000000000010000) >= 1,
            x: (pte & 0b00000000000000000000000000001000) >= 1,
            w: (pte & 0b00000000000000000000000000000100) >= 1,
            r: (pte & 0b00000000000000000000000000000010) >= 1,
            v: (pte & 0b00000000000000000000000000000001) >= 1,
        }
    }
}

impl Into<u32> for PTE {
    fn into(self) -> u32 {
        let res = (self.ppn as u32) << 10
            | (self.rsw as u32) << 8
            | (self.d as u32) << 7
            | (self.a as u32) << 6
            | (self.g as u32) << 5
            | (self.u as u32) << 4
            | (self.x as u32) << 3
            | (self.w as u32) << 2
            | (self.r as u32) << 1
            | (self.v as u32);
        res
    }
}

#[allow(dead_code)]
impl PTE {
    fn set_a(self) -> Self {
        Self { a: true, ..self }
    }
    fn set_d(self) -> Self {
        Self { d: true, ..self }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct SATP {
    mode: u8,
    asid: u32,
    ppn: u32,
}

impl From<u32> for SATP {
    fn from(val: u32) -> Self {
        SATP {
            mode: ((val & 0b10000000000000000000000000000000) >> 31) as u8,
            asid: (val & 0b01111111110000000000000000000000) >> 22,
            ppn: (val & 0b00000000001111111111111111111111),
        }
    }
}

#[derive(Debug)]
struct VA {
    vpn1: u32,
    vpn0: u32,
    offset: u32,
}

impl From<u32> for VA {
    fn from(val: u32) -> Self {
        VA {
            vpn1: (val & 0b11111111110000000000000000000000) >> 22,
            vpn0: (val & 0b00000000001111111111000000000000) >> 12,
            offset: val & 0b00000000000000000000111111111111,
        }
    }
}

#[derive(Debug)]
struct PA {
    ppn1: u32,
    ppn0: u32,
    offset: u32,
}

impl Into<u32> for PA {
    fn into(self) -> u32 {
        let ppn1 = self.ppn1 << 22;
        let ppn0 = self.ppn0 << 12;
        ppn1 | ppn0 | self.offset
    }
}

pub struct Kmem {
    freelist: *const u8,
}
impl Kmem {
    pub fn kalloc_init() {
        let kernel_end = unsafe { &ekernel as *const u8 };
        let cursor = align_up(kernel_end as usize, PAGESIZE) as *mut u8;
        while (cursor as usize) < RAMEND {
            cursor.wrapping_add(PAGESIZE);
        }
    }
    pub fn kalloc(&mut self) -> Result<*const u8, ()> {
        // TODO: check if out of memory

        let head = self.freelist;
        self.freelist = unsafe { (self.freelist as *const usize).read() as *mut u8 };
        return Ok(head);
    }
    pub fn kfree(&mut self, ptr: *const u8) {
        // TODO: check if ptr is correct

        unsafe { (ptr as *mut usize).write(self.freelist as usize) };
        self.freelist = ptr;
    }
}

pub struct Kvm {
    pagetree: PTPage,
}

impl Kvm {
    pub fn init(&mut self, memory: &mut Kmem) {
        // map all sections
        self.kvmmap(virt, phys, size, perm);
    }

    // Cretae PTEs for translaition virt -> phys
    // continous virt to virt + size to continous phys to phys + size
    fn kvmmap(&mut self, virt: usize, phys: usize, size: usize, perm: usize) {
        // TODO: tests
        // - size and virt addr aligned on page
        // - size > 0 and end < RAMEND

        let mut addr = virt;
        let addr_end = virt + size;
        while addr < addr_end {
            let pte = walk(self.pagetree, addr);
            match pte {
                Ok(pte) => {}
                Err(_) => {
                    // coudlnt find
                }
            }
            addr += PAGESIZE;
        }
    }
}

// returns leaf pte addr for given virtual address
// with support for megapages
fn walk(memory: &mut Kmem, pagetree: *const u8, virt_a: u32, alloc: bool) -> Result<*mut PTE, ()> {
    let va = VA::from(virt_a);

    let mut a = pagetree as *mut u32;
    let mut i = LEVELS - 1;

    for i in (0..LEVELS).rev() {
        let index = va.vpn1;
        let mut pte_addr = a.with_addr(index as usize);
        let pte_u32 = unsafe { pte_addr.read() };

        let mut pte = PTE::from(pte_u32);

        if pte.v {
            a = (pte.ppn << 12) as *mut u32;
        } else {
            if !alloc {
                return Err(());
            }
            let new_page = memory.kalloc()?;
        }
    }

    // level 1

    if !(pte.r || pte.x) {
        if pte.d || pte.a || pte.u {
            return Err(None);
        }
        //level 0
        i -= 1;
        a = pte.ppn * PAGESIZE;
        let index = va.vpn0 * PTESIZE;
        pte_addr = a + index;
        let pte_m0 = phys_read_word(pte_addr, hart, bus)?;
        pte = PTE::from(pte_m0);
        if !pte.v || (!pte.r && pte.w) {
            // page fault
            // print!(" mmu5 ");
            return Err(None);
        }

        if !(pte.r || pte.x) {
            // level < 0
            // page fault
            // print!(" mmu4 ");
            return Err(None);
        }

        if pte.u {
            //user page
            if (mode == 1 && mstatus_sum == 0) || mode > 1 {
                // print!(" mmu3 ");
                return Err(None);
            }
        } else {
            //supervisor page
            if mode != 1 {
                // print!(" mmu2 ");
                return Err(None);
            }
        }
    }

    // leaf pte has been reached
    if i > 0 && pte.ppn0 != 0 {
        // misaligned superpage
        // print!(" mmu1 ");
        return Err(None);
    }

    let pa = PA {
        ppn1: pte.ppn1,
        ppn0: if i > 0 { va.vpn0 } else { pte.ppn0 },
        offset: va.offset,
    };

    let phys_a: u32 = pa.into();

    let res: (u32, MemoryPermissions);
    if mstatus_mxr > 0 {
        // make eXecutable Readable
        res = (
            phys_a,
            MemoryPermissions {
                r: pte.x,
                w: pte.w,
                x: pte.x,
            },
        );
    } else {
        res = (
            phys_a,
            MemoryPermissions {
                r: pte.r,
                w: pte.w,
                x: pte.x,
            },
        );
    }
}

fn align_up(val: usize, alignment: usize) -> usize {
    let tmp = val + alignment - 1;
    align_down(tmp, alignment)
}

fn align_down(val: usize, alignment: usize) -> usize {
    let rem = val % alignment;
    val - rem
}
