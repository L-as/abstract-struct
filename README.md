# abstract_struct

A Rust macro for automatically generating a corresponding trait.

## Why?

The purpose of creating a corresponding trait is to hide
away the generic parameters of a struct.

A lot of the time, you don't really care about what types
a generic struct was instantiated with; you just want to use it.

Currently, this can get quite tedious if the struct has a
lot of generic parameters, especially if these parameters have
a lot of constraints. Then you have to specify these constraints everywhere.
RFC #2089 (implied_bounds) will make specifying the constraints unnecessary in functions
and impl blocks, however you will still need to do it in structs.

examples/simple.rs contains an example of this madness, and how this macro abstracts it all away.

## Requirements

You must be using a nightly compiler, since this crate uses
experimental proc_macro features and also non-lexical lifetimes.
This is because I am lazy and since this will work in the 2018 epoch anyway.

## Example

There are thorough examples in the examples directory in the repository.

## Usage

```ignore
abstract_struct! {
#[all_attribs_in_the_world]
pub struct MyAwesomeStruct<T, U, V> where
	T: A + B + C + D + E + F + G + H ...
{
	t: T,
	...
}

#[some_more_attribs]
pub unsafe trait {
	type T: {T} = T;
	// NB: Must use &Self::T and not T
	fn t(&self) -> &Self::T {
		&self.t
	}
}}

fn use_awesome_struct(_: impl MyAwesomeStruct) {...}
```

This creates a public struct called MyAwesomeStructConcrete and a corresponding trait
just called MyAwesomeStruct, which is also public.

You can now use these types as you would use all other types.

### Associated type syntax (`{T}`)
The syntax `{T}` in the associated type declaration is a shortcut for the
constraints specified on the generic type parameter `T`.
This syntax can currently only be used in associated type declarations.

### Unsafe traits
If you specify `unsafe`, then the trait will be an unsafe trait, which will
prevent other types from implementing it unless it's an `unsafe impl`.
