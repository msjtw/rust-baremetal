use core::{alloc::Layout, ptr::NonNull};

const MIN_ALIGMENT: usize = align_of::<Hole>();

struct Cursor {
    curr: NonNull<Hole>,
    prev: NonNull<Hole>,
}

struct HoleInfo {
    ptr: *mut Hole,
    size: usize,
}

struct Hole {
    size: usize,
    next: Option<NonNull<Hole>>,
}
pub struct LinkedList {
    head: Hole,
}

impl LinkedList {
    pub fn first_fit(&self, layout: Layout) -> *mut u8 {
        let layout = align_to_min(layout).unwrap();

        let mut cursor;
        match self.head.next {
            Some(ptr) => {
                cursor = Cursor {
                    curr: ptr,
                    prev: NonNull::from_ref(&self.head),
                };
            }
            None => {
                panic!("Heap is uninitialzed");
            }
        };

        loop {
            match cursor.split(layout) {
                Ok(x) => {
                    return x;
                }
                Err(_) => {
                    cursor.prev = cursor.curr;
                    match cursor.curr().next {
                        Some(x) => cursor.curr = x,
                        None => {
                            panic!("Out of memory");
                        }
                    };
                }
            }
        }
    }

    pub fn free(&self, ptr: *mut u8, layout: Layout) {
        let layout = align_to_min(layout).unwrap();

        let mut cursor;
        match self.head.next {
            Some(ptr) => {
                cursor = Cursor {
                    curr: ptr,
                    prev: NonNull::from_ref(&self.head),
                };
            }
            None => {
                panic!("Heap is uninitialzed");
            }
        };

        let mut hole = HoleInfo {
            ptr: ptr as *mut Hole,
            size: layout.size(),
        };

        loop {
            match cursor.curr().next {
                Some(next) => {
                    if hole.ptr < next.as_ptr() {
                        unsafe {
                            // found spot for new hole
                            let mut next_ptr = cursor.curr().next;
                            // check if we can merge
                            // with previous
                            match merge(&HoleInfo::from(cursor.prev), &hole) {
                                Some(x) => {
                                    hole = x;
                                }
                                None => {}
                            };
                            // with next
                            match merge(&hole, &HoleInfo::from(next)) {
                                Some(x) => {
                                    next_ptr = next.as_ref().next;
                                    hole = x;
                                }
                                None => {}
                            }

                            cursor.prev.as_mut().next = Some(NonNull::new_unchecked(hole.ptr));
                            hole.ptr.write(Hole {
                                size: hole.size,
                                next: next_ptr,
                            });
                            break;
                        }
                    } else {
                        cursor = cursor.next().unwrap(); // wont panic cause next is some
                    }
                }
                None => unsafe {
                    // hole is after last hole
                    cursor.prev.as_mut().next = Some(NonNull::new_unchecked(hole.ptr));
                    hole.ptr.write(Hole {
                        size: hole.size,
                        next: None,
                    });
                    break;
                },
            }
        }
    }

    pub const fn new() -> Self {
        LinkedList {
            head: Hole {
                size: 0,
                next: None,
            },
        }
    }

    pub fn init(&mut self, addr: *mut u8, size: usize) {
        let hole = Hole {
            size: size,
            next: None,
        };
        let ptr = align_up(addr, align_of::<Hole>()) as *mut Hole;
        unsafe { ptr.write(hole) };
        self.head.next = NonNull::new(ptr);
    }
}

impl Cursor {
    fn next(&self) -> Option<Cursor> {
        match self.curr().next {
            Some(x) => Some(Cursor {
                prev: self.curr,
                curr: x,
            }),
            None => None,
        }
    }

    fn curr(&self) -> &Hole {
        unsafe { self.curr.as_ref() }
    }

    fn prev(&self) -> &Hole {
        unsafe { self.prev.as_ref() }
    }

