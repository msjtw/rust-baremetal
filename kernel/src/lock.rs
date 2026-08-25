use core::ops::{Deref, DerefMut};
use spin::{Mutex, MutexGuard};

use crate::{debug, uart_print};

#[derive(Default)]
pub struct IntMutex<T>(Mutex<T>);

pub struct IntMutexGuard<'a, T> {
    guard: Option<MutexGuard<'a, T>>,
}

impl<T> IntMutex<T> {
    pub const fn new(value: T) -> Self {
        Self(Mutex::new(value))
    }

    pub fn lock(&self) -> IntMutexGuard<'_, T> {
        unsafe { crate::CPU.push_interrupt_off() };

        unsafe { crate::CPU.push_interrupt_off() };

        let guard = self.0.lock();

        // uart_print("MUTEX LOCKED\n");

        IntMutexGuard { guard: Some(guard) }
    }
}

impl<T> Deref for IntMutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.guard.as_ref().unwrap().deref()
    }
}

impl<T> DerefMut for IntMutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.guard.as_mut().unwrap().deref_mut()
    }
}

impl<T> Drop for IntMutexGuard<'_, T> {
    fn drop(&mut self) {
        drop(self.guard.take());

        unsafe { crate::CPU.pop_interrupt_off() };

        // uart_print("MUTEX UNLOCKED\n");
    }
}
