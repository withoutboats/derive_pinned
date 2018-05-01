#![feature(pin, arbitrary_self_types)]

#[macro_use] extern crate derive_pinned;

use std::mem::Pin;

#[derive(PinAccessor)]
struct Foo {
    #[pin_accessor]
    bar: i32,
}

fn compile_check(mut f: Pin<Foo>) -> i32 {
    let x: Pin<i32> = f.bar_pinned();
    *x + 1
}

#[test]
fn main() {
    let mut foo = Foo { bar: 0 };
    let pinned = Pin::new(&mut foo);
    assert_eq!(compile_check(pinned), 1);
}
