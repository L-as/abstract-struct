#![feature(proc_macro)]
#![feature(never_type)]

extern crate abstract_struct;

use abstract_struct::*;

#[abstract_struct(wrap = mywrap)]
#[derive(Clone)]
pub struct Simple<'a, T: Clone, U: PartialEq + 'a>(T, &'a U);

impl<T: Clone, U: PartialEq> Simple<'_, T, U> {
	fn inner(self) -> T {
		self.0
	}
}

impl Simple<'_, !, !> {
	fn new() -> Simple<'static, impl Clone, impl PartialEq> {
		Simple(22, &13)
	}
}

fn take_simple_concrete<T: Clone, U: PartialEq>(s: Simple<'_, T, U>) {
	take_simple(s.mywrap());
}

fn take_simple<'a>(s: impl SimpleAbstract<'a>) {
	let _ = s.into().inner();
}

fn main() {
	take_simple_concrete(Simple::new());
	take_simple(Simple::new().mywrap());
}
