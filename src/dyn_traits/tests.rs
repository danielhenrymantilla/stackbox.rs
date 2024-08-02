use super::*;

use ::core::ops::Not as _;

mod any {
    use super::*;

    #[test]
    fn coerce_unsync_unsend_into_any() {
        stackbox!(let mut stackbox = ::core::ptr::null::<()>());
        let mut dyn_any: StackBoxDynAny<'_> = stackbox.into_dyn();
        assert!(dyn_any.is::<*const ()>());
        assert!(dyn_any.is::<bool>().not());
        let &(_): &'_ (*const ()) = dyn_any.downcast_ref().unwrap();
        let &mut (_): &'_ mut (*const ()) = dyn_any.downcast_mut().unwrap();
        stackbox = dyn_any.downcast().unwrap();
        drop(stackbox);
    }

    #[test]
    fn coerce_sync_unsend_into_sync_any() {
        #[derive(Default)]
        struct PhantomUnsend(::core::marker::PhantomData<*mut ()>);
        unsafe impl Sync for PhantomUnsend {}

        stackbox!(let stackbox = PhantomUnsend::default());
        let _: StackBoxDynAny<'_, dyn Sync> = stackbox.into_dyn();
    }

    #[test]
    fn coerce_send_unsync_into_send_any() {
        stackbox!(let stackbox = ::core::cell::Cell::new(0_u8));
        let _: StackBoxDynAny<'_, dyn Send> = stackbox.into_dyn();
    }

    #[test]
    fn coerce_send_sync_into_send_sync_any() {
        stackbox!(let stackbox = ());
        let _: StackBoxDynAny<'_, dyn Send + Sync> = stackbox.into_dyn();
    }

    #[test]
    fn test_drops() {
        let rc = ::std::rc::Rc::new(());
        let count = || ::std::rc::Rc::strong_count(&rc);
        let rc = || rc.clone();

        stackbox!(let stackbox = rc());
        assert_eq!(count(), 2);
        drop(stackbox);
        assert_eq!(count(), 1);

        stackbox!(let stackbox = rc());
        assert_eq!(count(), 2);
        let rc2 = StackBox::into_inner(stackbox);
        assert_eq!(count(), 2);
        drop(rc2);
        assert_eq!(count(), 1);

        stackbox!(let stackbox = rc());
        assert_eq!(count(), 2);
        let dyn_any: StackBoxDynAny<'_> = stackbox.into_dyn();
        assert_eq!(count(), 2);
        drop(dyn_any);
        assert_eq!(count(), 1);

        stackbox!(let mut stackbox = rc());
        assert_eq!(count(), 2);
        let dyn_any: StackBoxDynAny<'_> = stackbox.into_dyn();
        assert_eq!(count(), 2);
        stackbox = dyn_any.downcast().unwrap();
        assert_eq!(count(), 2);
        drop(stackbox);
        assert_eq!(count(), 1);
    }

    compile_fail! {
        #![name = cannot_coerce_unsync_into_sync_any]

        stackbox!(let stackbox = ::core::cell::Cell::new(0_u8));
        let _: StackBoxDynAny<'_, dyn Sync> = stackbox.into_dyn();
    }

    compile_fail! {
        #![name = cannot_coerce_unsend_into_send_any]

        stackbox!(let stackbox = ::core::ptr::null::<()>());
        let _: StackBoxDynAny<'_, dyn Send> = stackbox.into_dyn();
    }

    compile_fail! {
        #![name = cannot_coerce_unsync_into_sync_send_any]

        stackbox!(let stackbox = ::core::cell::Cell::new(0_u8));
        let _: StackBoxDynAny<'_, dyn Send + Sync> = stackbox.into_dyn();
    }

    compile_fail! {
        #![name = cannot_coerce_unsend_into_sync_send_any]

        stackbox!(let stackbox = ::core::ptr::null::<()>());
        let _: StackBoxDynAny<'_, dyn Send + Sync> = stackbox.into_dyn();
    }
}

mod fn_once {
    use super::*;

