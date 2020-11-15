#[cfg(doc)]
use crate::StackBox;

/**
Helper macro to define custom `StackBox<dyn …>` trait objects.

Mainly useful for [`FnOnce`], **when higher-order lifetimes** are needed.

<details><summary><b>What are higher-order lifetimes?</b></summary>

Consider the following example: you are able to create some local value within
a function, and want to call some provided callback on a _borrow_ of it.

```rust
# use ::core::cell::RefCell;
#
fn with_nested_refcell (
    nested: &'_ RefCell<RefCell<str>>,
    f: fn(&'_ str),
)
{
    f(&*nested.borrow().borrow())
}
```

The main question then is:

> what is the lifetime of that borrow?

 1. If we try to unsugar each elided lifetime (`'_`) with a lifetime parameter
    on the function, the code no longer compiles:

    ```rust,compile_fail
    # use ::core::cell::RefCell;
    #
    fn with_nested_refcell<'nested, 'arg> (
        nested: &'nested RefCell<RefCell<str>>,
        f: fn(&'arg str),
    )
    {
        f(&*nested.borrow().borrow())
    }
    ```

    <details><summary>Error message</summary>

    ```text
    error[E0716]: temporary value dropped while borrowed
     --> src/lib.rs:8:9
      |
    3 | fn with_nested_refcell<'nested, 'arg> (
      |                                 ---- lifetime `'arg` defined here
    ...
    8 |     f(&*nested.borrow().borrow())
      |         ^^^^^^^^^^^^^^^---------
      |         |
      |         creates a temporary which is freed while still in use
      |         argument requires that borrow lasts for `'arg`
    9 | }
      | - temporary value is freed at the end of this statement

    error[E0716]: temporary value dropped while borrowed
     --> src/lib.rs:8:9
      |
    3 | fn with_nested_refcell<'nested, 'arg> (
      |                                 ---- lifetime `'arg` defined here
    ...
    8 |     f(&*nested.borrow().borrow())
      |     ----^^^^^^^^^^^^^^^^^^^^^^^^-
      |     |   |
      |     |   creates a temporary which is freed while still in use
      |     argument requires that borrow lasts for `'arg`
    9 | }
      | - temporary value is freed at the end of this statement
    ```

    ___

    </details>

 1. So Rust considers that **generic lifetime parameters represent
    lifetimes that span beyond the end of a function's body** (_c.f._ the
    previous error message).

    Or in other words, since generic parameters, including generic lifetime
    parameters, are _chosen by the caller_, and since the body of a callee
    is opaque to the caller, it is impossible for the caller to provide /
    choose a lifetime that is able to match an internal scope.

 1. Because of that, in order for a callback to be able to use a borrow with
    some existential but unnameable lifetime, only one solution remains:

    > **That the caller provide a callback able to handle _all_ the lifetimes /
    _any lifetime possible_**.

      - Imaginary syntax:

        ```rust
        # macro_rules! ignore {($($t:tt)*) => ()} ignore! {
        fn with_nested_refcell<'nested> (
            nested: &'nested RefCell<RefCell<str>>,
            f: fn<'arg>(&'arg str),
        )
        {
            f(&*nested.borrow().borrow())
        }
        # } fn main () {}
        ```

        the difference then being:

        <!--

        ```diff
        - fn with_nested_refcell<'nested, 'arg> (
        + fn with_nested_refcell<'nested> (
              nested: &'nested RefCell<RefCell<str>>,
        -     f: fn(&'arg str),
        +     f: fn<'arg>(&'arg str),
          )
        ```

        -->

        <html>
        <head>
        <meta http-equiv="Content-Type" content="text/html; charset=utf-8"/>
        <style type="text/css">
        pre { white-space: pre-wrap; }
        .ef0,.f0 { color: #000000; } .eb0,.b0 { background-color: #000000; }
        .ef1,.f1 { color: #AA0000; } .eb1,.b1 { background-color: #AA0000; }
        .ef2,.f2 { color: #00AA00; } .eb2,.b2 { background-color: #00AA00; }
        .ef3,.f3 { color: #AA5500; } .eb3,.b3 { background-color: #AA5500; }
        .ef4,.f4 { color: #0000AA; } .eb4,.b4 { background-color: #0000AA; }
        .ef5,.f5 { color: #AA00AA; } .eb5,.b5 { background-color: #AA00AA; }
        .ef6,.f6 { color: #00AAAA; } .eb6,.b6 { background-color: #00AAAA; }
        .ef7,.f7 { color: #AAAAAA; } .eb7,.b7 { background-color: #AAAAAA; }
        .ef8, .f0 > .bold,.bold > .f0 { color: #555555; font-weight: normal; }
        .ef9, .f1 > .bold,.bold > .f1 { color: #FF5555; font-weight: normal; }
        .ef10,.f2 > .bold,.bold > .f2 { color: #55FF55; font-weight: normal; }
        .ef11,.f3 > .bold,.bold > .f3 { color: #FFFF55; font-weight: normal; }
        .ef12,.f4 > .bold,.bold > .f4 { color: #5555FF; font-weight: normal; }
        .ef13,.f5 > .bold,.bold > .f5 { color: #FF55FF; font-weight: normal; }
        .ef14,.f6 > .bold,.bold > .f6 { color: #55FFFF; font-weight: normal; }
        .ef15,.f7 > .bold,.bold > .f7 { color: #FFFFFF; font-weight: normal; }
        .eb8  { background-color: #555555; }
        .eb9  { background-color: #FF5555; }
        .eb10 { background-color: #55FF55; }
        .eb11 { background-color: #FFFF55; }
        .eb12 { background-color: #5555FF; }
        .eb13 { background-color: #FF55FF; }
        .eb14 { background-color: #55FFFF; }
        .eb15 { background-color: #FFFFFF; }
        .ef16 { color: #000000; } .eb16 { background-color: #000000; }
        .ef17 { color: #00005f; } .eb17 { background-color: #00005f; }
        .ef18 { color: #000087; } .eb18 { background-color: #000087; }
        .ef19 { color: #0000af; } .eb19 { background-color: #0000af; }
        .ef20 { color: #0000d7; } .eb20 { background-color: #0000d7; }
        .ef21 { color: #0000ff; } .eb21 { background-color: #0000ff; }
        .ef22 { color: #005f00; } .eb22 { background-color: #005f00; }
        .ef23 { color: #005f5f; } .eb23 { background-color: #005f5f; }
        .ef24 { color: #005f87; } .eb24 { background-color: #005f87; }
        .ef25 { color: #005faf; } .eb25 { background-color: #005faf; }
        .ef26 { color: #005fd7; } .eb26 { background-color: #005fd7; }
        .ef27 { color: #005fff; } .eb27 { background-color: #005fff; }
        .ef28 { color: #008700; } .eb28 { background-color: #008700; }
        .ef29 { color: #00875f; } .eb29 { background-color: #00875f; }
        .ef30 { color: #008787; } .eb30 { background-color: #008787; }
        .ef31 { color: #0087af; } .eb31 { background-color: #0087af; }
        .ef32 { color: #0087d7; } .eb32 { background-color: #0087d7; }
        .ef33 { color: #0087ff; } .eb33 { background-color: #0087ff; }
        .ef34 { color: #00af00; } .eb34 { background-color: #00af00; }
        .ef35 { color: #00af5f; } .eb35 { background-color: #00af5f; }
        .ef36 { color: #00af87; } .eb36 { background-color: #00af87; }
        .ef37 { color: #00afaf; } .eb37 { background-color: #00afaf; }
        .ef38 { color: #00afd7; } .eb38 { background-color: #00afd7; }
        .ef39 { color: #00afff; } .eb39 { background-color: #00afff; }
        .ef40 { color: #00d700; } .eb40 { background-color: #00d700; }
        .ef41 { color: #00d75f; } .eb41 { background-color: #00d75f; }
        .ef42 { color: #00d787; } .eb42 { background-color: #00d787; }
        .ef43 { color: #00d7af; } .eb43 { background-color: #00d7af; }
        .ef44 { color: #00d7d7; } .eb44 { background-color: #00d7d7; }
        .ef45 { color: #00d7ff; } .eb45 { background-color: #00d7ff; }
        .ef46 { color: #00ff00; } .eb46 { background-color: #00ff00; }
        .ef47 { color: #00ff5f; } .eb47 { background-color: #00ff5f; }
        .ef48 { color: #00ff87; } .eb48 { background-color: #00ff87; }
        .ef49 { color: #00ffaf; } .eb49 { background-color: #00ffaf; }
        .ef50 { color: #00ffd7; } .eb50 { background-color: #00ffd7; }
        .ef51 { color: #00ffff; } .eb51 { background-color: #00ffff; }
        .ef52 { color: #5f0000; } .eb52 { background-color: #5f0000; }
        .ef53 { color: #5f005f; } .eb53 { background-color: #5f005f; }
        .ef54 { color: #5f0087; } .eb54 { background-color: #5f0087; }
        .ef55 { color: #5f00af; } .eb55 { background-color: #5f00af; }
        .ef56 { color: #5f00d7; } .eb56 { background-color: #5f00d7; }
        .ef57 { color: #5f00ff; } .eb57 { background-color: #5f00ff; }
        .ef58 { color: #5f5f00; } .eb58 { background-color: #5f5f00; }
        .ef59 { color: #5f5f5f; } .eb59 { background-color: #5f5f5f; }
        .ef60 { color: #5f5f87; } .eb60 { background-color: #5f5f87; }
        .ef61 { color: #5f5faf; } .eb61 { background-color: #5f5faf; }
        .ef62 { color: #5f5fd7; } .eb62 { background-color: #5f5fd7; }
        .ef63 { color: #5f5fff; } .eb63 { background-color: #5f5fff; }
        .ef64 { color: #5f8700; } .eb64 { background-color: #5f8700; }
        .ef65 { color: #5f875f; } .eb65 { background-color: #5f875f; }
        .ef66 { color: #5f8787; } .eb66 { background-color: #5f8787; }
        .ef67 { color: #5f87af; } .eb67 { background-color: #5f87af; }
        .ef68 { color: #5f87d7; } .eb68 { background-color: #5f87d7; }
        .ef69 { color: #5f87ff; } .eb69 { background-color: #5f87ff; }
        .ef70 { color: #5faf00; } .eb70 { background-color: #5faf00; }
        .ef71 { color: #5faf5f; } .eb71 { background-color: #5faf5f; }
        .ef72 { color: #5faf87; } .eb72 { background-color: #5faf87; }
        .ef73 { color: #5fafaf; } .eb73 { background-color: #5fafaf; }
        .ef74 { color: #5fafd7; } .eb74 { background-color: #5fafd7; }
        .ef75 { color: #5fafff; } .eb75 { background-color: #5fafff; }
        .ef76 { color: #5fd700; } .eb76 { background-color: #5fd700; }
        .ef77 { color: #5fd75f; } .eb77 { background-color: #5fd75f; }
        .ef78 { color: #5fd787; } .eb78 { background-color: #5fd787; }
        .ef79 { color: #5fd7af; } .eb79 { background-color: #5fd7af; }
        .ef80 { color: #5fd7d7; } .eb80 { background-color: #5fd7d7; }
        .ef81 { color: #5fd7ff; } .eb81 { background-color: #5fd7ff; }
        .ef82 { color: #5fff00; } .eb82 { background-color: #5fff00; }
        .ef83 { color: #5fff5f; } .eb83 { background-color: #5fff5f; }
        .ef84 { color: #5fff87; } .eb84 { background-color: #5fff87; }
        .ef85 { color: #5fffaf; } .eb85 { background-color: #5fffaf; }
        .ef86 { color: #5fffd7; } .eb86 { background-color: #5fffd7; }
        .ef87 { color: #5fffff; } .eb87 { background-color: #5fffff; }
        .ef88 { color: #870000; } .eb88 { background-color: #870000; }
        .ef89 { color: #87005f; } .eb89 { background-color: #87005f; }
        .ef90 { color: #870087; } .eb90 { background-color: #870087; }
        .ef91 { color: #8700af; } .eb91 { background-color: #8700af; }
        .ef92 { color: #8700d7; } .eb92 { background-color: #8700d7; }
        .ef93 { color: #8700ff; } .eb93 { background-color: #8700ff; }
        .ef94 { color: #875f00; } .eb94 { background-color: #875f00; }
        .ef95 { color: #875f5f; } .eb95 { background-color: #875f5f; }
        .ef96 { color: #875f87; } .eb96 { background-color: #875f87; }
        .ef97 { color: #875faf; } .eb97 { background-color: #875faf; }
        .ef98 { color: #875fd7; } .eb98 { background-color: #875fd7; }
        .ef99 { color: #875fff; } .eb99 { background-color: #875fff; }
        .ef100 { color: #878700; } .eb100 { background-color: #878700; }
        .ef101 { color: #87875f; } .eb101 { background-color: #87875f; }
        .ef102 { color: #878787; } .eb102 { background-color: #878787; }
        .ef103 { color: #8787af; } .eb103 { background-color: #8787af; }
        .ef104 { color: #8787d7; } .eb104 { background-color: #8787d7; }
        .ef105 { color: #8787ff; } .eb105 { background-color: #8787ff; }
        .ef106 { color: #87af00; } .eb106 { background-color: #87af00; }
        .ef107 { color: #87af5f; } .eb107 { background-color: #87af5f; }
        .ef108 { color: #87af87; } .eb108 { background-color: #87af87; }
        .ef109 { color: #87afaf; } .eb109 { background-color: #87afaf; }
        .ef110 { color: #87afd7; } .eb110 { background-color: #87afd7; }
        .ef111 { color: #87afff; } .eb111 { background-color: #87afff; }
        .ef112 { color: #87d700; } .eb112 { background-color: #87d700; }
        .ef113 { color: #87d75f; } .eb113 { background-color: #87d75f; }
        .ef114 { color: #87d787; } .eb114 { background-color: #87d787; }
        .ef115 { color: #87d7af; } .eb115 { background-color: #87d7af; }
        .ef116 { color: #87d7d7; } .eb116 { background-color: #87d7d7; }
        .ef117 { color: #87d7ff; } .eb117 { background-color: #87d7ff; }
        .ef118 { color: #87ff00; } .eb118 { background-color: #87ff00; }
        .ef119 { color: #87ff5f; } .eb119 { background-color: #87ff5f; }
        .ef120 { color: #87ff87; } .eb120 { background-color: #87ff87; }
        .ef121 { color: #87ffaf; } .eb121 { background-color: #87ffaf; }
        .ef122 { color: #87ffd7; } .eb122 { background-color: #87ffd7; }
        .ef123 { color: #87ffff; } .eb123 { background-color: #87ffff; }
        .ef124 { color: #af0000; } .eb124 { background-color: #af0000; }
        .ef125 { color: #af005f; } .eb125 { background-color: #af005f; }
        .ef126 { color: #af0087; } .eb126 { background-color: #af0087; }
        .ef127 { color: #af00af; } .eb127 { background-color: #af00af; }
        .ef128 { color: #af00d7; } .eb128 { background-color: #af00d7; }
        .ef129 { color: #af00ff; } .eb129 { background-color: #af00ff; }
        .ef130 { color: #af5f00; } .eb130 { background-color: #af5f00; }
        .ef131 { color: #af5f5f; } .eb131 { background-color: #af5f5f; }
        .ef132 { color: #af5f87; } .eb132 { background-color: #af5f87; }
        .ef133 { color: #af5faf; } .eb133 { background-color: #af5faf; }
        .ef134 { color: #af5fd7; } .eb134 { background-color: #af5fd7; }
        .ef135 { color: #af5fff; } .eb135 { background-color: #af5fff; }
        .ef136 { color: #af8700; } .eb136 { background-color: #af8700; }
        .ef137 { color: #af875f; } .eb137 { background-color: #af875f; }
        .ef138 { color: #af8787; } .eb138 { background-color: #af8787; }
        .ef139 { color: #af87af; } .eb139 { background-color: #af87af; }
        .ef140 { color: #af87d7; } .eb140 { background-color: #af87d7; }
        .ef141 { color: #af87ff; } .eb141 { background-color: #af87ff; }
        .ef142 { color: #afaf00; } .eb142 { background-color: #afaf00; }
        .ef143 { color: #afaf5f; } .eb143 { background-color: #afaf5f; }
        .ef144 { color: #afaf87; } .eb144 { background-color: #afaf87; }
        .ef145 { color: #afafaf; } .eb145 { background-color: #afafaf; }
        .ef146 { color: #afafd7; } .eb146 { background-color: #afafd7; }
        .ef147 { color: #afafff; } .eb147 { background-color: #afafff; }
        .ef148 { color: #afd700; } .eb148 { background-color: #afd700; }
        .ef149 { color: #afd75f; } .eb149 { background-color: #afd75f; }
        .ef150 { color: #afd787; } .eb150 { background-color: #afd787; }
        .ef151 { color: #afd7af; } .eb151 { background-color: #afd7af; }
        .ef152 { color: #afd7d7; } .eb152 { background-color: #afd7d7; }
        .ef153 { color: #afd7ff; } .eb153 { background-color: #afd7ff; }
        .ef154 { color: #afff00; } .eb154 { background-color: #afff00; }
        .ef155 { color: #afff5f; } .eb155 { background-color: #afff5f; }
        .ef156 { color: #afff87; } .eb156 { background-color: #afff87; }
        .ef157 { color: #afffaf; } .eb157 { background-color: #afffaf; }
        .ef158 { color: #afffd7; } .eb158 { background-color: #afffd7; }
        .ef159 { color: #afffff; } .eb159 { background-color: #afffff; }
        .ef160 { color: #d70000; } .eb160 { background-color: #d70000; }
        .ef161 { color: #d7005f; } .eb161 { background-color: #d7005f; }
        .ef162 { color: #d70087; } .eb162 { background-color: #d70087; }
        .ef163 { color: #d700af; } .eb163 { background-color: #d700af; }
        .ef164 { color: #d700d7; } .eb164 { background-color: #d700d7; }
        .ef165 { color: #d700ff; } .eb165 { background-color: #d700ff; }
        .ef166 { color: #d75f00; } .eb166 { background-color: #d75f00; }
        .ef167 { color: #d75f5f; } .eb167 { background-color: #d75f5f; }
        .ef168 { color: #d75f87; } .eb168 { background-color: #d75f87; }
        .ef169 { color: #d75faf; } .eb169 { background-color: #d75faf; }
        .ef170 { color: #d75fd7; } .eb170 { background-color: #d75fd7; }
        .ef171 { color: #d75fff; } .eb171 { background-color: #d75fff; }
        .ef172 { color: #d78700; } .eb172 { background-color: #d78700; }
        .ef173 { color: #d7875f; } .eb173 { background-color: #d7875f; }
        .ef174 { color: #d78787; } .eb174 { background-color: #d78787; }
        .ef175 { color: #d787af; } .eb175 { background-color: #d787af; }
        .ef176 { color: #d787d7; } .eb176 { background-color: #d787d7; }
        .ef177 { color: #d787ff; } .eb177 { background-color: #d787ff; }
        .ef178 { color: #d7af00; } .eb178 { background-color: #d7af00; }
        .ef179 { color: #d7af5f; } .eb179 { background-color: #d7af5f; }
        .ef180 { color: #d7af87; } .eb180 { background-color: #d7af87; }
        .ef181 { color: #d7afaf; } .eb181 { background-color: #d7afaf; }
        .ef182 { color: #d7afd7; } .eb182 { background-color: #d7afd7; }
        .ef183 { color: #d7afff; } .eb183 { background-color: #d7afff; }
        .ef184 { color: #d7d700; } .eb184 { background-color: #d7d700; }
        .ef185 { color: #d7d75f; } .eb185 { background-color: #d7d75f; }
        .ef186 { color: #d7d787; } .eb186 { background-color: #d7d787; }
        .ef187 { color: #d7d7af; } .eb187 { background-color: #d7d7af; }
        .ef188 { color: #d7d7d7; } .eb188 { background-color: #d7d7d7; }
        .ef189 { color: #d7d7ff; } .eb189 { background-color: #d7d7ff; }
        .ef190 { color: #d7ff00; } .eb190 { background-color: #d7ff00; }
        .ef191 { color: #d7ff5f; } .eb191 { background-color: #d7ff5f; }
        .ef192 { color: #d7ff87; } .eb192 { background-color: #d7ff87; }
        .ef193 { color: #d7ffaf; } .eb193 { background-color: #d7ffaf; }
        .ef194 { color: #d7ffd7; } .eb194 { background-color: #d7ffd7; }
        .ef195 { color: #d7ffff; } .eb195 { background-color: #d7ffff; }
        .ef196 { color: #ff0000; } .eb196 { background-color: #ff0000; }
        .ef197 { color: #ff005f; } .eb197 { background-color: #ff005f; }
        .ef198 { color: #ff0087; } .eb198 { background-color: #ff0087; }
        .ef199 { color: #ff00af; } .eb199 { background-color: #ff00af; }
        .ef200 { color: #ff00d7; } .eb200 { background-color: #ff00d7; }
        .ef201 { color: #ff00ff; } .eb201 { background-color: #ff00ff; }
        .ef202 { color: #ff5f00; } .eb202 { background-color: #ff5f00; }
        .ef203 { color: #ff5f5f; } .eb203 { background-color: #ff5f5f; }
        .ef204 { color: #ff5f87; } .eb204 { background-color: #ff5f87; }
        .ef205 { color: #ff5faf; } .eb205 { background-color: #ff5faf; }
        .ef206 { color: #ff5fd7; } .eb206 { background-color: #ff5fd7; }
        .ef207 { color: #ff5fff; } .eb207 { background-color: #ff5fff; }
        .ef208 { color: #ff8700; } .eb208 { background-color: #ff8700; }
        .ef209 { color: #ff875f; } .eb209 { background-color: #ff875f; }
        .ef210 { color: #ff8787; } .eb210 { background-color: #ff8787; }
        .ef211 { color: #ff87af; } .eb211 { background-color: #ff87af; }
        .ef212 { color: #ff87d7; } .eb212 { background-color: #ff87d7; }
        .ef213 { color: #ff87ff; } .eb213 { background-color: #ff87ff; }
        .ef214 { color: #ffaf00; } .eb214 { background-color: #ffaf00; }
        .ef215 { color: #ffaf5f; } .eb215 { background-color: #ffaf5f; }
        .ef216 { color: #ffaf87; } .eb216 { background-color: #ffaf87; }
        .ef217 { color: #ffafaf; } .eb217 { background-color: #ffafaf; }
        .ef218 { color: #ffafd7; } .eb218 { background-color: #ffafd7; }
        .ef219 { color: #ffafff; } .eb219 { background-color: #ffafff; }
        .ef220 { color: #ffd700; } .eb220 { background-color: #ffd700; }
        .ef221 { color: #ffd75f; } .eb221 { background-color: #ffd75f; }
        .ef222 { color: #ffd787; } .eb222 { background-color: #ffd787; }
        .ef223 { color: #ffd7af; } .eb223 { background-color: #ffd7af; }
        .ef224 { color: #ffd7d7; } .eb224 { background-color: #ffd7d7; }
        .ef225 { color: #ffd7ff; } .eb225 { background-color: #ffd7ff; }
        .ef226 { color: #ffff00; } .eb226 { background-color: #ffff00; }
        .ef227 { color: #ffff5f; } .eb227 { background-color: #ffff5f; }
        .ef228 { color: #ffff87; } .eb228 { background-color: #ffff87; }
        .ef229 { color: #ffffaf; } .eb229 { background-color: #ffffaf; }
        .ef230 { color: #ffffd7; } .eb230 { background-color: #ffffd7; }
        .ef231 { color: #ffffff; } .eb231 { background-color: #ffffff; }
        .ef232 { color: #080808; } .eb232 { background-color: #080808; }
        .ef233 { color: #121212; } .eb233 { background-color: #121212; }
        .ef234 { color: #1c1c1c; } .eb234 { background-color: #1c1c1c; }
        .ef235 { color: #262626; } .eb235 { background-color: #262626; }
        .ef236 { color: #303030; } .eb236 { background-color: #303030; }
        .ef237 { color: #3a3a3a; } .eb237 { background-color: #3a3a3a; }
        .ef238 { color: #444444; } .eb238 { background-color: #444444; }
        .ef239 { color: #4e4e4e; } .eb239 { background-color: #4e4e4e; }
        .ef240 { color: #585858; } .eb240 { background-color: #585858; }
        .ef241 { color: #626262; } .eb241 { background-color: #626262; }
        .ef242 { color: #6c6c6c; } .eb242 { background-color: #6c6c6c; }
        .ef243 { color: #767676; } .eb243 { background-color: #767676; }
        .ef244 { color: #808080; } .eb244 { background-color: #808080; }
        .ef245 { color: #8a8a8a; } .eb245 { background-color: #8a8a8a; }
        .ef246 { color: #949494; } .eb246 { background-color: #949494; }
        .ef247 { color: #9e9e9e; } .eb247 { background-color: #9e9e9e; }
        .ef248 { color: #a8a8a8; } .eb248 { background-color: #a8a8a8; }
        .ef249 { color: #b2b2b2; } .eb249 { background-color: #b2b2b2; }
        .ef250 { color: #bcbcbc; } .eb250 { background-color: #bcbcbc; }
        .ef251 { color: #c6c6c6; } .eb251 { background-color: #c6c6c6; }
        .ef252 { color: #d0d0d0; } .eb252 { background-color: #d0d0d0; }
        .ef253 { color: #dadada; } .eb253 { background-color: #dadada; }
        .ef254 { color: #e4e4e4; } .eb254 { background-color: #e4e4e4; }
        .ef255 { color: #eeeeee; } .eb255 { background-color: #eeeeee; }

        .f9 { color: #000000; }
        .b9 { background-color: #FFFFFF; }
        .f9 > .bold,.bold > .f9, body.f9 > pre > .bold {
        /* Bold is heavy black on white, or bright white
            depending on the default background */
        color: #000000;
        font-weight: bold;
        }
        .reverse {
        /* CSS does not support swapping fg and bg colours unfortunately,
            so just hardcode something that will look OK on all backgrounds. */
        color: #000000; background-color: #AAAAAA;
        }
        .underline { text-decoration: underline; }
        .line-through { text-decoration: line-through; }
        .blink { text-decoration: blink; }

        /* Avoid pixels between adjacent span elements.
        Note this only works for lines less than 80 chars
        where we close span elements on the same line.
        span { display: inline-block; }
        */
        </style>
        </head>

        <body class="f9 b9">
        <pre>
        <span class="f1">- fn with_nested_refcell&lt;'nested, 'arg&gt; (</span>
        <span class="f2">+ fn with_nested_refcell&lt;'nested&gt; (</span>
              nested: &amp;'nested RefCell&lt;RefCell&lt;str&gt;&gt;,
        <span class="f1">-     f: fn(&amp;'arg str),</span>
        <span class="f2">+     f: fn&lt;'arg&gt;(&amp;'arg str),</span>
          )
        </pre>
        </body>
        </html>


