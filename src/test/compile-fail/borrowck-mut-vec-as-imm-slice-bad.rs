// Copyright 2012 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

fn want_slice(v: &[int]) -> int {
    let mut sum = 0;
    for vec::each(v) |i| { sum += *i; }
    return sum;
}

fn has_mut_vec(+v: @~[mut int]) -> int {
    want_slice(*v) //~ ERROR illegal borrow unless pure
        //~^ NOTE impure due to access to impure function
}

fn main() {
    assert has_mut_vec(@~[mut 1, 2, 3]) == 6;
}