    #[test]
    fn move_semantics() {
        let not_copy: [&'static mut (); 0] = [];
        stackbox!(let stackbox_fn_once = || drop(not_copy));
        let mut dyn_fn_once: StackBoxDynFnOnce_0<'_, ()> = stackbox_fn_once.into_dyn();
        dyn_fn_once.call();
        let not_copy: [&'static mut (); 0] = [];
        stackbox!(let stackbox_fn_once = || drop(not_copy));
        dyn_fn_once = stackbox_fn_once.into_dyn();
        drop(dyn_fn_once);
    }

    compile_fail! {
        #![name = move_semantics_2]

        let mut not_copy: [&'static mut (); 0] = [];
        stackbox!(let stackbox_fn_once = || drop(not_copy));
        let mut dyn_fn_once: StackBoxDynFnOnce_0<'_, ()> = stackbox_fn_once.into_dyn();
        drop({ "use of moved value"; stackbox_fn_once });
        dyn_fn_once.call();
        drop({ "use of moved value"; dyn_fn_once });
    }

    #[test]
    fn test_drops() {
        let rc = ::std::rc::Rc::new(());
        let count = || ::std::rc::Rc::strong_count(&rc);
        let rc = || {
            let rc = rc.clone();
            move || drop(rc)
        };

        stackbox!(let stackbox = rc());
        assert_eq!(count(), 2);
        drop(stackbox);
        assert_eq!(count(), 1);

        stackbox!(let stackbox = rc());
        assert_eq!(count(), 2);
        let f = StackBox::into_inner(stackbox);
        assert_eq!(count(), 2);
        drop(f);
        assert_eq!(count(), 1);

        stackbox!(let stackbox = rc());
        assert_eq!(count(), 2);
        let f = StackBox::into_inner(stackbox);
        assert_eq!(count(), 2);
        f();
        assert_eq!(count(), 1);

        stackbox!(let stackbox = rc());
        assert_eq!(count(), 2);
        let dyn_fn: StackBoxDynFnOnce_0<'_, ()> = stackbox.into_dyn();
        assert_eq!(count(), 2);
        drop(dyn_fn);
        assert_eq!(count(), 1);

        stackbox!(let stackbox = rc());
        assert_eq!(count(), 2);
        let dyn_fn: StackBoxDynFnOnce_0<'_, ()> = stackbox.into_dyn();
        assert_eq!(count(), 2);
        dyn_fn.call();
        assert_eq!(count(), 1);
    }
}

mod custom_dyn {
    use super::compile_fail;
    use ::stackbox::prelude::*;

    mod fn_once {
        use super::*;

        custom_dyn! {
            dyn FnOnceRef<Arg> : FnOnce(&Arg)
            where { Arg : ?Sized }
            {
                fn call (self: Self, s: &'_ Arg)
                {
                    self(s)
                }
            }
        }

        #[test]
        fn fn_once_higher_order_param() {
            stackbox!(let f = |_: &str| ());
            let f: StackBoxDynFnOnceRef<'_, str, dyn Send + Sync> = f.into_dyn();
            let f = |s: &str| f.call(s);
            f("");
        }

        compile_fail! {
            #![name = auto_traits]
            custom_dyn! {
                dyn FnOnceRef<Arg> : FnOnce(&Arg)
                where { Arg : ?Sized }
                {
                    fn call (self: Self, s: &'_ Arg)
                    {
                        self(s)
                    }
                }
            }

            let not_send = ::std::rc::Rc::new(());
            stackbox!(let f = |_: &str| drop(not_send));
            let f: StackBoxDynFnOnceRef<'_, str, dyn Send> =
                f.into_dyn()
            ;
            f.call("");
        }

        #[test]
        fn test_drops() {
            let rc = ::std::rc::Rc::new(());
            let count = || ::std::rc::Rc::strong_count(&rc);
            let rc = || {
                let rc = rc.clone();
                move |_: &str| drop(rc)
            };

            stackbox!(let stackbox = rc());
            assert_eq!(count(), 2);
            drop(stackbox);
            assert_eq!(count(), 1);

            stackbox!(let stackbox = rc());
            assert_eq!(count(), 2);
            let f = StackBox::into_inner(stackbox);
            assert_eq!(count(), 2);
            f("");
            assert_eq!(count(), 1);

            stackbox!(let stackbox = rc());
            assert_eq!(count(), 2);
            let f = StackBox::into_inner(stackbox);
            assert_eq!(count(), 2);
            drop(f);
            assert_eq!(count(), 1);

            stackbox!(let stackbox = rc());
            assert_eq!(count(), 2);
            let f = StackBox::into_inner(stackbox);
            assert_eq!(count(), 2);

            // The following would fail should the lifetime param not be higher-order
            if false {
                f(&String::new());
                loop {}
            }
            f(&String::new());
            assert_eq!(count(), 1);

            stackbox!(let stackbox = rc());
            assert_eq!(count(), 2);
            let dyn_fn: StackBoxDynFnOnceRef<'_, str> = stackbox.into_dyn();
            assert_eq!(count(), 2);
            drop(dyn_fn);
            assert_eq!(count(), 1);

            stackbox!(let stackbox = rc());
            assert_eq!(count(), 2);
            let dyn_fn: StackBoxDynFnOnceRef<'_, str> = stackbox.into_dyn();
            assert_eq!(count(), 2);
            // The following would fail should the lifetime param not be higher-order
            if false {
                dyn_fn.call(&String::new());
                loop {}
            }
            dyn_fn.call(&String::new());
            assert_eq!(count(), 1);
        }
    }

    #[test]
    fn non_owned_receiver() {
        use ::core::any;

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

        // Test calling that macro inside an `fn` body
        custom_dyn! {
            dyn UselessAny : any::Any
            {
                fn type_id (self: &'_ Self)
                  -> any::TypeId
                {
                    any::TypeId::of::<Self>()
                }
            }
        }

        stackbox!(let it = ());
        let it: StackBoxDynUselessAny<'_> = it.into_dyn();
        assert_eq!(it.type_id(), any::TypeId::of::<()>());
        assert_eq!(it.type_id(), any::TypeId::of::<()>());
    }
}

macro_rules! compile_fail {(#[doc = $doc:expr] $item:item) => (#[doc = $doc] $item); (
    #![name = $name:ident]
    $($code:tt)*
) => (
    compile_fail! {
        #[doc = concat!(
            "```rust,",
                "compile_fail", /* Comment to show the error messages */
            "\n",
            stringify! {
                use ::stackbox::prelude::*;

                fn main ()
                {
                    fn main ()
                    {}

                    {
                        main();

                        $($code)*
                    }
                }
            }, "\n",
            "```", "\n",
        )]
        pub mod $name {}
    }
)}
use compile_fail;
