#![cfg_attr(not(any(doc, test)),
    no_std,
)]
#![allow(unused_parens)]
#![deny(rust_2018_idioms)]
#![allow(explicit_outlives_requirements)] // much unsafe code; better safe than sorry

#[cfg(test)]
extern crate self as stackbox;

pub
mod dyn_traits;

mod iter;

mod marker;

pub mod prelude {
    #[doc(no_inline)]
    pub use crate::{
        StackBox,
        stackbox,
        dyn_traits::{
            any::StackBoxDynAny,
            fn_once::*,
        },
    };
}

use ::core::mem::ManuallyDrop;

/// Stack<sup>1</sup>-allocated `Box`. Think of this as of `&'frame mut T`, but
/// with `move` semantics (no reborrowing!) which allow the "reference" to drop
/// its pointee.
///
/// <small><sup>1</sup> Pedantic nit: actually, it is _local_-allocated: if the
/// local is created inside a generator such as an `async` block or function,
/// and that generator / future is `Box`-ed, then the local will be living on
/// the heap.</small>
///
/// Given the `move` semantics / lack of reborrowing, there may seem to be
/// little point in using this over the seemingly more flexible
/// `&'frame mut T`, or the clearly more simple `T`.
///
/// And indeed that is mostly true: the usage of this wrapper is a bit _niche_.
/// Use this wrapper when:
///
///  1. You want / _need_ the move semantics (`FnOnce`) ⇒ no `&mut` for you
///     (assuming `Option::take` is too cumbersome, costly or directly unusable
///     for your use case).
///
///  1. You _need_ the indirection:
///
///       - if `T` is big, and you need move semantics, moving `T` around may
///         be expensive if the compiler is not able to elide the
///         bitwise-copies (`memcpy`) that happen when the value is moved.
///
///       - ### Main usage
///
///         If you need a **fat pointer to perform some type erasure**, while
///         preserving ownership / `move` semantics, and you don't want (or
///         actually _can't_) use the heap allocation from [`Box`], then this
///         type is for you!
///
/// ### Examples of type erasure
///
/// #### 1 - Array to slice coercion and `IntoIter`
///
/// `IntoIterator` for ~~arrays~~ slices:
///
/// ```rust
/// # use ::core::mem::drop as stuff;
/// use ::stackbox::prelude::*;
///
/// stackbox!([
///     String::from("Hello, "),
///     String::from("World!"),
/// ] => let boxed_slice: StackBox<'_, [_]>);
/// for s in boxed_slice {
///     println!("{}", s);
///     stuff::<String>(s);
/// }
/// ```
//
//   - or with some [`#[with]` sugar:](https://docs.rs/with_locals):
//
//     <details>
//
//     ```rust
//     # use ::core::mem::drop as stuff;
//     use ::stackbox::prelude::*;
//     use ::with_locals::with;
//
//     #[with]
//     fn main ()
//     {
//         #[with]
//         let boxed_array = StackBox::new([
//             String::from("Hello, "),
//             String::from("World!"),
//         ]);
//         let boxed_slice: StackBox<'_, [_]> = boxed_array;
//         for s in boxed_slice {
//             println!("{}", s);
//             stuff::<String>(s);
//         }
//     }
//     ```
//
//     ___
//
//     </details>
///
/// While `&mut [T; N] → &mut [T]` already covers most of the use cases,
/// imagine needing the `[T]` slice type erasure (_e.g._, an `if` branch which
/// yields arrays of different lengths) and also needing to have
/// [`IntoIterator`] available to you. And you don't want to "stupidly" pay a
/// heap allocation for something that should not deserve one:
///
/// ```rust
/// # use ::core::mem::drop as stuff;
/// use ::core::mem::ManuallyDrop;
/// use ::stackbox::prelude::*;
///
/// # let some_condition = || true;
/// let mut storage = (None, None);
/// let boxed_slice_of_strings: StackBox<'_, [String]> =
///     if some_condition() {
///         let p0 = storage.0.get_or_insert(ManuallyDrop::<[String; 1]>::new([
///             "Hi.".into(),
///         ]));
///         unsafe {
///             // Safety: nobody else may free `storage.0`
///             StackBox::assume_owns(p0)
///         }
///     } else {
///         let p1 = storage.1.get_or_insert(ManuallyDrop::<[String; 2]>::new([
///             "Hello, ".into(),
///             "World!".into(),
///         ]));
///         unsafe {
///             // Safety: nobody else may free `storage.0`
///             StackBox::assume_owns(p1)
///         }
///     }
/// ;
/// for s in boxed_slice_of_strings {
///     println!("{}", s);
///     stuff::<String>(s);
/// }
/// ```
#[repr(transparent)]
pub
struct StackBox<'frame, T : ?Sized + 'frame> (
    /// TYPE INVARIANTS:
    ///   - See the `# Safety` section of [`StackBox::assume_owns`].
    &'frame mut ManuallyDrop<T>,
);

