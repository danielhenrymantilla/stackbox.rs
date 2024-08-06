use crate::{prelude::*, ptr, Slot};
use ::core::mem::ManuallyDrop;

pub use slice::iter;
mod slice;

/// Stack<sup>1</sup>-allocated `Box`. Think of this as of `&'frame mut T`, but
/// with `move` semantics (no reborrowing!) which allow the "reference" to drop
/// its pointee.
///
/// <small><sup>1</sup> Pedantic nit: actually, it is _local_-allocated: if the
/// local is created inside a generator such as an `async` block or function,
/// crossing a `yield` / `.await` point (thus captured by the generator),
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
/// stackbox!(let boxed_slice: StackBox<'_, [_]> = [
///     String::from("Hello, "),
///     String::from("World!"),
/// ]);
/// for s in boxed_slice {
///     println!("{}", s);
///     stuff::<String>(s);
/// }
/// ```
///
///   - or with some [`#[with]` sugar:](https://docs.rs/with_locals):
///
///     <details>
///
///     ```rust
///     # use ::core::mem::drop as stuff;
///     use ::stackbox::prelude::*;
///     use ::with_locals::with;
///
///     #[with('local)]
///     fn main ()
///     {
///         let boxed_array: StackBox<'local, [String; 2]> = StackBox::new([
///             String::from("Hello, "),
///             String::from("World!"),
///         ]);
///         let boxed_slice: StackBox<'_, [String]> = boxed_array.into_slice();
///         for s in boxed_slice {
///             println!("{}", s);
///             stuff::<String>(s);
///         }
///     }
///     ```
///
///     ___
///
///     </details>
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
/// mk_slots!(storage1, storage2); // uninit stack allocations.
/// let boxed_slice_of_strings: StackBox<'_, [String]> =
///     if some_condition() {
///         StackBox::new_in(storage1, [
///             String::from("Hi."),
///         ])
///         .into_slice() // [String; 1] → [String]
///     } else {
///         // If using the macro, the coercion happens automagically
///         stackbox!(storage2, [
///             "Hello, ".into(),
///             "World!".into(),
///         ])
///     }
/// ;
/// for s in boxed_slice_of_strings {
///     println!("{}", s);
///     stuff::<String>(s);
/// }
/// ```
///
/// #### 2 - Allocation-less `dyn FnOnce` (and owned `dyn Any`)
///
/// See the [dedicated module for more info][`crate::dyn_traits`].
///
/// ```rust
/// use ::stackbox::prelude::*;
/// # let some_condition = || true;
///
/// mk_slots!(f1, f2);
/// let f: StackBoxDynFnOnce_0<()> = if some_condition() {
///     f1.stackbox(move || {
///         // …
///     }).into_dyn()
/// } else {
///     f2.stackbox(move || {
///         // …
///     }).into_dyn()
/// };
/// // …
/// f.call();
/// ```
// TYPE INVARIANTS:
//   - See the `# Safety` section of [`StackBox::assume_owns`].
#[repr(transparent)]
pub struct StackBox<'frame, T: ?Sized + 'frame> {
    /// Covariant and non-null and, ideally, tagged as unaliased.
    unique_ptr: ptr::Unique<T>,
    /// Covariant lifetime (this is an `&'frame mut MD<T>`, afterall).
    _covariant_lt: ::core::marker::PhantomData<&'frame ()>,
}