This is actually a real property with real syntax that can be expressed in
Rust, and thus that Rust can check, and is precisely the mechanism that enables
to have closures (and other objects involving custom traits with lifetimes,
such as [`Deserialize<'de>`]( https://serde.rs/lifetimes.html)) operate on
borrow over callee locals.

The real syntax for this, is:

```rust
# use ::core::cell::RefCell;
#
fn with_nested_refcell<'nested> (
    nested: &'nested RefCell<RefCell<str>>,
    f: for<'arg> fn(&'arg str), // ≈ fn<'arg> (&'arg str),
)
{
    f(&*nested.borrow().borrow())
}
```

  - For function pointer types (`fn`):

    ```rust
    # macro_rules! ignore {($($t:tt)*) => ()} ignore! {
    for<'lifetimes...> fn(args...) -> Ret
    # } fn main () {}
    ```

  - For trait bounds:

    ```rust
    # macro_rules! ignore {($($t:tt)*) => ()} ignore! {
    for<'lifetimes...>
        Type : Bounds
    ,
    // e.g.,
    for<'arg>
        F : Fn(&'arg str)
    ,
    # } fn main () {}
    ```

    or:

    ```rust
    # macro_rules! ignore {($($t:tt)*) => ()} ignore! {
    Type : for<'lifetimes...> Bounds
    // e.g.,
    F : for<'arg> Fn(&'arg str)
    # } fn main () {}
    ```

Back to the `::serde::Deserialize` example, it can be interesting to
observe that `DeserializeOwned` is defined as the following simple trait alias:

```rust
# macro_rules! ignore {($($t:tt)*) => ()} ignore! {
DeserializeOwned = for<'any> Deserialize<'any>
# } fn main () {}
```

> **This whole thing is called Higher-Rank Trait Bounds ([HRTB]) or
> higher-order lifetimes.**

[HRTB]: https://doc.rust-lang.org/nomicon/hrtb.html

___

Ok, time to go back to the question at hand, that of using the [`custom_dyn!`]
macro in the context of [`StackBox`]es:

</details>

So the problem is that the currently defined types and impls in this crate do
not support higher-order lifetimes. Indeed, although it is possible to write
an impl that supports a specific higher-order signature, it is currently
impossible in Rust to write generic code that covers all the possible
higher-order signatures:

 1. For instance, given an `impl<A> Trait for fn(A) { … }`, we won't
    have `fn(&str)` implementing `Trait`, since
    `fn(&str) = for<'any> fn(&'any str) ≠ fn(A)`.

 1. And even if we wrote `impl<A : ?Sized> Trait for fn(&A) { … }`, we won't
    have `fn(&&str)` implementing `Trait`, since

    ```rust
    # macro_rules! ignore {($($t:tt)*) => ()} ignore! {
    fn(&&str) = for<'a, 'b> fn(&'a &'b str) != for<'c> fn(&'c A) = fn(&A)
    # } fn main () {}
    ```

That's where [`custom_dyn!`] comes into play:

  - I, the crate author, cannot know which higher-order signature(s) you, the
    user of the library, will be using, and sadly cannot cover that with a
    generic impl.

  - But since _you_ know which kind of signature you need, I can "let you impl
    yourself". Sadly, the actual impl is complex and error-prone (involves
    `unsafe` and VTables!), so instead I provide you this handy macro that will
    take care of all the nitty-gritty details for you.

In a way, since I am hitting a limitation of the too-early-type-checked Rust
metaprogramming tool (generics), I am falling back to the duck-typed /
only-type-checked-when-instanced metaprogramming tool (macros), thus acting as
a C++ template of sorts, we could say 😄

## Example: `dyn FnOnce(&str) = dyn for<'any> FnOnce(&'any str)`

The following example fails to compile:

```rust,compile_fail
use ::stackbox::prelude::*;

//                       `f: StackBox<dyn FnOnce(&'arg str) -> ()>`
fn call_with_local<'arg> (f: StackBoxDynFnOnce_1<&'arg str, ()>)
{
    let local = format!("...");
    f.call(&local)
}

fn main ()
{
    stackbox!(let f = |_: &str| ());
    call_with_local(f.into_dyn());
}
```

<details><summary>Error message</summary>

```text
error[E0597]: `local` does not live long enough
 --> src/some/file.rs:8:12
  |
5 | fn call_with_local<'arg> (f: StackBoxDynFnOnce_1<&'arg str, ()>)
  |                    ---- lifetime `'arg` defined here
