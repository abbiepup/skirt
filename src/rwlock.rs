use crate::sync::atomic::AtomicUsize;
use core::cell::UnsafeCell;
use core::fmt::{Debug, Display, Formatter};
use core::ops::{Deref, DerefMut};

const MASK: u8 = (1 << 6) - 1;
const READ_LOCKED: u8 = 1;
const WRITE_LOCKED: u8 = MASK;
const READERS_WAITING: u8 = 1 << 5;
const WRITERS_WAITING: u8 = 1 << 6;
const MAX_READERS: u8 = MASK - 1;
const DOWNGRADE: u8 = READ_LOCKED.wrapping_sub(WRITE_LOCKED);

/// A reader-writer lock.
///
/// This type of lock allows a number of readers or at most one writer at any
/// point in time. The write portion of this lock typically allows modification
/// of the underlying data (exclusive access) and the read portion of this lock
/// typically allows for read-only access (shared access).
pub struct RwLock<T: ?Sized> {
    lock: AtomicUsize,
    data: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Send> Send for RwLock<T> {}
unsafe impl<T: ?Sized + Send + Sync> Sync for RwLock<T> {}

impl<T> RwLock<T> {
    #[inline]
    pub const fn new(data: T) -> Self {
        Self {
            lock: AtomicUsize::new(0),
            data: UnsafeCell::new(data),
        }
    }
}

impl<T: ?Sized> RwLock<T> {
    pub fn read(&self) {}

    pub fn try_read(&self) {}

    pub fn write(&self) {}

    pub fn try_write(&self) {}
}

impl<T> From<T> for RwLock<T> {
    fn from(data: T) -> Self {
        Self::new(data)
    }
}

impl<T: Default> Default for RwLock<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: ?Sized + Debug> Debug for RwLock<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        todo!()
    }
}

// pub struct RwLockReadGuard<'r, T: ?Sized + 'r> {
//     data: NonNull<T>,
// }

/// RAII structure used to release the exclusive write access of a lock when
/// dropped.
///
/// This structure is created by the [`write`] and [`try_write`] methods
/// on [`RwLock`].
///
/// [`write`]: RwLock::write
/// [`try_write`]: RwLock::try_write
pub struct RwLockWriteGuard<'rw, T: ?Sized + 'rw> {
    lock: &'rw RwLock<T>,
    #[cfg(not(feature = "nightly"))]
    phantom: core::marker::PhantomData<*const ()>,
}

#[cfg(feature = "nightly")]
impl<T: ?Sized> !Send for RwLockWriteGuard<'_, T> {}
unsafe impl<T: ?Sized + Sync> Sync for RwLockWriteGuard<'_, T> {}

impl<'rw, T: ?Sized> RwLockWriteGuard<'rw, T> {
    const fn new(lock: &'rw RwLock<T>) -> Self {
        Self {
            lock,
            #[cfg(not(feature = "nightly"))]
            phantom: core::marker::PhantomData,
        }
    }
}

impl<T: ?Sized> Deref for RwLockWriteGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T: ?Sized> DerefMut for RwLockWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T: ?Sized> Drop for RwLockWriteGuard<'_, T> {
    fn drop(&mut self) {
        todo!()
    }
}

impl<T: ?Sized + Debug> Debug for RwLockWriteGuard<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        (**self).fmt(f)
    }
}

impl<T: ?Sized + Display> Display for RwLockWriteGuard<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        (**self).fmt(f)
    }
}
