//! Since `StackBox<'_, dyn Any…>` does not auto-implement `Any…`, we
//! need to do it manually.

#![allow(nonstandard_style)]
#![allow(unused_imports)] // issue #78894

use super::*;

use ::core::any::{Any, TypeId};

mod T {
    pub use super::Sendness::T as Sendness;
    pub use super::Syncness::T as Syncness;
}

pub use private::StackBoxDynAny;
mod private {
    use super::*;

    /// `StackBox<'frame, dyn Any + 'static + AutoTraits>`.
    ///
    /// ### `AutoTraits`: `Send / Sync`
    ///
    ///  - `dyn Any` → `StackBoxDynAny`;
    ///
    ///  - `dyn Any + Send` → `StackBoxDynAny<dyn Send>`;
    ///
    ///  - `dyn Any + Sync` → `StackBoxDynAny<dyn Sync>`;
    ///
    ///  - `dyn Any + Send + Sync` → `StackBoxDynAny<dyn Send + Sync>`;
    pub struct StackBoxDynAny<'frame, AutoTraits: ?Sized + T::Sendness + T::Syncness = NoAutoTraits> {
        ptr: ptr::NonNull<ty::Erased>,
        vtable: &'frame VTable,
        _auto_traits: ::core::marker::PhantomData<AutoTraits>,
    }

    struct VTable {
        drop_in_place: unsafe fn(ptr: ptr::NonNull<ty::Erased>),
        /// could be a `const` if that constructor was made `const`
        type_id: fn() -> TypeId,
        as_Any: unsafe fn(ptr: ptr::NonNull<ty::Erased>) -> ptr::NonNull<dyn Any + 'static>,
    }

    impl<T> HasVTable for T where Self: Sized + Any {}
    trait HasVTable
    where
        Self: Sized + Any,
    {
        const VTABLE: VTable = VTable {
            drop_in_place: {
                unsafe fn drop_in_place<Self_>(ptr: ptr::NonNull<ty::Erased>) {
                    ptr::drop_in_place(ptr.cast::<Self_>().as_ptr())
                }
                drop_in_place::<Self>
            },
            type_id: || TypeId::of::<Self>(),
            as_Any: {
                unsafe fn it<Self_: Any + 'static>(
                    ptr: ptr::NonNull<ty::Erased>,
                ) -> ptr::NonNull<dyn Any + 'static> {
                    let ptr: ptr::NonNull<Self_> = ptr.cast();
                    ptr as ptr::NonNull<dyn Any + 'static>
                }
                it::<Self>
            },
        };
    }

    define_coercions! {
        [Send] => dyn Send,
        [Sync] => dyn Sync,
        [Send, Sync] => dyn Send + Sync,
        [] => NoAutoTraits,
    }
    macro_rules! define_coercions {(
        $(
            [$($AutoTrait:ident),* $(,)?] => $Marker:ty
        ),* $(,)?
    ) => (
        $(
            impl<'frame, T : 'frame>
                DynCoerce<StackBox<'frame, T>>
            for
                StackBoxDynAny<'frame, $Marker>
            where
                T : Any,
                $(
                    T : $AutoTrait,
                )*
            {
                fn fatten (it: StackBox<'frame, T>)
                  -> Self
                {
                    StackBoxDynAny {
                        vtable: &<T as HasVTable>::VTABLE,
                        ptr: unsafe { ::core::mem::transmute(it) },
                        _auto_traits: ::core::marker::PhantomData,
                    }
                }
            }

            $(
                unsafe // Safety: from the `DynCoerce` bound added at construction site.
                    impl<'frame>
                        $AutoTrait
                    for
                        StackBoxDynAny<'frame, $Marker>
                    {}
            )*
        )*
    )}
    use define_coercions;

    impl<'frame, AutoTraits: ?Sized + T::Sendness + T::Syncness> StackBoxDynAny<'frame, AutoTraits> {
        #[inline]
        pub fn type_id(self: &'_ Self) -> TypeId {
            (self.vtable.type_id)()
        }

        #[inline]
        pub fn is<U: Any>(self: &'_ Self) -> bool {
            self.type_id() == TypeId::of::<U>()
        }

        #[inline]
        pub fn downcast_ref<U: Any>(self: &'_ Self) -> Option<&'_ U> {
            if self.is::<U>() {
                unsafe { Some(::core::mem::transmute(self.ptr)) }
            } else {
                None
            }
        }

        #[inline]
        pub fn downcast_mut<U: Any>(self: &'_ mut Self) -> Option<&'_ mut U> {
            if self.is::<U>() {
                unsafe { Some(::core::mem::transmute(self.ptr)) }
            } else {
                None
            }
        }

        #[inline]
        pub fn downcast<U: Any>(self: Self) -> Result<StackBox<'frame, U>, Self> {
            if self.is::<U>() {
                unsafe {
                    let ptr = ::core::mem::ManuallyDrop::new(self).ptr;
                    Ok(::core::mem::transmute(ptr))
                }
            } else {
                Err(self)
            }
        }

        #[inline]
        pub fn as_Any(self: &'_ Self) -> &'_ (dyn Any + 'static) {
            derive_AsRef_for_auto_trait_combination! {
                (), (Sync), // (Send), (Send + Sync), /* These should not be required */
            }
            macro_rules! derive_AsRef_for_auto_trait_combination {(
                $(
                    ( $($($auto_traits:tt)+)? )
                ),* $(,)?
            ) => (
                $(
                    impl<'frame, AutoTraits : ?Sized + T::Sendness + T::Syncness>
                        AsRef<dyn Any $(+ $($auto_traits)+)? + 'static>
                    for
                        StackBoxDynAny<'frame, AutoTraits>
                    $(
                        where
                            Self : $($auto_traits)+,
                    )?
                    {
                        fn as_ref (self: &'_ Self)
                          -> &'_ (dyn Any $(+ $($auto_traits)+)? + 'static)
                        {
                            unsafe {
                                ::core::mem::transmute(
                                    (self.vtable.as_Any)(self.ptr)
                                )
                            }
                        }
                    }
                )*
            )}
            use derive_AsRef_for_auto_trait_combination;

            self.as_ref()
        }

        #[inline]
        pub fn as_Any_mut(self: &'_ mut Self) -> &'_ mut (dyn Any + 'static) {
            derive_AsMut_for_auto_trait_combination! {
                (), (Send), (Sync), (Send + Sync)
            }
            macro_rules! derive_AsMut_for_auto_trait_combination {(
                $(
                    ( $($($auto_traits:tt)+)? )
                ),* $(,)?
            ) => (
                $(
                    impl<'frame, AutoTraits : ?Sized + T::Sendness + T::Syncness>
                        AsMut<dyn Any $(+ $($auto_traits)+)? + 'static>
                    for
                        StackBoxDynAny<'frame, AutoTraits>
                    $(
                        where
                            Self : $($auto_traits)+,
                    )?
                    {
                        fn as_mut (self: &'_ mut Self)
                          -> &'_ mut (dyn Any $(+ $($auto_traits)+)? + 'static)
                        {
                            unsafe {
                                ::core::mem::transmute(
                                    (self.vtable.as_Any)(self.ptr)
                                )
                            }
                        }
                    }
                )*
            )}
            use derive_AsMut_for_auto_trait_combination;

            self.as_mut()
        }
    }

    impl<'frame, AutoTraits: ?Sized + T::Sendness + T::Syncness> Drop
        for StackBoxDynAny<'frame, AutoTraits>
    {
        fn drop(self: &'_ mut Self) {
            unsafe { (self.vtable.drop_in_place)(self.ptr) }
        }
    }

    impl<'frame, AutoTraits: ?Sized + T::Sendness + T::Syncness> ::core::fmt::Debug
        for StackBoxDynAny<'frame, AutoTraits>
    {
        fn fmt(self: &'_ Self, f: &'_ mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.pad("Any")
        }
    }
}
