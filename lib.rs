#![feature(external_doc)]

#![doc(include = "README.md")]

extern crate abstract_struct_macro;

pub use abstract_struct_macro::*;

use std::ops::Deref;

/// A simple wrapper over the contained
/// type, that implements std::ops::Deref
/// with the contained type as Target.
pub struct Wrapper<T>(pub T);

impl<T> Deref for Wrapper<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
