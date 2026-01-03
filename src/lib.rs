#![no_std]
#![feature(impl_trait_in_assoc_type)]

#[cfg(any(feature = "alloc", test))]
extern crate alloc;

#[cfg(any(feature = "std", test))]
extern crate std;

mod array;
mod core;
#[cfg(feature = "alloc")]
mod owned;
#[cfg(feature = "pool")]
mod pool;
#[cfg(test)]
mod tests;
mod utils;

pub use crate::core::{ForcePushQueue, MPMCQueue, slots};
pub use array::{StaticPooledQueue, StaticQueue};
#[cfg(feature = "alloc")]
pub use owned::{PooledQueue, Queue};
