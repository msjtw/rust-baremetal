use core::ops::{Deref, DerefMut};
use spin::{Mutex, MutexGuard};

// Wrapper around spin::mutex that disables interrupts
#[derive(Default)]
pub struct IntMutex<T>(Mutex<T>);

pub struct IntMutexGuard<'a, T>(Option<MutexGuard<'a, T>>);

impl<T> IntMutex<T> {
    pub const fn new(value: T) -> Self {
        Self(Mutex::new(value))
    }

    pub fn lock(&self) -> IntMutexGuard<'_, T> {
        // FIX: Disable interrupts before taking the spin lock so trap handlers cannot
        // re-enter the same lock on this CPU.
        unsafe { crate::CPU.push_interrupt_off() };
        IntMutexGuard(Some(self.0.lock()))
    }
}

impl<T> Deref for IntMutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.0.as_ref().expect("interrupt mutex guard missing")
    }
}

impl<T> DerefMut for IntMutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.0.as_mut().expect("interrupt mutex guard missing")
    }
}

impl<T> Drop for IntMutexGuard<'_, T> {
    fn drop(&mut self) {
        // FIX: Drop the spin guard first, then restore interrupt state.
        // This closes the window where interrupts were re-enabled while lock was still held.
        self.0.take();
        unsafe { crate::CPU.pop_interrupt_off() };
    }
}
