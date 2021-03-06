// Copyright 2012 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// xfail-fast
// aux-build:trait_inheritance_auto_xc_aux.rs

extern mod aux(name = "trait_inheritance_auto_xc_aux");

use aux::{Foo, Bar, Baz, Quux};

struct A { x: int }

impl A : Foo { fn f() -> int { 10 } }
impl A : Bar { fn g() -> int { 20 } }
impl A : Baz { fn h() -> int { 30 } }

fn f<T: Quux>(a: &T) {
    assert a.f() == 10;
    assert a.g() == 20;
    assert a.h() == 30;
}

pub fn main() {
    let a = &A { x: 3 };
    f(a);
}

