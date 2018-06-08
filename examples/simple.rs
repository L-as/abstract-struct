#![feature(proc_macro)]

extern crate abstract_struct;

use abstract_struct::abstract_struct;

use std::hash::Hash;

pub trait T<A, B> {}

abstract_struct!{
#[derive(Clone)]
pub struct Simple<A: T<u32, u64>, B, C> where
	A: Clone + Send + Sync + 'static + Copy + PartialEq + Eq + Hash,
	B: Iterator<Item=u32> + Clone + Into<u64> + Send + Sync + 'static + Copy + PartialEq + Eq + Hash,
	C: From<u128> + Clone + Send + Sync + 'static + Copy + PartialEq + Eq + Hash,
{
	a: A,
	b: B,
	c: C,
}

trait {
}}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct MyA;

impl T<u32, u64> for MyA {}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct MyB;

impl Iterator for MyB {
	type Item = u32;

	fn next(&mut self) -> Option<u32> {
		None
	}
}

impl From<MyB> for u64 {
	fn from(_: MyB) -> u64 {
		0
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct MyC;

impl From<u128> for MyC {
	fn from(_: u128) -> MyC {
		MyC
	}
}

// Fucking ugly
fn take_simple_concrete<A: T<u32, u64>, B, C>(_s: SimpleConcrete<A, B, C>) where
	A: Clone + Send + Sync + 'static + Copy + PartialEq + Eq + Hash,
	B: Iterator<Item=u32> + Clone + Into<u64> + Send + Sync + 'static + Copy + PartialEq + Eq + Hash,
	C: From<u128> + Clone + Send + Sync + 'static + Copy + PartialEq + Eq + Hash,
{}

// Absolute masterpiece
fn take_simple(_s: impl Simple) {}

fn main() {
	let s = SimpleConcrete {
		a: MyA,
		b: MyB,
		c: MyC,
	};
	take_simple_concrete(s.clone());
	take_simple(s);
}
