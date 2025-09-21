#![no_std]

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