impl<'frame, T : 'frame> StackBox<'frame, T> {
    /// ### `#[::with_locals::with]`
    /// Using this constructor can be made quite ergonomic if using the
    /// [`#[with]` CPS sugar](https://docs.rs/with_locals):
    ///
    /// ```rust
    /// # macro_rules! ignore {($($t:tt)*) => ()} ignore! {
    /// use ::stackbox::prelude::*;
    /// use ::with_locals::with;
    ///
    /// #[with]
    /// fn main ()
    /// {
    ///     let stackbox: StackBox<'ref, /* … */> = StackBox::new({
    ///         /* … */
    ///     });
    ///     // …
    /// }
    /// # } fn main () {}
    /// ```
    pub
    fn with_new<R, F> (value: T, ret: F) -> R
    where
        F: for<'local> FnOnce(StackBox<'local, T>) -> R,
    {
        ret(stackbox!(value))
    }

    // Note: `self` receiver is fine because there is no `DerefMove` yet.
    pub
    fn into_inner (self: StackBox<'frame, T>)
      -> T
    {
        unsafe {
            // Safety: from the type invariant.

            // 1 - Disable the `Drop` glue.
            let mut this = ManuallyDrop::new(self);
            // 2 - We can now *take* the value:
            ManuallyDrop::take(&mut *this.0)
        }
    }
}

impl<'frame, T : ?Sized + 'frame> StackBox<'frame, T> {
    /// # Safety
    ///
    /// This type has ownership of the pointee `T`. This means that despite the
    /// borrow-looking nature of the `&'frame mut`, the pointee should not be
    /// used (⇒ not dropped!) once it has been pointed to by a `StackBox` /
    /// given to this function: the `ManuallyDrop<T>` pointee will represent
    /// deallocated memory after the `'frame` lifetime!
    ///
    /// As a rule of thumb, it is _sound_ to call this function when and only
    /// when calling [`ManuallyDrop::drop`] is.
    ///
    /// When possible (`T : Sized`), prefer to use the non-`unsafe`
    /// constructors:
    ///
    ///   - Either the [`stackbox!`] macro,
    ///
    ///   - Or the [CPS / callback](
    ///     https://en.wikipedia.org/wiki/Continuation-passing_style)-based
    ///     [`StackBox::with_new`] constructor.
    pub
    unsafe
    fn assume_owns (it: &'frame mut ManuallyDrop<T>)
      -> StackBox<'frame, T>
    {
        Self(it)
    }
}

impl<'frame, T : ?Sized + 'frame>
    ::core::ops::Deref
for
    StackBox<'frame, T>
{
    type Target = T;

    fn deref (self: &'_ StackBox<'frame, T>)
      -> &'_ T
    {
        &**self.0
    }
}

impl<'frame, T : ?Sized + 'frame>
    ::core::ops::DerefMut
for
    StackBox<'frame, T>
{
    fn deref_mut (self: &'_ mut StackBox<'frame, T>)
      -> &'_ mut T
    {
        &mut **self.0
    }
}

impl<T : ?Sized> Drop for StackBox<'_, T> {
    fn drop (self: &'_ mut Self)
    {
        unsafe {
            // # Safety
            //
            //   - From the type invariant
            let Self(&mut ref mut this) = *self;
            ManuallyDrop::drop(this)
        }
    }
}

impl<'frame, Item : 'frame> StackBox<'frame, [Item]> {
    /// # Safety
    ///
    /// Same requirements as [`StackBox::assume_owns`].
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

    pub
    fn stackbox_pop (self: &'_ mut StackBox<'frame, [Item]>)
      -> Option<Item>
    {
        if self.is_empty() {
            return None;
        }
        let placeholder = unsafe {
            // Safety: empty slice.
            StackBox::assume_owns_all(&mut [][..])
        };
        let this = ::core::mem::replace(self, placeholder);
        let (hd, tl) = this.stackbox_split_at(1);
        *self = tl;
        Some(unsafe {
            ::core::ptr::read(&ManuallyDrop::new(hd)[0])
        })
    }
}

#[macro_export]
macro_rules! stackbox {
    (
        // `<binding> = stackbox!($expr);`
        // Examples:
        //   - `stackbox!(<expr> => let new_var);`
        //   - `let var; stackbox!(<expr> => var);`
        //   - `stackbox!(<expr> => let mut mutable_var);`
        $expr:expr => $($binding:tt)*
    ) => (
        let ref mut ptr = $crate::__::ManuallyDrop::new($expr);
        $($binding)* = unsafe { $crate::StackBox::assume_owns(ptr) };
    );

    (
        // To be used as a temporary fed to a function parameter, or as a
        // `[::with_locals::with]` "return" value.
        $expr:expr
    ) => (
        // Do not wrap `$expr` in `unsafe` hygiene
        ({
            // Use an `fn` to avoid `unused_unsafe` lint.
            #[allow(unsafe_code)]
            fn it<T : ?$crate::__::Sized> (ptr: &'_ mut $crate::__::ManuallyDrop<T>)
              -> $crate::StackBox<'_, T>
            { unsafe {
                // Safety: only visible / usable within the very next call.
                $crate::StackBox::assume_owns(ptr)
            }}
            it
        })(
            &mut $crate::__::ManuallyDrop::new($expr)
        )
    );
}


#[doc(hidden)] /** Macro internals, not subject to semver rules */ pub
mod __ {
    pub use ::core::{
        marker::Sized,
        mem::ManuallyDrop,
    };
}
