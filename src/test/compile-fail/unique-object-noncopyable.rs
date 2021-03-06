// Copyright 2012 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

trait Foo {
    fn f(&self);
}

struct Bar {
    x: int,
}

impl Bar : Drop {
    fn finalize(&self) {}
}

impl Bar : Foo {
    fn f(&self) {
        io::println("hi");
    }
}

fn main() {
    let x = ~Bar { x: 10 };
    let y: ~Foo = x as ~Foo;
    let _z = copy y; //~ ERROR copying a value of non-copyable type
}

