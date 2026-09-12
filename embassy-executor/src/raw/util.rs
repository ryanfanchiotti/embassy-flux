use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::ptr;

pub(crate) struct UninitCell<T>(MaybeUninit<UnsafeCell<T>>);
impl<T> UninitCell<T> {
    pub const fn uninit() -> Self {
        Self(MaybeUninit::uninit())
    }

    // I think this is a property of UnsafeCell. Maybe can express it in specs for core?
    // We don't need to show that the data is initted. In fact it probably isn't mostly,
    // and that's not UB
    #[flux::spec(fn (&UninitCell<T>) ->
        *mut{p: p.addr % T::align_of() == 0 &&
                (T::size_of() == 0 ||
                    (p.addr != 0 && p.addr >= p.base &&
                        T::size_of() <= p.size && p.size >= 0))} T)]
    #[flux::trusted]
    pub unsafe fn as_mut_ptr(&self) -> *mut T {
        (*self.0.as_ptr()).get()
    }

    #[allow(clippy::mut_from_ref)]
    pub unsafe fn as_mut(&self) -> &mut T {
        &mut *self.as_mut_ptr()
    }

    #[inline(never)]
    pub unsafe fn write_in_place(&self, func: impl FnOnce() -> T) {
        ptr::write(self.as_mut_ptr(), func())
    }

    #[flux::spec(fn (me: &UninitCell<T>))]
    pub unsafe fn drop_in_place(&self) {
        ptr::drop_in_place(self.as_mut_ptr())
    }
}

unsafe impl<T> Sync for UninitCell<T> {}

#[repr(transparent)]
pub struct SyncUnsafeCell<T> {
    value: UnsafeCell<T>,
}

unsafe impl<T: Sync> Sync for SyncUnsafeCell<T> {}

impl<T> SyncUnsafeCell<T> {
    #[inline]
    pub const fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
        }
    }

    pub unsafe fn set(&self, value: T) {
        *self.value.get() = value;
    }

    pub unsafe fn get(&self) -> T
    where
        T: Copy,
    {
        *self.value.get()
    }
}