...
8 |     f.call(&local)
  |     -------^^^^^^-
  |     |      |
  |     |      borrowed value does not live long enough
  |     argument requires that `local` is borrowed for `'arg`
9 | }
  | - `local` dropped here while still borrowed
```

This is exactly the same problem we had when I was explaining higher-order
signatures and we had `fn nested_refcells<'nested, 'arg>...`: the lifetime
of the parameter is an _outer_ (fixed) generic lifetime parameter, and it can
thus not work with a local / callee-specific lifetime.

___

</details>

The solution is to define a new `DynFnOnce...` trait, which involves a
higher-order lifetime in the signature:

```rust
use ::stackbox::prelude::*;

custom_dyn! {
    pub
    dyn FnOnceStr : FnOnce(&str) {
        fn call (self: Self, arg: &str)
        {
            self(arg)
        }
    }
}
//                 `f: StackBox<dyn FnOnce(&str) -> ()>`
fn call_with_local (f: StackBoxDynFnOnceStr)
{
    let local = format!("...");
    f.call(&local)
}

fn main ()
{
    stackbox!(let f = |_: &str| ());
    call_with_local(f.into_dyn());
}
```

*/
#[macro_export]
macro_rules! custom_dyn {(
    #![dollar = $__:tt]
    $( #[doc = $doc:expr] )*
    $pub:vis
    dyn $Trait:ident $(
        <
            $($lt:lifetime),* $(,)? $($T:ident),* $(,)?
        >
    )?
        : $super:path
    $(
        where { $($wc:tt)* }
    )?
    {
        $(
            fn $method:ident (
                $self:ident :
                    $(
                        & $($ref:lifetime)?
                        $(mut $(@$mut:tt)?)?
                    )?
                    Self
              $(,
                $arg_name:ident : $ArgTy:ty )*
                $(,)?
            ) $(-> $RetTy:ty)?
            {
                $($body:tt)*
            }
        )*
    }
) => ($crate::__::paste! {
    $( #[doc = $doc] )*
    $pub
    struct [<StackBoxDyn $Trait>] <
        '__frame,
        $($($lt : '__frame ,)* $($T : '__frame ,)*)?
        __AutoTraits : ?$crate::__::Sized + $crate::__::Sendness + $crate::__::Syncness = $crate::__::NoAutoTraits,
    >
    $( where $($wc)* )?
    {
        ptr: $crate::__::ErasedPtr,
        vtable: &'__frame <Self as $crate::__::GetVTable>::VTable,
        _auto_traits: $crate::__::PhantomData<__AutoTraits>,
    }
    const _: () = {
        trait $Trait<$($($lt ,)* $($T ,)*)?> : $crate::__::Sized + $super
        $( where $($wc)* )?
        {
            $(
                #[inline(always)]
                fn [<__ $method>] ( // underscored to avoid conflicts
                    $self :
                        $(
                            & $($ref)?
                            $(mut $(@$mut)?)?
                        )?
                        Self
                  $(,
                    $arg_name : $ArgTy )*
                ) $(-> $RetTy)?
                {
                    $($body)*
                }
            )*
        }
        impl<$($($lt ,)* $($T ,)*)? __Self : $super> $Trait<$($($lt ,)* $($T ,)*)?>
            for __Self
        $(where $($wc)* )?
        {}

        pub
        struct __VTable<$($($lt ,)* $($T ,)*)?>
        $( where $($wc)* )?
        {
            drop_in_place: unsafe fn ($crate::__::ErasedPtr),
            $(
                $method:
                    unsafe
                    fn(
                        __ptr: $crate::__::ErasedPtr $(,
                        $arg_name: $ArgTy )*
                    ) $(-> $RetTy)?
                ,
            )*
        }

        impl<
            '__frame,
            $($($lt ,)* $($T : '__frame ,)*)?
            __AutoTraits : ?$crate::__::Sized + $crate::__::Sendness + $crate::__::Syncness,
        >
            $crate::__::GetVTable
        for
            [<StackBoxDyn $Trait>]<'__frame, $($($lt ,)* $($T ,)*)? __AutoTraits>
        $( where $($wc)* )?
        {
            type VTable = __VTable<$($($lt ,)* $($T ,)*)?>;
        }

        trait HasVTable<
            '__frame,
            $($($lt ,)* $($T : '__frame ,)*)?
        >
        :
            $crate::__::Sized
        $( where $($wc)* )?
        {
            const VTABLE: __VTable<$($($lt ,)* $($T ,)*)?>;
        }

        impl<
            '__frame,
            $($($lt ,)* $($T : '__frame ,)*)?
            __Self : $super,
        >
            HasVTable<'__frame, $($($lt ,)* $($T ,)*)?>
        for
            __Self
        $( where $($wc)* )?
        {
            const VTABLE: __VTable<$($($lt ,)* $($T ,)*)?> = __VTable {
                drop_in_place: $crate::__::drop_in_place::<Self>,
                $(
                    $method: {
                        |
                            __ptr: $crate::__::ErasedPtr $(,
                            $arg_name: $ArgTy )*
                        | $( -> $RetTy )?
                        { unsafe {
                            // Safety: immediately coerced to an `unsafe fn`
                            let _convert =
                                |ptr: $crate::__::ErasedPtr| -> Self {
                                    let ptr: $crate::StackBox<'_, Self> =
                                        $crate::__::transmute(ptr)
                                    ;
                                    ptr.into_inner()
                                }
                            ;
                            {
                                $(
                                    let _convert =
                                        $crate::__::transmute::<
                                            $crate::__::ErasedPtr,
                                            &'_ $(mut $($mut)?)? Self,
                                        >
                                    ;
                                )?
                                _convert(__ptr).[<__ $method>]($($arg_name),*)
                            }
                        }}
                    },
                )*
            };
        }

        define_coercions! {
            [$crate::__::Send] => dyn $crate::__::Send,
            [$crate::__::Sync] => dyn $crate::__::Sync,
            [$crate::__::Send, $crate::__::Sync] => dyn $crate::__::Send + $crate::__::Sync,
            [] => $crate::__::NoAutoTraits,
        } macro_rules! define_coercions {(
            $__(
                [$__($AutoTrait:path),* $__(,)?] => $Marker:ty
            ),* $__(,)?
        ) => (
            $__(
                impl<
                    '__frame,
                    $($($lt : '__frame ,)* $($T : '__frame ,)*)?
                    __Pointee : $super
                >
                    $crate::__::DynCoerce<$crate::StackBox<'__frame, __Pointee>>
                for
                    [<StackBoxDyn $Trait>]<'__frame, $($($lt ,)* $($T ,)*)? $Marker>
                where
                    $__(
                        __Pointee : $AutoTrait,
                    )*
                    $($($wc)*)?
                {
                    fn fatten (it: $crate::StackBox<'__frame, __Pointee>)
                      -> Self
                    {
                        Self {
                            vtable: &<__Pointee as HasVTable<'__frame, $($($lt ,)* $($T ,)*)?>>::VTABLE,
                            ptr: unsafe { $crate::__::transmute(it) },
                            _auto_traits: $crate::__::PhantomData,
                        }
                    }
                }

                $__(
                    unsafe // Safety: from the `DynCoerce` bound added at construction site.
                        impl<'__frame, $($($lt : '__frame ,)* $($T : '__frame ,)*)?>
                            $AutoTrait
                        for
                            [<StackBoxDyn $Trait>]<'__frame, $($($lt ,)* $($T ,)*)? $Marker>
                        {}
                )*
            )*
        )} use define_coercions;

        impl<
            '__frame,
            $($($lt : '__frame ,)* $($T : '__frame ,)*)?
            __AutoTraits : ?$crate::__::Sized + $crate::__::Sendness + $crate::__::Syncness,
        >
            [<StackBoxDyn $Trait>]<'__frame, $($($lt ,)* $($T ,)*)? __AutoTraits>
        $(where $($wc)* )?
        {
            $(
                $pub
                fn $method (
                    self:
                        $(
                            & $($ref)?
                            $(mut $($mut)?)?
                        )?
                        Self
                    $(,
                    $arg_name: $ArgTy )*
                ) $(-> $RetTy)?
                {
                    unsafe {
                        (self.vtable.$method)(
                            $crate::__::ManuallyDrop::new(self).ptr $(,
                            $arg_name )*
                        )
                    }
                }
            )*
        }

        impl<
            '__frame,
            $($($lt : '__frame ,)* $($T : '__frame ,)*)?
            __AutoTraits : ?$crate::__::Sized + $crate::__::Sendness + $crate::__::Syncness,
        >
            $crate::__::Drop
        for
            [<StackBoxDyn $Trait>]<'__frame, $($($lt ,)* $($T ,)*)? __AutoTraits>
        $(where $($wc)* )?
        {
            fn drop (self: &'_ mut Self)
            {
                unsafe {
                    (self.vtable.drop_in_place)(self.ptr)
                }
            }
        }
    };
}); (
    // Missing dollar case
    $( #[doc = $doc:expr ] )*
    $pub:vis
    dyn $($rest:tt)*
) => ($crate::custom_dyn! {
    #![dollar = $]
    $( #[doc = $doc] )*
    $pub
    dyn $($rest)*
})}
