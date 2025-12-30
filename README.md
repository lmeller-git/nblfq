[![Codecov](https://codecov.io/github/lmeller-git/nblfqueue/coverage.svg?branch=main)](https://codecov.io/gh/lmeller-git/nblfqueue)
![CI Test](https://github.com/lmeller-git/nblfqueue/actions/workflows/test.yml/badge.svg?branch=main)

# nblf-queue

An atomic wait-free MPMC queue based on the NBLFQ algorithm.

This repository provides mutliple queue implementations, a long with different `Slot`s.
Slots determine how data is stored and updated. Currently only `TaggedPtr64` and `TaggedPtr128` are supported.
`TaggedPtr64` and `TaggedPtr128` are only usable when storing pointers, which are marked using `PtrLike`. Stored pointers should not have sizes >48 bits and >64 bits respectively.

## Usage

`nblf_queue::array::StaticQueue`:

```rust
  use nblfq::{array::StaticQueue, slot::TaggedPtr64};

  let q: StaticQueue<10, TaggedPtr64<_>> = StaticQueue::new();

  assert!(q.push(&42).is_ok());
  assert!(q.push(&1).is_ok());

  assert_eq!(q.pop(), Some(&42));
  assert_eq!(q.pop(), Some(&1));
```


`nblf_queue::owned::Queue`:

```rust
  use nblfq::{owned::Queue, slot::TaggedPtr64};

  let q: Queue<TaggedPtr64<_>> = Queue::new(10);

  assert!(q.push(&42).is_ok());
  assert!(q.push(&1).is_ok());

  assert_eq!(q.pop(), Some(&42));
  assert_eq!(q.pop(), Some(&1));
```


## Platform Support

Multiple storage types are available, dependent on platform:

- **TaggedPtr64** - 64-bit platforms with 48-bit virtual addresses and 64-btit atomic operations

- **TaggedPtr128** - platforms with native atomic 128-bit support


## Feature Flags

- `std`: Enables `std` and `alloc` support

- `alloc`: Enables `alloc` support (required for `crate::owned::*`)

- `tagged-ptr` (default): Enables TaggedPtr64


## References

Alexandre Denis, Charles Goedefroit. NBLFQ: a lock-free MPMC queue optimized for low contention.
IPDPS 2025 - 39th International Parallel & Distributed Processing Symposium, IEEE, Jun 2025,
Milan, Italy. hal-04851700v2
