// Copyright 2012 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

trait bar<T> {
    fn get_bar() -> T;
}

fn foo<T, U: bar<T>>(b: U) -> T {
    b.get_bar()
}

struct cbar {
    x: int,
}

impl cbar : bar<int> {
    fn get_bar() -> int {
        self.x
    }
}

fn cbar(x: int) -> cbar {
    cbar {
        x: x
    }
}

pub fn main() {
    let x: int = foo::<int, cbar>(cbar(5));
    assert x == 5;
}
