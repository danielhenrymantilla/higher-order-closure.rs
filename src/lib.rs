#![cfg_attr(feature = "better-docs",
    cfg_attr(all(), doc = include_str!("../README.md")),
)]
#![no_std]
#![forbid(unsafe_code)]

pub use higher_order_closure as hrtb;

/// See [the main docs][crate] for more info.
#[macro_export]
macro_rules! higher_order_closure {(
    $(#![
        with<
            $($(
                $lt:lifetime $(: $super_lt:lifetime)?
            ),+ $(,)?)?
            $($(
                $T:ident $(:
                    $(
                        ?$Sized:ident $(+)?
                    )?
                    $(
                        $super:lifetime $(+)?
                    )?
                    $(
                        $Trait:path
                    )?
                )?
            ),+ $(,)?)?
        >
        $(where
            $($wc:tt)*
        )?
    ])?

    $( for<$($hr:lifetime),* $(,)?> )?
    $( move $(@$move:tt)?)?
    | $($arg_pat:tt : $ArgTy:ty),* $(,)?|
      -> $Ret:ty
    $body:block
) => (
    ({
        fn __funnel__<
            $(
                $($(
                    $lt $(: $super_lt)?
                    ,
                )+)?
                $($(
                    $T
                    $(:
                        $(?$Sized +)?
                        $($super +)?
                        $($Trait)?
                    )?
                    ,
                )+)?
            )?
                __Closure,
            >
        (
            f: __Closure,
        ) -> __Closure
        where
            __Closure : for<$($($hr ,)*)?> $crate::__::FnOnce($($ArgTy),*) -> $Ret,
            $($($($wc)*)?)?
        {
            f
        }

        __funnel__::<$($($($T ,)+)?)? _>
    })(
        $(move $($move)?)? |$($arg_pat),*| $body
    )
)}

// macro internals
#[doc(hidden)] /** Not part of the public API */ pub
mod __ {
    pub use ::core::ops::FnOnce;
}

#[cfg_attr(feature = "ui-tests",
    cfg_attr(all(), doc = include_str!("compile_fail_tests.md")),
)]
mod _compile_fail_tests {}
