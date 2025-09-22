#![no_std]
#![feature(impl_trait_in_assoc_type)]

#[cfg(any(feature = "alloc", test))]
extern crate alloc;

#[cfg(any(feature = "std", test))]
extern crate std;

mod arrayqueue;
mod components;
#[cfg(test)]
mod tests;
mod utils;

pub use arrayqueue::*;
