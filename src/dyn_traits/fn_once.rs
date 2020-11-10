//! Since `StackBox<'_, dyn FnOnce…>` does not auto-implement `FnOnce…`, we
//! need to do it manually.
#![allow(nonstandard_style)]

use super::*;

mod T { pub use crate::marker::Sendness::T as Sendness; }

generate!(_9 _8 _7 _6 _5 _4 _3 _2 _1 _0); macro_rules! generate {() => (); (
    $_N:tt $($_K:tt)*
) => (generate! { $($_K)* } ::paste::paste! {
    pub use [<FnOnce$_N>]::[<StackBoxDynFnOnce$_N>];
    mod [<FnOnce$_N>] {
        use super::*;

        pub
        struct [<StackBoxDynFnOnce$_N>] <
                'frame, $(
                [</*Arg*/$_K>], )*
                Ret,
                Sendness : ?Sized + T::Sendness = NoAutoTraits,
            >
        {
            ptr: ptr::NonNull<ty::Erased>,
            vtable: &'frame VTable<$([</*Arg*/$_K>] ,)* Ret>,
            _is_send: ::core::marker::PhantomData<Sendness>,
        }

        struct VTable<$([</*Arg*/$_K>] ,)* Ret> {
            drop_in_place: unsafe fn(ptr: ptr::NonNull<ty::Erased>),
            call_once: unsafe fn(
                ptr::NonNull<ty::Erased> $(,
                [</*Arg*/$_K>] )*
            ) -> Ret,
        }

        impl<$([</*Arg*/$_K>], )* Ret, F> HasVTable<$([</*Arg*/$_K>] ,)* Ret> for F
        where
            Self : Sized + FnOnce($([</*Arg*/$_K>]),*) -> Ret,
        {}
        trait HasVTable<$([</*Arg*/$_K>] ,)* Ret>
        where
            Self : Sized + FnOnce($([</*Arg*/$_K>]),*) -> Ret,
        {
            const VTABLE: VTable<$([</*Arg*/$_K>] ,)* Ret> = VTable {
                drop_in_place: {
                    unsafe
                    fn drop_in_place<Self_> (ptr: ptr::NonNull<ty::Erased>)
                    {
                        ptr::drop_in_place(ptr.cast::<Self_>().as_ptr())
                    }
                    drop_in_place::<Self>
                },
                call_once: {
                    unsafe
                    fn call_once<Self_, $([</*Arg*/$_K>] ,)* Ret> (
                        ptr: ptr::NonNull<ty::Erased> $(,
                        [</*arg*/$_K>]: [</*Arg*/$_K>] )*
                    ) -> Ret
                    where
                        Self_ : FnOnce($([</*Arg*/$_K>]),*) -> Ret,
                    {
                        let f: StackBox<'_, Self_> = {
                            ::core::mem::transmute(ptr)
                        };
                        let f: Self_ = StackBox::into_inner(f);
                        f($([</*arg*/$_K>]),*)
                    }
                    call_once::<Self, $([</*Arg*/$_K>] ,)* Ret>
                },
            };
        }

        impl<'frame, $([</*Arg*/$_K>] ,)* Ret, F : 'frame>
            DynCoerce<StackBox<'frame, F>>
        for
            [<StackBoxDynFnOnce$_N>]<'frame, $([</*Arg*/$_K>] ,)* Ret>
        where
            F : FnOnce($([</*Arg*/$_K>]),*) -> Ret,
        {
            fn fatten (it: StackBox<'frame, F>)
              -> Self
            {
                [<StackBoxDynFnOnce$_N>] {
                    vtable: &<F as HasVTable<$([</*Arg*/$_K>] ,)* Ret>>::VTABLE,
                    ptr: unsafe { ::core::mem::transmute(it) },
                    _is_send: ::core::marker::PhantomData,
                }
            }
        }

        /// And now with the `Send` bound
        impl<'frame, $([</*Arg*/$_K>] ,)* Ret, F : 'frame>
            DynCoerce<StackBox<'frame, F>>
        for
            [<StackBoxDynFnOnce$_N>]<'frame, $([</*Arg*/$_K>] ,)* Ret, dyn Send>
        where
            F : FnOnce($([</*Arg*/$_K>]),*) -> Ret,
            F : Send,
        {
            fn fatten (it: StackBox<'frame, F>)
              -> Self
            {
                [<StackBoxDynFnOnce$_N>] {
                    vtable: &<F as HasVTable<$([</*Arg*/$_K>] ,)* Ret>>::VTABLE,
                    ptr: unsafe { ::core::mem::transmute(it) },
                    _is_send: ::core::marker::PhantomData,
                }
            }
        }

        impl<'frame, $([</*Arg*/$_K>] ,)* Ret, Sendness : ?Sized + T::Sendness>
            [<StackBoxDynFnOnce$_N>]<'frame, $([</*Arg*/$_K>] ,)* Ret, Sendness>
        {
            pub
            fn call (
                self: Self $(,
                [</*arg*/$_K>]: [</*Arg*/$_K>] )*
            ) -> Ret
            {
                unsafe {
                    let Self { ptr, vtable, .. } =
                        *::core::mem::ManuallyDrop::new(self)
                    ;
                    (vtable.call_once)(ptr, $([</*arg*/$_K>]),*)
                }
            }
        }

        impl<'frame, $([</*Arg*/$_K>] ,)* Ret, Sendness : ?Sized + T::Sendness> Drop
            for [<StackBoxDynFnOnce$_N>]<'frame, $([</*Arg*/$_K>] ,)* Ret, Sendness>
        {
            fn drop (self: &'_ mut Self)
            {
                unsafe {
                    (self.vtable.drop_in_place)(self.ptr)
                }
            }
        }

        unsafe // Safety: no shared API whatsoever
            impl<'frame, $([</*Arg*/$_K>] ,)* Ret, Sendness : ?Sized + T::Sendness>
                Sync
            for
                [<StackBoxDynFnOnce$_N>]<'frame, $([</*Arg*/$_K>] ,)* Ret, Sendness>
            {}

        unsafe // Safety: `Sendness = dyn Send` requires a `Send` bound on `F`:
            impl<'frame, $([</*Arg*/$_K>] ,)* Ret>
                Send
            for
                [<StackBoxDynFnOnce$_N>]<'frame, $([</*Arg*/$_K>] ,)* Ret, dyn Send>
            {}
    }
})} use generate;
