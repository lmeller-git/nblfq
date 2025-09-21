# nblfq

An atomic wait-free MPMC queue based on the NBLFQ algorithm.

This repository provides two queue implementations:

- `HeaplessQueue`: A bounded, stack-allocated queue

- `HeapBackedQueue`: A bounded, heap-allocated queue


## Usage

`HeaplessQueue`:

```rust
  use nblfq::HeaplessQueue;
  
  let q: HeaplessQueue<10, i32> = HeaplessQueue::new();

  assert!(q.push(&42).is_ok());
  assert!(q.push(&1).is_ok());

  assert_eq!(q.pop(), Some(&42));
  assert_eq!(q.pop(), Some(&1)); 
```


`HeapBackedQueue`:

```rust
  use nblfq::HeapBackedQueue;
  
  let q: HeapBackedQueue<i32> = HeapBackedQueue::new(10);

  assert!(q.push(42).is_ok());
  assert!(q.push(1).is_ok());

  assert_eq!(q.pop(), Some(42));
  assert_eq!(q.pop(), Some(1));
```


## Platform Support

Multiple storage types are available, dependant on platform:

- **Tagged ptr** - 64-bit platforms with 48-bit virtual addresses

- **AtomicU128** - platforms with native atomic 128-bit support (crate protable-atomic)


## Feature Flags

- `std` (default): Enables `std` and `alloc` support

- `alloc`: Enables `alloc` support (required for `HeapBackedQueue`)

- `no-tagged-ptr`: Disables the default storage type (`Tagged ptr`), and replaces it with portable-atomic AtomicU128. (This is currently untested)


## References

Alexandre Denis, Charles Goedefroit. NBLFQ: a lock-free MPMC queue optimized for low contention.
IPDPS 2025 - 39th International Parallel & Distributed Processing Symposium, IEEE, Jun 2025,
Milan, Italy. hal-04851700v2
