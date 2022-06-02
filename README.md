# uri_path_router

This library provides a procedural macro that uses a small domain specific
language to create a naive parser that can translate the path of a URI into a
variant of a flattened `enum` for routing purposes.

The generated logic is basically a big nested `match` that greedily accepts an
input `&str`, splits it into segments separated by `/` characters, and outputs
a variant of the `enum`, optionally capturing segments of the path if they are
specified as variables.

# Usage

You write this:

```rust
use uri_path_router::route;

route! {
    Router,
    "foo" => VariantA,
    "bar" => VariantBar {
        "x" => BarX,
        var => BarWithVariable(var)
    },
    "nested" / "syntax" / "demonstration" => Nested,
}
```

The macro will produce a flattened `enum` like this:

```rust
pub enum Router<'a> {
    VariantA,
    VariantBar,
    BarX,
    BarWithVariable { var: &'a str },
    Nested,
}
```

Note that the `enum` only captures the variable when a variant specifies it,
and always does so as a borrow and as such without any allocations.
To convert a `&str` into a variant, use `TryFrom`:

```rust
assert_eq!(Router::try_from("/foo"), Ok(Router::VariantA));
assert_eq!(Router::try_from("/bar"), Ok(Router::VariantBar));
assert_eq!(Router::try_from("/bar/x"), Ok(Router::BarX));
assert_eq!(
    Router::try_from("/bar/not-x"),
    Ok(Router::BarWithVariable { var: "not-x" })
);
assert_eq!(Router::try_from("/whatever"), Err(()));
assert_eq!(
    Router::try_from("/nested/syntax/demonstration"),
    Ok(Router::Nested)
);
```

A nifty feature of this crate is that documentation is auto-generated for
every variant of the `enum` describing what pattern matches it.
You can check this out by hovering over the variant in your IDE (assuming you
have `rust-analyzer` or something similar configured to display tooltips), or
by running `cargo doc` on your crate and searching for the generated `enum`.

# License

This library is provided under the MIT license. See [LICENSE](LICENSE).
