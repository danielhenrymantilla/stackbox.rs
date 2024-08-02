use crate::prelude::*;

use ::core::{
    // marker::PhantomData,
    mem,
};

/// Same as [`Slot::VACANT`], but using function call syntax to avoid firing
/// the [trigger-happy `const_item_mutation` lint](
/// https://github.com/rust-lang/rust/pull/75573).
///
/// A so-created `&mut Slot<T>` is then to be used to derive a
/// [`StackBox`]`<'slot, T>` out of it, by feeding a value to the slot:
///
///   - either by using [`StackBox::new_in()`],
///
///   - or the equivalent [`.stackbox()`] convenience method on [`Slot`]s.
///
/// [`.stackbox()`]: `Slot::stackbox`
#[inline(always)]
pub const fn mk_slot<T>() -> Slot<T> {
    Slot::VACANT
}

/// A `Sized` and [`uninit`][`::core::mem::MaybeUninit`] slot to
/// manually handle the scope of [`StackBox`]'s backing inline allocations.
///
/// If needed, multiple such slots can be defined within the local scope and
/// bound to a variadic number of identifiers (variable names) using the
/// [`mk_slots!`][`crate::mk_slots`] macro.
pub struct Slot<T> {
    place: mem::MaybeUninit<T>,
    // /// Invariant lifetime just in case.
    // _borrow_mut_once: PhantomData<fn(&()) -> &mut &'frame ()>,
}

impl<T> Slot<T> {
    /// A vacant slot, to be used to derive a [`StackBox`]`<'slot, T>` out of
    /// it, by feeding a value to the slot:
    ///
    ///   - either by using [`StackBox::new_in()`],
    ///
    ///   - or the equivalent [`.stackbox()`] convenience method on [`Slot`]s.
    ///
    /// [`.stackbox()`]: `Slot::stackbox`
    pub const VACANT: Self = Slot {
        place: mem::MaybeUninit::uninit(),
        // _borrow_mut_once: PhantomData,
    };

    /// Convenience shortcut for [`StackBox::new_in()`].
    #[inline(always)]
    pub fn stackbox<'frame>(self: &'frame mut Slot<T>, value: T) -> StackBox<'frame, T>
    where
        T: 'frame,
    {
        let ptr = Self::__init_raw(self, value);
        unsafe {
            // Safety: `mem::MaybeUninit` does not drop it.
            StackBox::assume_owns(ptr)
        }
    }

    #[doc(hidden)]
    /** Not part of the public API */
    pub fn __init_raw<'frame>(
        this: &'frame mut Slot<T>,
        value: T,
    ) -> &'frame mut ::core::mem::ManuallyDrop<T> {
        this.place = mem::MaybeUninit::new(value);
        unsafe {
            // Safety: value has been initialized.
            mem::transmute::<&'_ mut mem::MaybeUninit<T>, &'_ mut mem::ManuallyDrop<T>>(
                &mut this.place,
            )
        }
    }
}

/// Convenience macro to batch-create multiple [`Slot`]s.
///
/// ```rust
/// # use ::stackbox::prelude::*;
/// mk_slots!(foo, bar, baz);
/// # drop::<[&mut ::stackbox::Slot<()>; 3]>([foo, bar, baz]);
/// ```
///
/// is a shorthand for multiple [`mk_slot()`][`mk_slot`] calls:
///
/// ```rust
/// # use ::stackbox::prelude::*;
/// let foo = &mut mk_slot();
/// let bar = &mut mk_slot();
/// let baz = &mut mk_slot();
/// # drop::<[&mut ::stackbox::Slot<()>; 3]>([foo, bar, baz]);
/// ```
///
/// A so-created `&mut Slot<T>` is then to be used to derive a
/// [`StackBox`]`<'slot, T>` out of it, by feeding a value to the slot:
///
///   - either by using [`StackBox::new_in()`],
///
///   - or the equivalent [`.stackbox()`] convenience method on [`Slot`]s.
///
/// [`.stackbox()`]: `Slot::stackbox`
#[macro_export]
macro_rules! mk_slots {(
    $($var_name:ident),+ $(,)?
) => (
    $(
        let ref mut $var_name = $crate::prelude::mk_slot();
    )+
)}