    fn split(&mut self, layout: Layout) -> Result<*mut u8, ()> {
        let front_padding: Option<HoleInfo>;
        let alloc_ptr: *mut u8;
        let alloc_size: usize;
        let back_padding: Option<HoleInfo>;

        let curr_ptr_u8 = self.curr.as_ptr() as *mut u8;

        //front padding
        let front_off = self.curr.as_ptr().align_offset(layout.align());
        if front_off > 0 {
            // padding
            let front_off = front_off.max(size_of::<Hole>());
            alloc_ptr = align_up(curr_ptr_u8.wrapping_add(front_off), layout.align());
            front_padding = Some(HoleInfo {
                ptr: curr_ptr_u8 as *mut Hole,
                size: alloc_ptr as usize - curr_ptr_u8 as usize,
            })
        } else {
            // no padding
            front_padding = None;
            alloc_ptr = curr_ptr_u8;
        }

        alloc_size = layout.size();

        //back padding
        let alloc_end = alloc_ptr.wrapping_add(alloc_size);
        let hole_end = self.curr.as_ptr().wrapping_add(self.curr().size) as *mut u8;
        if alloc_end > hole_end {
            // not enough space
            return Err(());
        } else if alloc_end == hole_end {
            // no padding
            back_padding = None;
        } else {
            // padding
            let alloc_end = alloc_ptr.wrapping_add(alloc_size);
            let back_padding_size = hole_end as usize - alloc_end as usize;
            if back_padding_size < size_of::<Hole>() {
                // Not enough space left to create a hole
                return Err(());
            } else {
                // Can create a hole
                let back_padding_ptr = alloc_ptr.wrapping_add(alloc_size);
                let back_padding_size = hole_end as usize - back_padding_ptr as usize;
                back_padding = Some(HoleInfo {
                    ptr: back_padding_ptr as *mut Hole,
                    size: back_padding_size,
                })
            }
        }

        match (front_padding, back_padding) {
            (None, None) => unsafe {
                self.prev.as_mut().next = self.curr().next;
            },
            (Some(f), None) => unsafe {
                self.prev.as_mut().next = Some(NonNull::new_unchecked(f.ptr));
                f.ptr.write(Hole {
                    size: f.size,
                    next: self.curr().next,
                });
            },
            (None, Some(b)) => unsafe {
                self.prev.as_mut().next = Some(NonNull::new_unchecked(b.ptr));
                b.ptr.write(Hole {
                    size: b.size,
                    next: self.curr().next,
                });
            },
            (Some(f), Some(b)) => unsafe {
                self.prev.as_mut().next = Some(NonNull::new_unchecked(f.ptr));
                f.ptr.write(Hole {
                    size: f.size,
                    next: Some(NonNull::new_unchecked(b.ptr)),
                });
                b.ptr.write(Hole {
                    size: b.size,
                    next: self.curr().next,
                });
            },
        }

        Ok(alloc_ptr)
    }
}

impl From<NonNull<Hole>> for HoleInfo {
    fn from(value: NonNull<Hole>) -> Self {
        HoleInfo {
            ptr: value.as_ptr(),
            size: unsafe { value.as_ref().size },
        }
    }
}

fn align_up(ptr: *mut u8, align: usize) -> *mut u8 {
    let offset = ptr.align_offset(align);
    ptr.wrapping_add(offset)
}

fn align_to_min(layout: Layout) -> Result<Layout, core::alloc::LayoutError> {
    let res = layout.align_to(MIN_ALIGMENT);
    match res {
        Ok(x) => Ok(x.pad_to_align()),
        Err(x) => Err(x),
    }
}

fn merge(hp: &HoleInfo, hn: &HoleInfo) -> Option<HoleInfo> {
    // if hp is root, we cant merge with it
    if hp.size == 0 {
        return None;
    }
    let hp_end = hp.ptr.wrapping_add(hp.size);
    if hp_end == hn.ptr {
        return Some(HoleInfo {
            ptr: hp.ptr,
            size: hp.size + hn.size,
        });
    }
    None
}

unsafe impl Sync for Hole {}
