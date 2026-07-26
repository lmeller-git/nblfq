[![Codecov](https://codecov.io/github/lmeller-git/nblf-queue/coverage.svg?branch=main)](https://codecov.io/gh/lmeller-git/nblf-queue)
![CI Test](https://github.com/lmeller-git/nblf-queue/actions/workflows/test.yml/badge.svg?branch=main)
![Safety Test](https://github.com/lmeller-git/nblf-queue/actions/workflows/safety.yml/badge.svg?branch=main)
![no_std Test](https://github.com/lmeller-git/nblf-queue/actions/workflows/nostd.yml/badge.svg?branch=main)
[![Crates.io](https://img.shields.io/crates/v/nblf-queue)](https://crates.io/crates/nblf-queue)
[![Docs.rs](https://docs.rs/nblf-queue/badge.svg)](https://docs.rs/nblf-queue)
[![PyPI version](https://img.shields.io/pypi/v/nblf-queue)](https://pypi.org/project/nblf-queue)

# nblf-queue

> Non-Blocking Lock-Free Queue

<!-- cargo-rdme start -->

An atomic lock-free MPMC queue based on the NBLFQ algorithm.

This repository provides multiple queue implementations with different storage and allocation strategies.

All queues in this repository are safe to use in a concurrent context.
All variants with the exception of [`DynamicQueue`](https://docs.rs/nblf-queue/latest/nblf_queue/growable/queue/struct.DynamicQueue.html) and [`PooledDynamicQueue`](https://docs.rs/nblf-queue/latest/nblf_queue/growable/pooled/struct.PooledDynamicQueue.html) are strictly non-blocking and will never block the calling thread.

### Queue variants

- **Static queues**: fixed-capacity queues backed by static storage.
- **Allocated queues**: fixed-capacity queues backed by dynamically allocated storage, only available on feature `alloc`.
- **Dynamic queues**: dynamically resizeable queues, only available on feature `dynamic`.
- **Pooled Queues**: variants of other queues, which may store arbitrary types, only available on feature `pool`.

Non-pooled queues store items in atomically updated slots, restricting the stored items to small, pointer-like values.

### Usage

[`InlineQueue`](https://docs.rs/nblf-queue/latest/nblf_queue/array/queue/struct.InlineQueue.html):

```rust
  #[cfg(feature = "unsafe-ptr48")]
  fn run() {
    use nblf_queue::{InlineQueue, MPMCQueue};

    let q: InlineQueue<_, 2> = InlineQueue::new();

    assert!(q.push(&42).is_ok());
    assert!(q.push(&1).is_ok());
    assert!(q.push(&4242).is_err());

    assert_eq!(q.pop(), Some(&42));
    assert_eq!(q.pop(), Some(&1));
    assert!(q.pop().is_none());
  }

  #[cfg(feature = "unsafe-ptr48")]
  run();
```

[`PooledInlineQueue`](https://docs.rs/nblf-queue/latest/nblf_queue/array/queue/pooled_static/struct.PooledInlineQueue.html):

```rust
  #[cfg(feature = "pool")]
  fn run() {
    use nblf_queue::{PooledInlineQueue, MPMCQueue};

    let q: PooledInlineQueue<_, 2> = PooledInlineQueue::new();

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

[`DynamicQueue`](https://docs.rs/nblf-queue/latest/nblf_queue/growable/queue/struct.DynamicQueue.html):

```rust
  #[cfg(feature = "dynamic")]
  fn run() {
    use nblf_queue::{DynamicQueue, MPMCQueue, Resize};

    let q = DynamicQueue::new(1);

    assert!(q.push(42).is_ok());
    assert!(q.push(4242).is_err());

    assert!(q.resize(2));
    assert_eq!(q.capacity(), 2);
    assert!(q.push(4242).is_ok());

    assert_eq!(q.pop(), Some(42));
    assert_eq!(q.pop(), Some(4242));
    assert!(q.pop().is_none());
  }

  #[cfg(feature = "dynamic")]
  run();
```

### Choosing a queue type

Do you have an allocator? -> Use a non-static Queue.
Do you want to send large owned items? -> Use `Pooled*`.
Do you want to resize your queue? -> Use `Dynamic*`.

- [`InlineQueue`](https://docs.rs/nblf-queue/latest/nblf_queue/array/queue/struct.InlineQueue.html) and [`Queue`](https://docs.rs/nblf-queue/latest/nblf_queue/owned/queue/struct.Queue.html): may only store small values and are optimized for this use case.

- [`PooledInlineQueue`](https://docs.rs/nblf-queue/latest/nblf_queue/array/queue/pooled_static/struct.PooledInlineQueue.html) and [`PooledQueue`](https://docs.rs/nblf-queue/latest/nblf_queue/owned/queue/pooled_queue/struct.PooledQueue.html): may store arbitrary types, at the cost of higher memory usage and runtime cost.

- [`DynamicQueue`](https://docs.rs/nblf-queue/latest/nblf_queue/growable/queue/struct.DynamicQueue.html) and [`PooledDynamicQueue`](https://docs.rs/nblf-queue/latest/nblf_queue/growable/pooled/struct.PooledDynamicQueue.html): may be resized dynamically, at the cost of higher total memory usage and runtime cost. This cost is even higher for [`PooledDynamicQueue`](https://docs.rs/nblf-queue/latest/nblf_queue/growable/pooled/struct.PooledDynamicQueue.html).

> [!WARNING]
> **Blocking Behaviour in Dynamic Queues**
>
> All dynamic queues may block on concurrent `resize` operations.
>
> Additionally, dynamic queues may block on `pop` operations (and operations depending on it), if a `push` is preempted by a concurrent `resize` and a concurrent `pop` happens.

### Platform Support

Multiple storage types are available, dependent on platform:

- **Tagged64** - platforms with native 64-bit atomic operations or feature `atomic-fallback`.

- **Tagged128** - platforms with native 128-bit atomic operations or feature `atomic-fallback`.

Storage types will be chosen automatically, unless sepcified explicitly.

> [!NOTE]
> **ABA Safety & Storage Selection**
>
> If it is plausible that other threads could perform `(2^15 - 1) * queue_size`
> pop and push operations while a single thread is paused/preempted in pop/push, [`core::slots::Tagged128`](https://docs.rs/nblf-queue/latest/nblf_queue/core/slots/tagged128/struct.Tagged128.html) slots should be used to ensure ABA safety.
>
> **Tagged64 Safety**
>
> Sending ptr-types via [`core::slots::Tagged64`](https://docs.rs/nblf-queue/latest/nblf_queue/core/slots/tagged64/struct.Tagged64.html) slots is not safe if more than 48 bits are used for pointers.
> This is currently enforced with a runtime check, however some unsafe usages may be missed by this check.

### Feature Flags

- `std`: Enables `std` and `alloc` support.
- `alloc`: Enables `alloc` support, allowing usage of some dynamically allocated queues.
- `pool`: Enables pooled queues, which may store any type.
- `dynamic`: Enables dynamic queues, which may be dynamically resized. Depends on `alloc`.
- `atomic-fallback`: Uses `portable-atomic` `fallback` feature for atomics if necessary. It is discouraged to use this feature, as `fallback` internally uses locks.
- `unsafe-ptr48`: implements AsPackedValue for pointers on `x86-64` and `aarch64`. This feature is safe to use if 48 or less bits are used for pointers on the target platform.
- `default`: `pool`

### Python Bindings

Python bindings backed by [`PooledQueue`](https://docs.rs/nblf-queue/latest/nblf_queue/owned/queue/pooled_queue/struct.PooledQueue.html) and [`PooledDynamicQueue`](https://docs.rs/nblf-queue/latest/nblf_queue/growable/pooled/struct.PooledDynamicQueue.html) are available for concurrent applications.
Core operations detach from the GIL to allow parallel execution.

> [!NOTE]
> The Python bindings strictly use [`core::slots::Auto`](https://docs.rs/nblf-queue/latest/nblf_queue/core/slots/struct.Auto.html) slots without feature `atomic-fallback`.
> As a result, these bindings are only supported on platforms with native 64-bit or 128-bit atomic operations.

```python
  from nblf_queue import Queue, DynamicQueue

  q: Queue[int] = Queue(10)

  assert q.push(42) is None
  item = q.pop()
  assert item == 42

  dq: DynamicQueue[str] = DynamicQueue(1)

  assert dq.push("hello") is None
  assert dq.resize(42)
  assert dq.push("world") is None

```

### Testing

The core test-suite of this crate was adapted from [`crossbeam-queue`](https://!github.com/crossbeam-rs/crossbeam/tree/main/crossbeam-queue).

Current testing is based on:

- **Miri** - to validate pointer arithmetic and catch UB.
- **Loom and Shuttle** - to test for race conditions and blocking code.
- **Echeneis** - to test basic obstruction freedom.
- **ASan** - to check for memory corruption.

### References

Alexandre Denis, Charles Goedefroit. NBLFQ: a lock-free MPMC queue optimized for low contention.
IPDPS 2025 - 39th International Parallel & Distributed Processing Symposium, IEEE, Jun 2025,
Milan, Italy. hal-04851700v2

<!-- cargo-rdme end -->
