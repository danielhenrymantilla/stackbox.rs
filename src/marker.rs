mod sealed {
    use super::*;

    pub
    trait RepresentsAutoTraits {}

    impl RepresentsAutoTraits for dyn Send + 'static {}
    impl RepresentsAutoTraits for dyn Sync + 'static {}
    impl RepresentsAutoTraits for dyn Send + Sync + 'static {}
    impl RepresentsAutoTraits for NoAutoTraits {}
}

pub
struct NoAutoTraits (
    PhantomNotSendNorSync,
    ::core::convert::Infallible,
);

type PhantomNotSendNorSync =
    ::core::marker::PhantomData<*mut ()>
;

// struct PhantomNotSend(PhantomNotSendNorSync);
// unsafe // Safety: no API whatsoever.
//     impl Sync for PhantomNotSend {}

// type PhantomNotSync =
//     ::core::marker::PhantomData<&'static ::core::cell::Cell<u8>>
// ;

/// This is a "fake type-level `enum`" to hint at the generated documentation
/// what the purpose of some type params is.
///
/// ```rust
/// # macro_rules! ignore {($($t:tt)*) => ()} ignore! {
/// #[type_level_enum]
/// pub enum Sendness {
///     /// `+ ?Send` (default)
///     MaybeNotSend,
///     /// `+ Send
///     dyn Send,
/// }
/// # } fn main () {}
/// ```
#[doc(hidden)] #[allow(nonstandard_style)]
pub
mod Sendness {
    use super::*;

    pub
    trait T : sealed::RepresentsAutoTraits {}

    impl T for dyn Send {}
    impl T for dyn Sync {}
    impl T for dyn Send + Sync {}
    impl T for NoAutoTraits {}
}

/// This is a "fake type-level `enum`" to hint at the generated documentation
/// what the purpose of some type params is.
///
/// ```rust
/// # macro_rules! ignore {($($t:tt)*) => ()} ignore! {
/// #[type_level_enum]
/// pub enum Syncness {
///     /// `+ ?Sync` (default)
///     MaybeNotSync,
///     /// `+ Sync
///     dyn Sync,
/// }
/// # } fn main () {}
/// ```
#[doc(hidden)] #[allow(nonstandard_style)]
pub
mod Syncness {
    use super::*;

    pub
    trait T : sealed::RepresentsAutoTraits {}

    impl T for dyn Send {}
    impl T for dyn Sync {}
    impl T for dyn Send + Sync {}
    impl T for NoAutoTraits {}
}