impl<'frame, T: 'frame> StackBox<'frame, T> {
    /// # Main non-`unsafe` non-macro non-callback constructor.
    ///
    /// To be used most of the time (when `T : Sized`, and when no implicit
    /// implicit coercion is needed).
    ///
    ///   - Creation of the [`Slot`]s is possible either manually, by binding
    ///     the return value of [`mk_slot()`] to some variable, by `ref mut`,
    ///     or if multiple slots are needed, they can be batch created thanks
    ///     to the [`mk_slots!`] convenience helper macro.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use ::stackbox::prelude::*;
    ///
    /// let slot = &mut mk_slot();
    /// let boxed = if true {
    ///     StackBox::new_in(slot, 42)
    /// } else {
    ///     StackBox::new_in(slot, 27)
    /// };
    /// assert_eq!(*boxed, 42);
    /// ```
    #[inline(always)]
    pub fn new_in(slot: &'frame mut Slot<T>, value: T) -> StackBox<'frame, T> {
        slot.stackbox(value)
    }

    /// Alternative non-`unsafe` non-macro constructor, where instead of an
    /// explicit [`Slot`] that defines the scope of validity of the
    /// [`StackBox`] (its stack frame), a callback is used: the `StackBox` is
    /// valid for the duration of the callback.
    ///
    /// ## Example
    ///
    /// ```rust
    /// use ::stackbox::prelude::*;
    ///
    /// StackBox::with_new(42, |stackbox: StackBox<'_, i32>| {
    ///     let any: StackBoxDynAny<'_> = stackbox.into_dyn();
    ///     assert_eq!(
    ///         any.downcast_ref::<i32>().unwrap(),
    ///         &42,
    ///     );
    /// }) // <- `StackBox` cannot outlive this point.
    /// ```
    ///
    /// ## Ergonomic usage thanks to `#[::with_locals::with]`
    ///
    /// Using this constructor can be made quite ergonomic by using the
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
    #[inline]
    pub fn with_new<R, F>(value: T, ret: F) -> R
    where
        F: for<'local> FnOnce(StackBox<'local, T>) -> R,
    {
        ret(StackBox::new_in(&mut mk_slot(), value))
    }

    /// Unwraps / extracts / moves the pointee out of the [`StackBox`].
    ///
    /// ### Note
    ///
    /// This lets the used [`Slot`] [vacant][`Slot::VACANT`] again, which can
    /// thus be reused to create another [`StackBox`].
    // Note: `self` receiver is fine because there is no `DerefMove` yet.
    #[inline]
    pub fn into_inner(self: StackBox<'frame, T>) -> T {
        unsafe {
            // Safety: from the type invariant.

            // 1 - Disable the `Drop` glue.
            let this = ManuallyDrop::new(self);
            // 2 - We can now *take* the value:
            ::core::ptr::read::<T>(&**this)
        }
    }
}

impl<'frame, T: ?Sized + 'frame> StackBox<'frame, T> {
    /// Raw `unsafe` constructor, by taking ownership of a borrowing pointer.
    ///
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
    ///   - Either [`StackBox::new_in`] (_e.g._, `Sized` case),
    ///
    ///       - (or the [CPS / callback](
    ///         https://en.wikipedia.org/wiki/Continuation-passing_style)-based
    ///         [`StackBox::with_new`] constructor).
    ///
    ///   - Or the [`stackbox!`] macro, for most usages.
    #[inline]
    pub unsafe fn assume_owns(it: &'frame mut ManuallyDrop<T>) -> StackBox<'frame, T> {
        Self {
            unique_ptr: ptr::Unique::<T>::from_raw(&mut **it),
            _covariant_lt: Default::default(),
        }
    }

    #[inline]
    pub(crate) fn into_inner_unique(self) -> ptr::Unique<T> {
        let this = ManuallyDrop::new(self);
        unsafe {
            // Safety: moving out of this which is not dropped.
            // This is basically destructuring self which impls `Drop`.
            ::core::ptr::read(&this.unique_ptr)
        }
    }
}

impl<'frame, T: ?Sized + 'frame> ::core::ops::Deref for StackBox<'frame, T> {
    type Target = T;

    #[inline]
    fn deref(self: &'_ StackBox<'frame, T>) -> &'_ T {
        &*self.unique_ptr
    }
}

impl<'frame, T: ?Sized + 'frame> ::core::ops::DerefMut for StackBox<'frame, T> {
    #[inline]
    fn deref_mut(self: &'_ mut StackBox<'frame, T>) -> &'_ mut T {
        &mut *self.unique_ptr
    }
}

impl<T: ?Sized> Drop for StackBox<'_, T> {
    #[inline]
    fn drop(self: &'_ mut Self) {
        unsafe {
            // # Safety
            //
            //   - From the type invariant
            ptr::Unique::<T>::drop_in_place(&mut self.unique_ptr)
        }
    }
}

