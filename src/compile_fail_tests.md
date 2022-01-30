# The following snippets fail to compile

Not using the macro leads to non-higher-order signatures:
```rust ,compile_fail
let f = |x| {
    let _: &i32 = x;
    x
};
match 42 { local => {
    f(&local);
}}
drop(f);
```

Or to partially higher-order signatures (input side), leading to borrow errors
on return:

```rust ,compile_fail
let _f =
    // ::higher_order_closure::higher_order_closure!
    {
        // for<>
        |x: &'_ i32| -> &'_ i32 {
            x // <- Error, does not live long enough.
        }
    }
;
```

___

A higher-order closure properly borrows from its input, no matter the actual
impl.

```rust ,compile_fail
use ::higher_order_closure::hrtb;

let f = hrtb!(|_: &()| -> &() { &() });
let it = {
    let local = ();
    f(&local)
};
drop(it);
```

<!-- Templated by `cargo-generate` using https://github.com/danielhenrymantilla/proc-macro-template -->
