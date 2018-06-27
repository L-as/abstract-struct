# abstract_struct

A Rust macro for automatically generating a corresponding trait.

## Why?

The purpose of creating a corresponding trait is to hide
away the generic parameters of a struct.

A lot of the time, you don't really care about what types
a generic struct was instantiated with; you just want to use it.

This way you can also add new generic parameters without breaking
existing code.

## Requirements

You must be using a nightly compiler, since attribute-like procedural macros
haven't been stabilized yet.

## Example

There are thorough examples in the examples directory in the repository.

```rust
extern crate abstract_struct;

use abstract_struct::{abstract_struct, abstract_struct_debug};

pub trait A {fn a(&self) {}}
pub trait B: Sized {fn b(self) {}}
pub trait C {fn c(&self) {}}
pub trait D {fn d(&self) {}}
pub trait E {fn e(&self) {}}
pub trait F {fn f(&self) {}}

impl A for usize {fn a(&self) {}}
impl B for usize {fn b(self) {}}
impl C for usize {fn c(&self) {}}
impl D for usize {fn d(&self) {}}
impl E for usize {fn e(&self) {}}
impl F for usize {fn f(&self) {}}

// use abstract_struct_debug if you want to inspect the generated code.
#[abstract_struct]
pub struct MyAwesomeStruct<T: A + B, U: C + D, V: E + F>
{
	t: T,
	u: U,
	v: V,
}

fn use_awesome_struct<U: C + D + std::fmt::Debug>(s: impl MyAwesomeStructAbstract<U = U>) {
	println!("{:?}", s.u);
	s.t.a();
	// `into` converts from `impl StructAbstract` to `Struct`
	s.into().t.b();
}

fn main() {
	let s = MyAwesomeStruct {
		t: 0,
		u: 1,
		v: 2,
	}.wrap();
	
	use_awesome_struct(s);
}
```

This creates a public struct called MyAwesomeStruct and a corresponding trait
called MyAwesomeStructAbstract, which is also public.
The publicity of the trait matches the publicity of the struct.

## Arguments to pass to macro

You can do `#[abstract_struct(nowrap)]`
to not generate the `fn wrap(self) -> Wrapper<Self>` method automatically.

You can do `#[abstract_struct(wrap = mywrap)]`
to rename the name of the wrap method to the passed in name.
