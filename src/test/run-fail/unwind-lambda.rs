// Copyright 2012 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// error-pattern:fail

fn main() {
    let cheese = ~"roquefort";
    let carrots = @~"crunchy";

    fn@(tasties: @~str, macerate: fn(~str)) {
        macerate(copy *tasties);
    } (carrots, |food| {
        let mush = food + cheese;
        let cheese = copy cheese;
        let f = fn@() {
            let chew = mush + cheese;
            die!(~"so yummy")
        };
        f();
    });
}
