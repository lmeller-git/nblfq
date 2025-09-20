#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod arrayqueue;
mod components;
#[cfg(test)]
mod tests;
mod utils;

pub use arrayqueue::*;
