#[cfg(all(not(loom), not(shuttle)))]
mod core;
#[cfg(feature)]
mod echeneis_tests;
#[cfg(loom)]
mod loom;
#[cfg(shuttle)]
mod shuttle;
// not thread::scope in loom
#[cfg(not(loom))]
mod test_library;
