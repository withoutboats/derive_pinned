A derive for accessing a pinned field of a type.

```rust
#[derive(PinAccessor)]
struct Foo {
    #[pin_accessor]
    bar: i32,
}
```

Adds an inherent method to `Foo`:

```rust
impl Foo {
    fn bar_pinned<'a>(self: &'a mut Pin<Foo>) -> Pin<'a, i32> { ... }
}
```

## Controlling the visibility and method name

The visibility of the method is, by default, the same as the visibility of the field.

The name of the method is, by default, the name of the field, with `_pinned` as a suffix.

Both of these can be controlled by arguments to the `pin_accessor` attribute:

```rust
#[derive(PinAccessor)]
pub struct Foo {
    #[pin_accessor(vis = "pub", name = "bar")]
    bar: i32,
}
```

This creates a public method called `bar`, instead of a private method called `bar_pinned`.

## Safety

This derive creates a method that is safe. This requires you to uphold an invariant in any
unsafe code you write using this field:

* If the field type implements `Unpin`, this method is always safe.
* If the field type does not implement `Unpin`, you must never move out of it from a `Pin` of the
  type it belongs to.

In other words, unless you are messing around with moving potentially `Unpin` data around, you are
definitely safe.

## Still TODO

* This derive is not yet no_std compatible, but easily could be.
* This derive creates a method requiring the double indirection of `&mut Pin<Self>`. This is
  because of limitations on the std APIs of `Pin`, which should probably be changed.
