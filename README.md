[![Codecov](https://codecov.io/github/lmeller-git/nblf-queue/coverage.svg?branch=main)](https://codecov.io/gh/lmeller-git/nblfqueue)
![CI Test](https://github.com/lmeller-git/nblf-queue/actions/workflows/test.yml/badge.svg?branch=main)

# nblf-queue

An atomic wait-free MPMC queue based on the NBLFQ algorithm.

This repository provides mutliple queue implementations, along with different `Slot` types.
Slots determine how data is stored and updated. Currently only `TaggedPtr64` and `TaggedPtr128` are supported.
`TaggedPtr64` and `TaggedPtr128` are only usable when storing pointers or small values, which are marked using `PtrLike`.
`PtrLike` is implemented for some widely used pointers, but may also be implemented on your own types.
To store arbitrary types, a pooled variant of all queues is available on feature `pool`.

## Usage

`nblf_queue::StaticQueue`:

```rust
  use nblfq_queue::{StaticQueue, MPMCQueue};

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
  use nblfq_queue::{PooledStaticQueue, MPMCQueue};

  let q: PooledStaticQueue<_, 2> = PooledStaticQueue::new();

  assert!(q.push(42).is_ok());
  assert!(q.push(1).is_ok());
  assert!(q.push(4242).is_err());

  assert_eq!(q.pop(), Some(42));
  assert_eq!(q.pop(), Some(1));
  assert!(q.pop().is_none());
```


## Platform Support

Multiple storage types are available, dependent on platform:

- **TaggedPtr64** - 64-bit platforms with (at most) 48-bit virtual addresses (this is currently not checked) and 64-bit atomic operations or feature `atomic-fallback`

- **TaggedPtr128** - platforms with native atomic 128-bit support or feature `atomic-fallback`


## Feature Flags

- `std`: Enables `std` and `alloc` support

- `alloc`: Enables `alloc` support, allowing usage of some dynamically allocated queues

- `pool`: Enables pooled queues, which may store any type

- `atomic-fallback`: Uses `portable-atomic` `fallback` for atomics if necessary. It is discouraged to use this feature, as `fallback` internally uses locks

- `default`: `pool`

## References

Alexandre Denis, Charles Goedefroit. NBLFQ: a lock-free MPMC queue optimized for low contention.
IPDPS 2025 - 39th International Parallel & Distributed Processing Symposium, IEEE, Jun 2025,
Milan, Italy. hal-04851700v2
