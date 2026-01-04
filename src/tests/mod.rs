#[cfg(all(not(loom), any(feature = "std", test)))]
mod core;
#[cfg(all(loom, any(feature = "std", test)))]
mod loom;
#[cfg(all(not(loom), any(feature = "std", test)))]
mod test_library;
