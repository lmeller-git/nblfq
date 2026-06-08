[![Codecov](https://codecov.io/github/lmeller-git/nblf-queue/coverage.svg?branch=main)](https://codecov.io/gh/lmeller-git/nblf-queue)
![CI Test](https://github.com/lmeller-git/nblf-queue/actions/workflows/test.yml/badge.svg?branch=main)
![Safety Test](https://github.com/lmeller-git/nblf-queue/actions/workflows/safety.yml/badge.svg?branch=main)
![no_std Test](https://github.com/lmeller-git/nblf-queue/actions/workflows/nostd.yml/badge.svg?branch=main)


# nblf-queue

> Non-Blocking Lock-Free Queue

An atomic lock-free MPMC queue based on the NBLFQ algorithm.

This repository provides multiple queue implementations with different storage and allocation strategies.

All queues in this repository are safe to use in a concurrent context and will never block the calling thread.

## Queue variants

- **Static queues**: fixed-capacity queues backed by static storage
- **Allocated queues**: fixed-capacity queues backed by dynamically allocated storage, only available on feature `alloc`
- **Pooled Queues**: variants of other queues, which may store arbitrary types, only available on feature `pool`

Non-pooled queues store items in atomically updated slots, restricting the stored items to small, pointer-like values.


## Usage

`nblf_queue::StaticQueue`:

```rust
  use nblf_queue::{StaticQueue, MPMCQueue};

  let q: StaticQueue<_, 2> = StaticQueue::new();

  assert!(q.push(&42).is_ok());
  assert!(q.push(&1).is_ok());
  assert!(q.push(&4242).is_err());

  assert_eq!(q.pop(), Some(&42));
  assert_eq!(q.pop(), Some(&1));
  assert!(q.pop().is_none());
```


`nblf_queue::PooledStaticQueue`:

```rust
  #[cfg(feature = "pool")]
  fn run() {
    use nblf_queue::{PooledStaticQueue, MPMCQueue};

    let q: PooledStaticQueue<_, 2> = PooledStaticQueue::new();

    assert!(q.push(42).is_ok());
    assert!(q.push(1).is_ok());
    assert!(q.push(4242).is_err());

    assert_eq!(q.pop(), Some(42));
    assert_eq!(q.pop(), Some(1));
    assert!(q.pop().is_none());
  }

  #[cfg(feature = "pool")]
  run();
```


## Choosing a queue type

`StaticQueue` and `Queue` may only store small values and are optimized for this use case.

`PooledStaticQueue` and `PooledQueue` may store arbitrary types, at the cost of higher memory usage and runtime cost.


## Platform Support

Multiple storage types are available, dependent on platform:

- **TaggedPtr64** - platforms with native 64-bit atomic operations or feature `atomic-fallback`

- **TaggedPtr128** - platforms with native 128-bit atomic operations or feature `atomic-fallback`

Storage types will be chosen automatically, unless sepcified explicitly.

## Feature Flags

- `std`: Enables `std` and `alloc` support

- `alloc`: Enables `alloc` support, allowing usage of some dynamically allocated queues

- `pool`: Enables pooled queues, which may store any type

- `atomic-fallback`: Uses `portable-atomic` `fallback` feature for atomics if necessary. It is discouraged to use this feature, as `fallback` internally uses locks

- `default`: `pool`

## Testing

The core test-suite of this crate was adapted from [`crossbeam-queue`](https://github.com/crossbeam-rs/crossbeam/tree/main/crossbeam-queue).

Current testing is based on:

- **Miri** - to validate pointer arithmetic
- **Loom and Shuttle** - to test for race conditions
- **ASan** - to check for memory corruption and leakage


## References

Alexandre Denis, Charles Goedefroit. NBLFQ: a lock-free MPMC queue optimized for low contention.
IPDPS 2025 - 39th International Parallel & Distributed Processing Symposium, IEEE, Jun 2025,
Milan, Italy. hal-04851700v2
