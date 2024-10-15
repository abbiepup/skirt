use crate::sync::atomic::{AtomicBool, Ordering};
use core::cell::UnsafeCell;
use core::fmt::{Debug, Display, Formatter};
use core::ops::{Deref, DerefMut};

/// A mutual exclusion primitive useful for protecting shared data.
///
/// This mutex will block thread waiting for the lock to become available.
/// In a `no_std` environment, the mutex employs a spin-lock mechanism, continiously checking for availability.
/// In a `std` environment, the mutex will yield the thread.
/// The mutex can be created via a [`new`] constructor.
/// Each mutex has a type parameter which represents the data that it is protecting.
/// The data can only be accessed through the RAII guards returned from [`lock`] and [`try_lock`],
/// which guarantees that the data is only ever accessed when the mutex is locked.
///
/// [`new`]: Self::new
/// [`lock`]: Self::lock
/// [`try_lock`]: Self::try_lock
///
/// # Examples
pub struct Mutex<T: ?Sized> {
    lock: AtomicBool,
    data: UnsafeCell<T>,
}

unsafe impl<T: ?Sized + Send> Send for Mutex<T> {}
unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    /// Creates a new mutex in an unlocked state ready for use.
    ///
    /// # Examples
    /// ```
    /// use skirt::sync::Mutex;
    ///
    /// let mutex = Mutex::new(0);
    /// ```
    #[inline]
    pub const fn new(data: T) -> Self {
        Self {
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    /// Returns the contained value by cloning it.
    ///
    /// # Examples
    /// ```
    /// use skirt::sync::Mutex;
    ///
    /// let mut mutex = Mutex::new(7);
    ///
    /// assert_eq!(mutex.get_cloned(), 7);
    /// ```
    pub fn get_cloned(&self) -> T
    where
        T: Clone,
    {
        (*self.lock()).clone()
    }

    /// Replaces the contained value with `data`, and returns the old contained value.
    ///
    /// # Examples
    /// ```
    /// use skirt::sync::Mutex;
    ///
    /// let mut mutex = Mutex::new(7);
    ///
    /// assert_eq!(mutex.replace(11), 7);
    /// assert_eq!(mutex.get_cloned(), 11);
    /// ```
    pub fn replace(&self, data: T) -> T {
        core::mem::replace(&mut *self.lock(), data)
    }
}

impl<T: ?Sized> Mutex<T> {
    /// Acquires a mutex, blocking the current thread until it is able to do so.
    ///
    /// # Examples
    /// ```
    /// use skirt::sync::Mutex;
    /// use std::sync::Arc;
    /// use std::thread;
    ///
    /// let mutex = Arc::new(Mutex::new(0));
    /// let c_mutex = Arc::clone(&mutex);
    ///
    /// thread::spawn(move || {
    ///     *c_mutex.lock() = 10;
    /// }).join().expect("thread::spawn failed");
    /// assert_eq!(*mutex.lock(), 10);
    /// ```
    pub fn lock(&self) -> MutexGuard<'_, T> {
        #[cfg(feature = "std")]
        let mut tries = 0;

        while self.try_lock().is_none() {
            core::hint::spin_loop();

            #[cfg(feature = "std")]
            match tries >= 10 {
                true => std::thread::yield_now(),
                false => tries += 1,
            }
        }

        MutexGuard::new(self)
    }

    pub fn lock_weak(&self) -> MutexGuard<'_, T> {
        #[cfg(feature = "std")]
        let mut tries = 0;

        while self
            .lock
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();

            #[cfg(feature = "std")]
            match tries >= 10 {
                true => std::thread::yield_now(),
                false => tries += 1,
            }
        }

        MutexGuard::new(self)
    }

    /// Attempts to acquire this lock.
    ///
    /// If the lock could not be acquired at this time, then [`None`] is returned.
    /// Otherwise, an RAII guard is returned. The lock will be unlocked when the
    /// guard is dropped.
    ///
    /// This function does not block.
    ///
    /// # Example
    /// ```
    /// use skirt::sync::Mutex;
    /// use std::sync::Arc;
    /// use std::thread;
    ///
    /// let mutex = Arc::new(Mutex::new(0));
    /// let c_mutex = Arc::clone(&mutex);
    ///     
    /// thread::spawn(move || {
    ///     let mut lock = c_mutex.try_lock();
    ///     if let Some(ref mut mutex) = lock {
    ///         **mutex = 10;
    ///     } else {
    ///         println!("try_lock failed");
    ///     }
    /// }).join().expect("thread::spawn failed");
    /// assert_eq!(*mutex.lock(), 10);
    /// ```
    #[must_use]
    pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
        self.lock
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
            .then(|| MutexGuard::new(self))
    }

    /// Consumes this mutex, returning the underlying data.
    ///
    /// # Examples
    /// ```
    /// use skirt::sync::Mutex;
    ///
    /// let mutex = Mutex::new(0);
    /// assert_eq!(mutex.into_inner(), 0);
    /// ```
    pub fn into_inner(self) -> T
    where
        T: Sized,
    {
        self.data.into_inner()
    }

    /// Returns a mutable reference to the underlying data.
    ///
    /// Since this call borrows the `Mutex` mutably, no actual locking needs to
    /// take place -- the mutable borrow statically guarantees no locks exist.
    ///
    /// # Examples
    /// ```
    /// use skirt::sync::Mutex;
    ///
    /// let mut mutex = Mutex::new(0);
    /// *mutex.get_mut() = 10;
    /// assert_eq!(*mutex.lock(), 10);
    /// ```
    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.data.get() }
    }
}

