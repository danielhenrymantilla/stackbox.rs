use super::*;

pub
mod iter;

impl<'frame, Item : 'frame> StackBox<'frame, [Item]> {
    /// # Safety
    ///
    /// Same requirements as [`StackBox::assume_owns`].
    #[inline]
    unsafe
    fn assume_owns_all (
        slice: &'frame mut [ManuallyDrop<Item>]
    ) -> StackBox<'frame, [Item]>
    {
        let fat_ptr: *mut [ManuallyDrop<Item>] = slice;
        let fat_ptr: *mut ManuallyDrop<[Item]> = fat_ptr as _;
        let slice: &'frame mut ManuallyDrop<[Item]> = &mut *fat_ptr;
        StackBox::assume_owns(slice)
    }

    /// [`Vec`]-like behavior for [`StackBox`]: pop its first item.
    pub
    fn stackbox_pop (self: &'_ mut StackBox<'frame, [Item]>)
      -> Option<Item>
    {
        if self.is_empty() {
            return None;
        }
        let placeholder = unsafe {
            // Safety: empty slice.
            StackBox::assume_owns_all(&mut [])
        };
        let this = ::core::mem::replace(self, placeholder);
        let (hd, tl) = this.stackbox_split_at(1);
        *self = tl;
        Some(unsafe {
            ::core::ptr::read(&ManuallyDrop::new(hd)[0])
        })
    }

    /// [`StackBox`] / owned equivalent of the `slice` splitting methods.
    #[inline]
    pub
    fn stackbox_split_at (self: StackBox<'frame, [Item]>, mid: usize)
      -> (
            StackBox<'frame, [Item]>,
            StackBox<'frame, [Item]>,
        )
    {
        assert!(mid <= self.len()); // before the MD
        let mut this = ::core::mem::ManuallyDrop::new(self);
        let (hd, tl): (&'_ mut [Item], &'_ mut [Item]) =
            this.split_at_mut(mid)
        ;
        unsafe {
            // Safety: recovering back the ownership initially yielded.
            (
                Self::assume_owns_all(
                    ::core::slice::from_raw_parts_mut(
                        hd.as_mut_ptr().cast(),
                        hd.len(),
                    )
                ),
                Self::assume_owns_all(
                    ::core::slice::from_raw_parts_mut(
                        tl.as_mut_ptr().cast(),
                        tl.len(),
                    )
                ),
            )
        }
    }
}

pub
trait IsArray<'frame> : 'frame {
    type Item : 'frame;

    fn into_slice (this: StackBox<'frame, Self>)
      -> StackBox<'frame, [Self::Item]>
    ;
}

/// `Array = [Array::Item; N]`.
impl<'frame, Array : IsArray<'frame>> StackBox<'frame, Array> {
    /// Coerces a `StackBox<[T; N]>` into a `StackBox<[T]>`.
    ///
    /// ### Requirements
    ///
    ///   - Either the `"const-generics"` feature needs to be enabled,
    ///
    ///   - Or `N` must be one of the hard-coded ones:
    ///
    ///       - a power of `2` up to `4096`;
    ///
    ///       - some other psychological numbers
    ///         (some multiples of 25, 50 or 100).
    ///
    ///   - Note that you may not need to use `.into_slice()` if instead of
    ///     [`StackBox::new_in`] you use [`stackbox!`] to construct it:
    ///
    ///     ```rust
    ///     use ::stackbox::prelude::*;
    ///
    ///     mk_slots!(slot);
    ///     //      boxed_slice: StackBox<'_, [String]> = StackBox::new_in(slot, [
    ///     let mut boxed_slice: StackBox<'_, [String]> = stackbox!(slot, [
    ///         "Hello, World!".into()
    ///     ]);
    ///     let _: String = boxed_slice.stackbox_pop().unwrap();
    ///     ```
    #[inline]
    pub
    fn into_slice (self: StackBox<'frame, Array>)
      -> StackBox<'frame, [Array::Item]>
    {
        IsArray::into_slice(self)
    }
}

macro_rules! impl_for_Ns {(
    $(
        $(@for [$($generics:tt)*])?
        $N:expr
    ),+ $(,)?
) => (
    $(
        impl<'frame, Item : 'frame $(, $($generics)*)?>
            IsArray<'frame>
        for
            [Item; $N]
        {
            type Item = Item;

            #[inline]
            fn into_slice (this: StackBox<'frame, [Item; $N]>)
              -> StackBox<'frame, [Item]>
            {
                unsafe {
                    let ptr: *mut [Item; $N] =
                        <*const _>::read(&
                            ::core::mem::ManuallyDrop::new(this).unique_ptr
                        )
                            .into_raw_nonnull()
                            .as_ptr()
                    ;
                    let ptr: *mut [Item   ] = ptr;
                    StackBox {
                        unique_ptr: ptr::Unique::from_raw(ptr),
                        _covariant_lt: Default::default(),
                    }
                }
            }
        }
    )+
)}

#[cfg(feature = "const-generics")]
const _: () = {
    impl_for_Ns! {
        @for [const N: usize] N
    }
};

#[cfg(not(feature = "const-generics"))]
const _: () = {
    impl_for_Ns! {
        /* Is this a drawing of a flag? */
        00, 01, 02, 03, 04, 05, 06, 07,
        08, 09, 10, 11, 12, 13, 14, 15,
        16, 17, 18, 19, 20, 21, 22, 23,
        24, 25, 26, 27, 28, 29, 30, 31,
        32, 33, 34, 35, 36, 37, 38, 39,
        40, 41, 42, 43, 44, 45, 46, 47,
        48, 49, 50, 51, 52, 53, 54, 55,
        56, 57, 58, 59, 60, 61, 62, 63,
        64,
        75,
        96,
       100,
       125,
       128,
       150,
       175,
       192,
       200,
       250,
       256,
       300,
       400,
       500,
       512,
       750,
      1000,
      1024,
      2048,
      4096,
    }
};
