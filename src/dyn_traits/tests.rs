use super::*;

use ::core::ops::Not as _;

mod any {
    use super::*;

    #[test]
    fn coerce_unsync_unsend_into_any ()
    {
        stackbox!(::core::ptr::null::<()>() => let mut stackbox);
        let mut dyn_any: StackBoxDynAny<'_> = stackbox.coerce_into_dyn();
        assert!(dyn_any.is::<*const ()>());
        assert!(dyn_any.is::<bool>().not());
        let &(_): &'_ (*const ()) = dyn_any.downcast_ref().unwrap();
        let &mut(_): &'_ mut (*const ()) = dyn_any.downcast_mut().unwrap();
        stackbox = dyn_any.downcast().unwrap();
        drop(stackbox);
    }

    #[test]
    fn coerce_sync_unsend_into_sync_any ()
    {
        #[derive(Default)]
        struct PhantomUnsend(::core::marker::PhantomData<*mut ()>);
        unsafe impl Sync for PhantomUnsend {}

        stackbox!(PhantomUnsend::default() => let stackbox);
        let _: StackBoxDynAny<'_, dyn Sync> = stackbox.coerce_into_dyn();
    }

    #[test]
    fn coerce_send_unsync_into_send_any ()
    {
        stackbox!(::core::cell::Cell::new(0_u8) => let stackbox);
        let _: StackBoxDynAny<'_, dyn Send> = stackbox.coerce_into_dyn();
    }

    #[test]
    fn coerce_send_sync_into_send_sync_any ()
    {
        stackbox!(() => let stackbox);
        let _: StackBoxDynAny<'_, dyn Send + Sync> = stackbox.coerce_into_dyn();
    }

    #[test]
    fn test_drops ()
    {
        let rc = ::std::rc::Rc::new(());
        let count = || ::std::rc::Rc::strong_count(&rc);
        let rc = || rc.clone();
        let mut stackbox;

        stackbox!(rc() => stackbox);
        assert_eq!(count(), 2);
        drop(stackbox);
        assert_eq!(count(), 1);

        stackbox!(rc() => stackbox);
        assert_eq!(count(), 2);
        let rc2 = StackBox::into_inner(stackbox);
        assert_eq!(count(), 2);
        drop(rc2);
        assert_eq!(count(), 1);

        stackbox!(rc() => stackbox);
        assert_eq!(count(), 2);
        let dyn_any: StackBoxDynAny<'_> = stackbox.coerce_into_dyn();
        assert_eq!(count(), 2);
        drop(dyn_any);
        assert_eq!(count(), 1);

        stackbox!(rc() => stackbox);
        assert_eq!(count(), 2);
        let dyn_any: StackBoxDynAny<'_> = stackbox.coerce_into_dyn();
        assert_eq!(count(), 2);
        stackbox = dyn_any.downcast().unwrap();
        assert_eq!(count(), 2);
        drop(stackbox);
        assert_eq!(count(), 1);
    }

    compile_fail! {
        #![name = cannot_coerce_unsync_into_sync_any]

        stackbox!(::core::ptr::null::<()>() => let stackbox);
        let _: StackBoxDynAny<'_, dyn Sync> = stackbox.coerce_into_dyn();
    }

    compile_fail! {
        #![name = cannot_coerce_unsync_into_sync_send_any]

        stackbox!(::core::ptr::null::<()>() => let stackbox);
        let _: StackBoxDynAny<'_, dyn Send + Sync> = stackbox.coerce_into_dyn();
    }
}

mod fn_once {
    use super::*;

    #[test]
    fn move_semantics ()
    {
        let not_copy: [&'static mut (); 0] = [];
        stackbox!(|| drop(not_copy) => let stackbox_fn_once);
        let mut dyn_fn_once: StackBoxDynFnOnce_0<'_, ()> = stackbox_fn_once.coerce_into_dyn();
        dyn_fn_once.call();
        let not_copy: [&'static mut (); 0] = [];
        stackbox!(|| drop(not_copy) => let stackbox_fn_once);
        dyn_fn_once = stackbox_fn_once.coerce_into_dyn();
        drop(dyn_fn_once);
    }

    compile_fail! {
        #![name = move_semantics_2]

        let mut not_copy: [&'static mut (); 0] = [];
        stackbox!(|| drop(not_copy) => let stackbox_fn_once);
        let mut dyn_fn_once: StackBoxDynFnOnce_0<'_, ()> = stackbox_fn_once.coerce_into_dyn();
        drop({ "use of moved value"; stackbox_fn_once });
        dyn_fn_once.call();
        drop({ "use of moved value"; dyn_fn_once });
    }

    #[test]
    fn test_drops ()
    {
        let rc = ::std::rc::Rc::new(());
        let count = || ::std::rc::Rc::strong_count(&rc);
        let rc = || { let rc = rc.clone(); move || drop(rc) };
        let mut stackbox;

        stackbox!(rc() => stackbox);
        assert_eq!(count(), 2);
        drop(stackbox);
        assert_eq!(count(), 1);

        stackbox!(rc() => stackbox);
        assert_eq!(count(), 2);
        stackbox.into_inner()();
        assert_eq!(count(), 1);

        stackbox!(rc() => stackbox);
        assert_eq!(count(), 2);
        let f = StackBox::into_inner(stackbox);
        assert_eq!(count(), 2);
        drop(f);
        assert_eq!(count(), 1);

        stackbox!(rc() => stackbox);
        assert_eq!(count(), 2);
        let f = StackBox::into_inner(stackbox);
        assert_eq!(count(), 2);
        f();
        assert_eq!(count(), 1);

        stackbox!(rc() => stackbox);
        assert_eq!(count(), 2);
        let dyn_fn: StackBoxDynFnOnce_0<'_, ()> = stackbox.coerce_into_dyn();
        assert_eq!(count(), 2);
        drop(dyn_fn);
        assert_eq!(count(), 1);

        stackbox!(rc() => stackbox);
        assert_eq!(count(), 2);
        let dyn_fn: StackBoxDynFnOnce_0<'_, ()> = stackbox.coerce_into_dyn();
        assert_eq!(count(), 2);
        dyn_fn.call();
        assert_eq!(count(), 1);
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
)} use compile_fail;
