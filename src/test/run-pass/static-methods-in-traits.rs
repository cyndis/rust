// Copyright 2012 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

mod a {
	pub trait Foo {
		static pub fn foo() -> Self;
	}

	impl int : Foo {
		static pub fn foo() -> int {
			3
		}
	}
	
	impl uint : Foo {
		static pub fn foo() -> uint {
			5u
		}
	}
}

pub fn main() {
	let x: int = a::Foo::foo();
	let y: uint = a::Foo::foo();
	assert x == 3;
	assert y == 5;
}

