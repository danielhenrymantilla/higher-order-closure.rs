# `::higher-order-closure`

Allow function lifetime elision and explicit `for<'a>` annotations on closures.

<!-- Templated by `cargo-generate` using https://github.com/danielhenrymantilla/proc-macro-template -->

[![Repository](https://img.shields.io/badge/repository-GitHub-brightgreen.svg)](
https://github.com/danielhenrymantilla/higher-order-closure.rs)
[![Latest version](https://img.shields.io/crates/v/higher-order-closure.svg)](
https://crates.io/crates/higher-order-closure)
[![Documentation](https://docs.rs/higher-order-closure/badge.svg)](
https://docs.rs/higher-order-closure)
[![MSRV](https://img.shields.io/badge/MSRV-1.42.0-white)](
https://gist.github.com/danielhenrymantilla/8e5b721b3929084562f8f65668920c33)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](
https://github.com/rust-secure-code/safety-dance/)
[![License](https://img.shields.io/crates/l/higher-order-closure.svg)](
https://github.com/danielhenrymantilla/higher-order-closure.rs/blob/master/LICENSE-ZLIB)
[![CI](https://github.com/danielhenrymantilla/higher-order-closure.rs/workflows/CI/badge.svg)](
https://github.com/danielhenrymantilla/higher-order-closure.rs/actions)

### Motivation / Rationale

See the [RFC #3216](https://github.com/rust-lang/rfcs/pull/3216): this crate
is a Proof-of-Concept of the ideas laid out there[^1].

[ˆ1]: with the exception of allowing elided lifetimes to have a meaning (chosen higher-order), which the RFC cannot do in order to be future-proof.

<details><summary>Click to expand</summary>

The following example fails to compile:

```rust ,compile_fail
let f = |x| {
    let _: &i32 = x;
    x
};
{
    let scoped = 42;
    f(&scoped);
} // <- scoped dropped here.
f(&42);
```

```console
error[E0597]: `scoped` does not live long enough
  --> src/lib.rs:10:7
   |
10 |     f(&scoped);
   |       ^^^^^^^ borrowed value does not live long enough
11 | } // <- scoped dropped here.
   | - `scoped` dropped here while still borrowed
12 | f(&42);
   | - borrow later used here

For more information about this error, try `rustc --explain E0597`.
```

Indeed, the signature of `f` in that example is that of:

```rust ,ignore
impl Fn(&'inferred i32) -> &'inferred i32
```

wherein `'inferred` represents some not yet known (to be inferred)
**but fixed** lifetime.

Then,

```rust ,ignore
{
    let scoped = 42;
    f(&scoped); // `'inferred` must "fit" into this borrow…
} // <- and thus can't span beyond this point.
f(&42) // And yet `'inferred` is used here as well => Error!
```

___

The solution, then, is to explicitly annotate the types involved in the closure
signature, and more importantly, the **lifetime "holes" / placeholders /
parameters involved in that signature**:

```rust
           // Rust sees this "hole" early enough in its compiler pass
           //                       to figure out that the closure signature
           // vv                    needs to be higher-order, **input-wise**
let f = |_x: &'_ i32| {
};
{
    let scoped = 42;
    f(&scoped);
}
f(&42);
```

This makes it so the input-side of the closure signature effectively gets to
be higher-order. Instead of:

```rust ,ignore
impl Fn(&'inferred_and_thus_fixed i32)...
```

for some outer inferred (and thus, _fixed_) lifetime `'inferred_and_thus_fixed`,
we now have:

```rust ,ignore
impl for<'any> Fn(&'any i32)...
```

___

This works, but **_quid_ of _returning_ borrows**? (all while remaining
higher-order)

Indeed, the following fails to compile:

```rust ,compile_fail
let f = |x: &'_ i32| -> &'_ i32 {
    x // <- Error, does not live long enough.
};
```

```console
error: lifetime may not live long enough
 --> src/lib.rs:5:5
  |
4 | let f = |x: &'_ i32| -> &'_ i32 {
  |             -           - let's call the lifetime of this reference `'2`
  |             |
  |             let's call the lifetime of this reference `'1`
5 |     x // <- Error, does not live long enough.
  |     ^ returning this value requires that `'1` must outlive `'2`
```

The reason for this is that "explicit lifetime 'holes' / placeholders become
higher-order lifetime parameters in the closure signature" mechanism only works
for the input-side of the signature.

The return side keeps using an inferred lifetime:

```rust ,ignore
let f = /* for<'any> */ |x: &'any i32| -> &'inferred i32 {
    x // <- Error, does not live long enough (when `'any < 'inferred`)
};
```

we'd like for `f` there to have the `fn(&'any i32) -> &'any i32` signature that
functions get [from the lifetime elision rules for function
signatures][lifetime elision rules].

Hence the reason for using this crate.

</details>

### Examples

This crate provides a `higher_order_closure!` macro, with which one can properly
annotate the closure signatures so that they become higher-order, featuring
universally quantified ("forall" quantification, `for<…>` in Rust) lifetimes:

```rust
#[macro_use]
extern crate higher_order_closure;

fn main ()
{
    let f = higher_order_closure! {
        for<'any> |x: &'any i32| -> &'any i32 {
            x
        }
    };
    {
        let local = 42;
        f(&local);
    }
    f(&42);
}
```

The lifetime elision rules of function signatures apply, which means the
previous signature can even be simplified down to:

```rust
#[macro_use]
extern crate higher_order_closure;

fn main ()
{
    let f =
        higher_order_closure! {
            for<> |x: &'_ i32| -> &'_ i32 {
                x
            }
        }
    ;
}
```

or even:

```rust
#[macro_use]
extern crate higher_order_closure;

fn main ()
{
    let f =
        higher_order_closure! {
            |x: &'_ i32| -> &'_ i32 {
                x
            }
        }
    ;
}
```

  - Because of these lifetime elision in function signatures semantics, it is
    highly advisable that the `elided_lifetimes_in_paths` be, at the very least,
    on `warn` when using this macro:

    ```rust ,ignore
    //! At the root of the `src/{lib,main}.rs`.
    #![warn(elided_lifetimes_in_paths)]
    ```

### Extra features

#### Macro shorthand

The main macro is re-exported as a `hrtb!` shorthand, to allow inlining
closures with less rightward drift:


```rust
#[macro_use]
extern crate higher_order_closure;

fn main ()
{
    let f = {
        let y = 42;
        hrtb!(for<'any> move |x: &'any i32| -> &'any i32 {
            println!("{y}");
            x
        })
    };
}
```

#### Outer generic parameters

Given how the macro internally works[^2], generic parameters "in scope" won't,
by default, be available in the closure signature (similar to `const`s and
nested function or type definitions).

In order to make them available, `higher_order_signature!` accepts an initial
optional `#![with<simple generics…>]` parameter (or even
`#![with<simple generics…> where clauses…]` if the "simple shape"
restrictions for the generic parameters are too restrictive).

<details><summary>"simple shaped" generics macro restrictions</summary>

The generics parameters inside `#![with<…>]` have to be of the form:

```rust ,ignore
<
    'a, 'b : 'a, ...
    T, U : ?Sized + 'a + ::core::fmt::Debug, V, ...
>
```

Mainly:

  - at most one super-lifetime bound on each lifetime,

  - the super-bounds on the types must be exactly of the form:
     1. optional `?Sized`,
     1. followed by an optional lifetime bound,
     1. followed by an optional trait bound.
     1. And nothing more.
    If you need more versatility, use the `where` clauses.

In practice, however, the bounds are seldom needed, since such generics are only
used for the _signature_ of the closure, not its body / implementation.

</details>


```rust
#[macro_use]
extern crate higher_order_closure;

use ::core::fmt::Display;

fn example<T : Display> (iter: impl IntoIterator<Item = (bool, T)>)
{
    let mb_display = higher_order_closure! {
        #![with<T>]
        |hide: bool, elem: &'_ T| -> &'_ dyn Display {
            if hide {
                &"<hidden>"
            } else {
                elem
            }
        }
    };
    for (hide, elem) in iter {
        println!("{}", mb_display(hide, &elem));
    }
}

fn main ()
{
    example([(false, 42), (true, 27), (false, 0)]);
}
```

#### Mixing higher-order lifetime parameters with inferred ones

Since inside `higher_order_closure!`, `'_` has the semantics of lifetime elision
for function signatures, it means that, by default, all the lifetime parameters
appearing in such closure signatures will necessarily be higher-order.

In some more contrived or rare scenarios, this may be undesirable.

In that case, a nice way to work around that is by artificially introducing
generic lifetime parameters as seen above, and using these newly introduced
named lifetime parameters where the inferred lifetime would be desired.

```rust
#[macro_use]
extern crate higher_order_closure;

fn main ()
{
    let y = 42;
    let f = higher_order_closure! {
        #![with<'inferred>]
        for<'any> |x: &'any i32| -> (&'any i32, &'inferred i32) {
            (x, &y)
        }
    };
    let (&a, &b) = f(&27);
    assert_eq!(a + b, 42 + 27);
}
```

[^2]: it generates a "closure identity" helper function, with the desired
higher-order signatures embedded as `Fn` bounds on its parameters, thus making
it act as a "funnel" that only lets closure with the right signature pass
through).

[lifetime elision rules]: https://doc.rust-lang.org/1.58.1/reference/lifetime-elision.html#lifetime-elision-in-functions
