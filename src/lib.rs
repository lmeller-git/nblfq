#![no_std]
#![feature(impl_trait_in_assoc_type)]

#[cfg(any(feature = "alloc", test))]
extern crate alloc;

#[cfg(any(feature = "std", test))]
extern crate std;

pub mod array;
mod core;
#[cfg(feature = "alloc")]
pub mod owned;
#[cfg(all(feature = "tagged-ptr", target_has_atomic = "64"))]
pub mod pool;
#[cfg(test)]
mod tests;
mod utils;

pub use crate::core::*;
