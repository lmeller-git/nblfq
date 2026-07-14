#[cfg(all(not(loom), not(shuttle), not(echeneis)))]
mod core;
#[cfg(echeneis)]
mod echeneis_tests;
#[cfg(loom)]
mod loom;
#[cfg(shuttle)]
mod shuttle;
// not thread::scope in loom, not needed with echeneis
#[cfg(not(any(loom, echeneis)))]
mod test_library;
