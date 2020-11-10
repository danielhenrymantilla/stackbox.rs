//! Hand-rolled implementations of pervasive `StackBox<dyn …>` types.
//!
//! ### Explanation
//!
//! Given the baked-into-the-language nature of trait objects, receiver types,
//! and auto-generated vtables, things like `StackBox<'_, dyn Any>` _& co._
//! Do Not Work™.
//!
//! So we need to hand-roll those.
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
//! [`.coerce_into_dyn()`][`StackBox::coerce_into_dyn`] method (thanks to an
//! internal trait: in the future, such trait may be exposed in the public
//! API to allow for downstream extensions of this module).

pub
mod any;

pub
mod fn_once;

use crate::{
    marker::{NoAutoTraits, Sendness, Syncness},
    prelude::*,
};
use ::core::ptr;

mod ty { pub struct Erased(()); }

mod __ {
    pub
    trait DynCoerce<StackBoxImplTrait> {
        fn fatten (it: StackBoxImplTrait)
          -> Self /* StackBoxDynTrait */
        ;
    }
}
use __::DynCoerce;

impl<'frame, ImplTrait : 'frame> StackBox<'frame, ImplTrait> {
    pub
    fn coerce_into_dyn<StackBoxDynTrait> (self: StackBox<'frame, ImplTrait>)
      -> StackBoxDynTrait
    where
        StackBoxDynTrait : DynCoerce<StackBox<'frame, ImplTrait>>,
    {
        DynCoerce::fatten(self)
    }
}

#[cfg(any(test, doctest))]
mod tests;