#[cfg(feature = "unsize")]
/// Allows conversion to a `StackBox` containing an unsized type.
///
/// # Usage
///
/// ```
/// use core::fmt::Display;
/// use unsize::{Coercion, CoerceUnsize};
/// use stackbox::prelude::*;
///
/// let slot = &mut mk_slot();
/// let num = StackBox::<usize>::new_in(slot, 42);
///
/// let display: StackBox<dyn Display> = num.unsize(Coercion::to_display());
/// ```
unsafe impl<'frame, T: 'frame, U: ?Sized + 'frame> ::unsize::CoerciblePtr<U>
    for StackBox<'frame, T>
{
    type Pointee = T;
    type Output = StackBox<'frame, U>;

    fn as_sized_ptr(self: &mut Self) -> *mut T {
        &*self.unique_ptr as *const T as *mut T
    }

    unsafe fn replace_ptr(self, new: *mut U) -> StackBox<'frame, U> {
        let _covariant_lt = self._covariant_lt;

        let new_ptr = self.into_inner_unique().into_raw_nonnull().replace_ptr(new);

        // Safety: we've forgotten the old pointer and this is the correctly unsized old pointer so
        // valid for the pointed-to memory.
        let unique_ptr = ptr::Unique::from_raw(new_ptr.as_ptr());

        StackBox {
            unique_ptr,
            _covariant_lt,
        }
    }
}

/// Convenience macro for more ergonomic [`StackBox`] constructions.
#[macro_export]
macro_rules! stackbox {
    (
        // Same as `StackBox::new_in`, except for it allowing an unsized
        // coercion to take place.
        $place:expr,
        $value:expr $(,)?
    ) => (match ($place, $value) { (place, value) => {
        let ptr = $crate::Slot::__init_raw(place, value);
        unsafe {
            let _ = $crate::__::concat!(
                "Safety: `", stringify!($place), "` has just been initialized",
            );
            $crate::StackBox::assume_owns(ptr)
        }
    }});

    (
        // Create a new `mut` `StackBox` without mentioning the backing _slot_:
        // `let mut <binding> = stackbox!($expr);`
        // Examples:
        //   - `stackbox!(let mut new_var = <expr>);`
        //   - `stackbox!(let mut new_var: StackBox<[_]> = <array expr>);`
        let mut $var:ident $(: $T:ty)? = $expr:expr
    ) => (
        $crate::stackbox!($expr => let mut $var $(: $T)?)
    );

    (
        // Create a new `StackBox` without mentioning the backing _slot_:
        // `let <binding> = stackbox!($expr);`
        // Examples:
        //   - `stackbox!(let new_var = <expr>);`
        //   - `stackbox!(let new_var: StackBox<[_]> = <array expr>);`
        let $var:ident $(: $T:ty)? = $expr:expr
    ) => (
        $crate::stackbox!($expr => let $var $(: $T)?)
    );

    (
        // Internal-ish: assign the result of a `stackbox!($expr)` to "some place"
        // where "some place" may be a new `let` binding or an actual assignment.
        //
        // No need to explicitly mention the backing _slot_ either.
        // Examples:
        //   - `let var: Ty; stackbox!(<expr> => var);`
        $expr:expr => $($binding:tt)*
    ) => (
        let ref mut ptr = $crate::__::ManuallyDrop::new($expr);
        $($binding)* = unsafe { $crate::StackBox::assume_owns(ptr) };
    );

    (
        // Shorthand for `stackbox!(let mut $var = $var)`
        let mut $var:ident
    ) => (
        $crate::stackbox!(let mut $var = $var)
    );

    (
        // Shorthand for `stackbox!(let $var = $var)`
        let $var:ident
    ) => (
        $crate::stackbox!(let $var = $var)
    );

    (
        // To be used as a temporary fed to a function parameter, or as a
        // `[::with_locals::with]` "return" value.
        //
        // Examples:
        //   - `fun(stackbox!(value))`
        $expr:expr
    ) => (
        match &mut $crate::__::ManuallyDrop::new($expr) { ptr => {
            #[allow(unused_unsafe)] {
                unsafe {
                    // Safety: anonymous temporary is unusable
                    $crate::StackBox::assume_owns(ptr)
                }
            }
        }}
    );
}