impl<T> From<T> for Mutex<T> {
    /// Creates a new mutex in an unlocked state ready for use.
    /// This is equivalent to [`Mutex::new`].
    fn from(data: T) -> Self {
        Self::new(data)
    }
}

impl<T: Default> Default for Mutex<T> {
    /// Creates a `Mutex<T>`, with the `Default` value for T.
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: ?Sized + Debug> Debug for Mutex<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut d = f.debug_struct("Mutex");

        match self.try_lock() {
            Some(guard) => d.field("data", &&*guard),
            None => d.field("data", &format_args!("<locked>")),
        };

        d.finish_non_exhaustive()
    }
}

/// An RAII implementation of a "scoped lock" of a mutex. When this structure is
/// dropped (falls out of scope), the lock will be unlocked.
///
/// The data protected by the mutex can be accessed through this guard via its
/// [`Deref`] and [`DerefMut`] implementations.
///
/// This structure is created by the [`lock`] and [`try_lock`] methods on
/// [`Mutex`].
///
/// [`lock`]: Mutex::lock
/// [`try_lock`]: Mutex::try_lock
pub struct MutexGuard<'m, T: ?Sized> {
    mutex: &'m Mutex<T>,
    #[cfg(not(feature = "nightly"))]
    phantom: core::marker::PhantomData<*const ()>,
}

#[cfg(feature = "nightly")]
impl<T: ?Sized> !Send for MutexGuard<'_, T> {}
unsafe impl<T: ?Sized + Sync> Sync for MutexGuard<'_, T> {}

impl<'m, T: ?Sized> MutexGuard<'m, T> {
    const fn new(mutex: &'m Mutex<T>) -> Self {
        Self {
            mutex,
            #[cfg(not(feature = "nightly"))]
            phantom: core::marker::PhantomData,
        }
    }
}

impl<T: ?Sized> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: The lock is held, giving us exclusive access to the data.
        unsafe { &*self.mutex.data.get() }
    }
}

impl<T: ?Sized> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: The lock is held, giving us exclusive access to the data.
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<T: ?Sized> Drop for MutexGuard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        self.mutex.lock.store(false, Ordering::Release);
    }
}

impl<T: ?Sized + Debug> Debug for MutexGuard<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        (**self).fmt(f)
    }
}

impl<T: ?Sized + Display> Display for MutexGuard<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        (**self).fmt(f)
    }
}

#[cfg(feature = "lock_api")]
unsafe impl lock_api::RawMutex for Mutex<()> {
    const INIT: Self = Self::new(());

    type GuardMarker = lock_api::GuardSend;

    fn lock(&self) {
        core::mem::forget(self.lock());
    }

    fn try_lock(&self) -> bool {
        self.try_lock().map(core::mem::forget).is_some()
    }

    unsafe fn unlock(&self) {
        self.lock.store(false, Ordering::Release);
    }
}
