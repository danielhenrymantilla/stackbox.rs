//! Hand-rolled implementations of pervasive `StackBox<dyn …>` types.
//!
//! ### Explanation
//!
//! Given the baked-into-the-language nature of trait objects, receiver types,
//! and auto-generated vtables, things like `StackBox<'_, dyn Any>` _& co._
//! Do Not Work™.
//!
//! So we need to hand-roll those, and this is what this module is about.
//!
//! ___
//!
//! Currently, only basic `FnOnce()` signatures and `Any` are supported,
//! since they are the only traits in the standard library with (part of) their
//! API _consuming ownership_ of the trait object, leading to `&'_ mut dyn …`
//! not sufficing (_c.f._, the classic "wrap that `FnOnce()` into an `Option`
//! to get an `FnMut()` you can thus type-erase with `&mut dyn FnMut()`"; which
//! exposes the pattern to runtime panics if the consumer of the
//! `&mut dyn FnMut()` calls it multiple times).
//!
//! ## Creation
//!
//! The `StackBoxDyn…` family of types is created using the
//! [`.into_dyn()`][`StackBox::into_dyn`] method. Such method is based on **an
//! internal trait**, that defines the vtable and usability of these wrapper
//! types, but the next section shows how downstream users may add
//! implementations of this trait.
//!
//! ### Custom-defined `StackBoxDyn…`s
//!
//! These can now be created thanks to the powerful [`custom_dyn!`] macro.
//!
//! [`custom_dyn!`]: `crate::custom_dyn`

pub mod any;

mod custom_dyn;

pub mod fn_once;

use crate::{
    marker::{NoAutoTraits, Sendness, Syncness},
    prelude::*,
};
use ::core::ptr;

mod ty {
    pub struct Erased(());
}

pub(crate) mod __ {
    pub trait DynCoerce<StackBoxImplTrait> {
        fn fatten(it: StackBoxImplTrait) -> Self /* StackBoxDynTrait */;
    }
}
use __::DynCoerce;

impl<'frame, ImplTrait: 'frame> StackBox<'frame, ImplTrait> {
    /// Coerces a `StackBox<impl Trait>` into a `StackBox<dyn Trait>`, provided
    /// the `Trait` is [one of the supported ones][`self`].
    pub fn into_dyn<StackBoxDynTrait>(self: StackBox<'frame, ImplTrait>) -> StackBoxDynTrait
    where
        StackBoxDynTrait: DynCoerce<StackBox<'frame, ImplTrait>>,
    {
        DynCoerce::fatten(self)
    }
}

#[cfg(any(test, doctest))]
mod tests;

#[cfg(test)]
mod my_test {
    use ::stackbox::prelude::*;

    /// Hack to have invocations work inside function bodies for the MSRV.
    macro_rules! custom_dyn {
        (
            @as_item
            $item:item
        ) => (
            $item
        );

        (
            $($input:tt)*
        ) => (
            custom_dyn! {@as_item
                ::stackbox::prelude::custom_dyn! {
                    $($input)*
                }
            }
        );
    }

    #[test]
    fn fn_once_higher_order_param() {
        custom_dyn! {
            dyn FnOnceRef<Arg> : FnOnce(&Arg)
            where {
                Arg : ?Sized,
            }
            {
                fn call (
                    self: Self,
                    s: &'_ Arg,
                )
                {
                    self(s)
                }
            }
        }

        let not_send = 42;
        stackbox!(let f = |_: &str| drop(not_send));
        let f: StackBoxDynFnOnceRef<'_, str, dyn Send + Sync> = f.into_dyn();
        let f = |s: &str| f.call(s);
        f("");
    }

    #[test]
    fn manual_any_non_owned_receiver() {
        use ::core::any;
        custom_dyn! {
            dyn Any<'__> : any::Any
            {
                fn type_id (self: &'_ Self, __: &'__ ())
                  -> any::TypeId
                {
                    any::TypeId::of::<Self>()
                }
            }
        }

        let it = ();
        stackbox!(let it);
        let it: StackBoxDynAny<'_, '_> = it.into_dyn();
        assert_eq!(it.type_id(&()), any::TypeId::of::<()>());
        assert_eq!(it.type_id(&()), any::TypeId::of::<()>());
    }
}
