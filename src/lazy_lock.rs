use crate::once::Once;
use core::cell::UnsafeCell;
use core::fmt::{Debug, Formatter};
use core::mem::ManuallyDrop;
use core::ops::Deref;

// We use the state of a Once as discriminant value. Upon creation, the state is
// "incomplete" and `f` contains the initialization closure. In the first call to
// `call_once`, `f` is taken and run. If it succeeds, `value` is set and the state
// is changed to "complete".
union Data<T, F> {
    value: ManuallyDrop<T>,
    f: ManuallyDrop<F>,
}

pub struct LazyLock<T, F = fn() -> T> {
    once: Once,
    data: UnsafeCell<Data<T, F>>,
}

unsafe impl<T: Sync + Send, F: Send> Sync for LazyLock<T, F> {}

impl<T, F: FnOnce() -> T> LazyLock<T, F> {
    #[inline]
    pub const fn new(f: F) -> Self {
        Self {
            once: Once::new(),
            data: UnsafeCell::new(Data {
                f: ManuallyDrop::new(f),
            }),
        }
    }

    #[inline]
    pub fn force(this: &Self) -> &T {
        this.once.call_once(|| {
            // SAFETY: `call_once` only runs this closure once, ever.
            let data = unsafe { &mut *this.data.get() };
            let f = unsafe { ManuallyDrop::take(&mut data.f) };
            let value = f();
            data.value = ManuallyDrop::new(value);
        });

        unsafe { &(*this.data.get()).value }
    }
}

impl<T, F: FnOnce() -> T> Deref for LazyLock<T, F> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        Self::force(self)
    }
}

impl<T, F> LazyLock<T, F> {
    #[inline]
    pub fn get(this: &Self) -> Option<&T> {
        if this.once.is_completed() {
            Some(unsafe { &(*this.data.get()).value })
        } else {
            None
        }
    }
}

impl<T: Default> Default for LazyLock<T> {
    #[inline]
    fn default() -> Self {
        Self::new(T::default)
    }
}

impl<T: Debug, F> Debug for LazyLock<T, F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut d = f.debug_tuple("LazyLock");
        match Self::get(self) {
            Some(v) => d.field(v),
            None => d.field(&format_args!("<uninit>")),
        };
        d.finish()
    }
}
