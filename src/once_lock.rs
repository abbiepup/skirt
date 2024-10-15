use crate::once::Once;
use core::cell::UnsafeCell;
use core::fmt::{Debug, Formatter};
use core::mem::MaybeUninit;

pub struct OnceLock<T> {
    once: Once,
    data: UnsafeCell<MaybeUninit<T>>,
}

unsafe impl<T: Sync + Send> Sync for OnceLock<T> {}
unsafe impl<T: Send> Send for OnceLock<T> {}

impl<T> OnceLock<T> {
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self {
            once: Once::new(),
            data: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    #[inline]
    pub fn get(&self) -> Option<&T> {
        if self.is_initialized() {
            Some(unsafe { self.get_unchecked() })
        } else {
            None
        }
    }

    #[inline]
    pub fn get_mut(&mut self) -> Option<&mut T> {
        if self.is_initialized() {
            Some(unsafe { self.get_unchecked_mut() })
        } else {
            None
        }
    }

    #[inline]
    pub fn get_or_init<F>(&self, f: F) -> &T
    where
        F: FnOnce() -> T,
    {
        if !self.is_initialized() {
            self.initialize(f);
        }

        unsafe { self.get_unchecked() }
    }

    #[inline]
    ///
    /// # Errors
    pub fn set(&self, data: T) -> Result<(), T> {
        if self.is_initialized() {
            return Err(data);
        }

        self.initialize(|| data);

        Ok(())
    }

    #[inline]
    pub fn into_inner(mut self) -> Option<T> {
        self.take()
    }

    #[inline]
    pub fn take(&mut self) -> Option<T> {
        if !self.is_initialized() {
            return None;
        }

        self.once = Once::new();

        // SAFETY: todo!()
        unsafe { Some((*self.data.get()).assume_init_read()) }
    }

    #[inline]
    fn is_initialized(&self) -> bool {
        self.once.is_completed()
    }

    #[cold]
    fn initialize<F>(&self, f: F)
    where
        F: FnOnce() -> T,
    {
        let slot = &self.data;
        self.once.call_once(|| unsafe {
            (*slot.get()).write(f());
        });
    }

    #[inline]
    unsafe fn get_unchecked(&self) -> &T {
        debug_assert!(self.is_initialized());
        unsafe { (*self.data.get()).assume_init_ref() }
    }

    #[inline]
    unsafe fn get_unchecked_mut(&mut self) -> &mut T {
        debug_assert!(self.is_initialized());
        unsafe { (*self.data.get()).assume_init_mut() }
    }
}

impl<T> Default for OnceLock<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T> From<T> for OnceLock<T> {
    #[inline]
    fn from(value: T) -> Self {
        let cell = Self::new();

        match cell.set(value) {
            Ok(()) => cell,
            Err(_) => unreachable!(),
        }
    }
}

impl<T: PartialEq> PartialEq for OnceLock<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get()
    }
}

impl<T: Eq> Eq for OnceLock<T> {}

impl<T> Drop for OnceLock<T> {
    #[inline]
    fn drop(&mut self) {
        if self.is_initialized() {
            unsafe { (*self.data.get()).assume_init_drop() };
        }
    }
}

impl<T: Clone> Clone for OnceLock<T> {
    #[inline]
    fn clone(&self) -> Self {
        let cell = Self::new();

        if let Some(data) = self.get() {
            match cell.set(data.clone()) {
                Ok(()) => (),
                Err(_) => unreachable!(),
            }
        }

        cell
    }
}

impl<T: Debug> Debug for OnceLock<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut d = f.debug_tuple("OnceLock");

        match self.get() {
            Some(v) => d.field(v),
            None => d.field(&format_args!("<uninit>")),
        };

        d.finish()
    }
}
