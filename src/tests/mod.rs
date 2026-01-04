#[cfg(all(not(loom), not(shuttle)))]
mod core;
#[cfg(loom)]
mod loom;
#[cfg(shuttle)]
mod shuttle;
#[cfg(all(not(loom), not(shuttle)))]
mod test_library;
