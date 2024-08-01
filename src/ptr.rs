//! Module that attemps to define a `Unique<T>` pointer:
//!
//!  - non-null, well-aligned & read-write dereferenceable,
//!    (for some _valid_ instance; wrap the `T` in a `MaybeUninit` to loosen
//!    that validity invariant)
//!  - unaliased,
//!  - covariant.
//!
//! When `Box` is available, `MD<Box<T>>` happens to satisfy all these
//! requirements, but for the dereferenceable one.
//!
//! Otherwise, `ptr::NonNull<T>` is the maximal covariant wrapper available
//! to stable Rust… 😩

#[allow(unused_imports)]
pub(crate) use ::core::ptr::*;

pub(crate) use __::Unique;

#[cfg(feature = "alloc")]
pub(crate) mod __ {
    use ::alloc::boxed::Box;
    use ::core::{mem::ManuallyDrop as MD, ptr};

    #[repr(transparent)]
    pub(crate) struct Unique<T: ?Sized>(MD<Box<T>>);

    impl<T: ?Sized> Unique<T> {
        #[inline]
        pub(crate) unsafe fn from_raw(ptr: *mut T) -> Unique<T> {
            Self(MD::new(Box::from_raw(ptr)))
        }

        #[inline]
        pub(crate) fn into_raw_nonnull(self: Unique<T>) -> ptr::NonNull<T> {
            Box::leak(MD::into_inner(self.0)).into()
        }

        #[inline]
        pub(crate) unsafe fn drop_in_place(this: &'_ mut Unique<T>) {
            ptr::drop_in_place::<T>(&mut **this)
        }
    }

    impl<T: ?Sized> ::core::ops::Deref for Unique<T> {
        type Target = T;

        #[inline]
        fn deref(self: &'_ Unique<T>) -> &'_ T {
            &**self.0
        }
    }

    impl<T: ?Sized> ::core::ops::DerefMut for Unique<T> {
        #[inline]
        fn deref_mut(self: &'_ mut Unique<T>) -> &'_ mut T {
            &mut **self.0
        }
    }
}

#[cfg(not(feature = "alloc"))]
pub(crate) mod __ {
    use ::core::ptr;

    #[repr(transparent)]
    pub(crate) struct Unique<T: ?Sized>(ptr::NonNull<T>);

    unsafe impl<T: Send + ?Sized> Send for Unique<T> {}
    unsafe impl<T: Sync + ?Sized> Sync for Unique<T> {}

    impl<T: ?Sized> Unique<T> {
        #[inline]
        pub(crate) unsafe fn from_raw(ptr: *mut T) -> Unique<T> {
            Self(ptr::NonNull::new_unchecked(ptr))
        }

        #[inline]
        pub(crate) fn into_raw_nonnull(self: Unique<T>) -> ptr::NonNull<T> {
            self.0
        }

        #[inline]
        pub(crate) unsafe fn drop_in_place(this: &'_ mut Unique<T>) {
            ptr::drop_in_place(this.0.as_ptr())
        }
    }

    impl<T: ?Sized> ::core::ops::Deref for Unique<T> {
        type Target = T;

        #[inline]
        fn deref(self: &'_ Unique<T>) -> &'_ T {
            unsafe { self.0.as_ref() }
        }
    }

    impl<T: ?Sized> ::core::ops::DerefMut for Unique<T> {
        #[inline]
        fn deref_mut(self: &'_ mut Unique<T>) -> &'_ mut T {
            unsafe { self.0.as_mut() }
        }
    }
}
