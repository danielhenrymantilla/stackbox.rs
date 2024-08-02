//! [`StackBox`]: `StackBox`
//!
#![cfg_attr(feature = "docs", feature(external_doc), doc(include = "../README.md"))]
#![cfg_attr(not(any(doc, feature = "std", test)), no_std)]
#![cfg_attr(feature = "const-generics", feature(min_const_generics))]
#![allow(unused_parens)]
#![deny(rust_2018_idioms)]
#![allow(explicit_outlives_requirements)] // much unsafe code; better safe than sorry

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(test)]
extern crate self as stackbox;

pub mod dyn_traits;

mod marker;

mod ptr;

pub use slot::{mk_slot, Slot};
mod slot;

pub use stackbox_mod::StackBox;
#[path = "stackbox/mod.rs"]
mod stackbox_mod;

/// This crates prelude: usage of this crate is designed to be ergonomic
/// provided all the items within this module are in scope.
pub mod prelude {
    #[doc(no_inline)]
    pub use crate::{
        custom_dyn,
        dyn_traits::{any::StackBoxDynAny, fn_once::*},
        mk_slot, mk_slots, stackbox, StackBox,
    };
}

#[doc(hidden)]
/** Macro internals, not subject to semver rules */
pub mod __ {
    pub use ::core::{marker::Sized, mem::ManuallyDrop};

    pub trait GetVTable {
        type VTable;
    }
    pub use crate::{
        dyn_traits::__::DynCoerce,
        marker::{NoAutoTraits, Sendness::T as Sendness, Syncness::T as Syncness},
    };
    pub use ::core::{
        concat,
        marker::{PhantomData, Send, Sync},
        mem::transmute,
        ops::Drop,
    };
    pub use ::paste::paste;
    mod ty {
        pub struct Erased(());
    }
    pub type ErasedPtr = ::core::ptr::NonNull<ty::Erased>;

    pub unsafe fn drop_in_place<T>(ptr: ErasedPtr) {
        ::core::ptr::drop_in_place::<T>(ptr.cast::<T>().as_ptr());
    }
}
