use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
};

// Shitty mutex

pub struct Mutex<T: ?Sized> {
    data: UnsafeCell<T>,
}

/// A guard to which the protected data can be accessed
///
/// When the guard falls out of scope it will release the lock.
#[derive(Debug)]
pub struct MutexGuard<'a, T: ?Sized + 'a> {
    data: &'a mut T,
}

// Same unsafe impls as `std::sync::Mutex`
unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}
unsafe impl<T: ?Sized + Send> Send for Mutex<T> {}

impl<T> Deref for Mutex<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &mut *self.data.get() }
    }
}
impl<T> DerefMut for Mutex<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.data.get() }
    }
}
impl<T> Mutex<T> {
    pub const fn new(data: T) -> Mutex<T> {
        Mutex {
            data: UnsafeCell::new(data),
        }
    }
    pub fn get(&self) -> &mut T {
        unsafe { &mut *self.data.get() }
    }
}
