# nblfq

An atomic wait-free MPMC queue based on the nblfq algorithm.

This repository provides two queue implementations:

- `HeaplessQueue`: a bounded, stack-allocated queue

- `HeapBackedQueue`: a bounded, heap allocated queue


## Usage

`HeaplessQueue`:

```
  let q: HeaplessQueue<10, i32> = HeaplessQueue::new();

  assert!(q.push(&42).is_ok());
  assert!(q.push(&1).is_ok());

  assert_eq!(q.pop(), Some(&42));
  assert_eq!(q.pop(), Some(&1)); 
```


`HeapBackedQueue`:

```
  let q: HeapBackedQueue<i32> = HeapBackedQueue::new(10);

  assert!(q.push(42).is_ok());
  assert!(q.push(1).is_ok());

  assert_eq!(q.pop(), Some(42));
  assert_eq!(q.pop(), Some(1));
```


## Features

Default-features include `std` and `tagged_ptr`.

Currently `tagged_ptr` must be enabled.

Both queues are compatible with no_std environmtns.

`HeapBackedQueue` requires the `alloc` feature.


## References
Alexandre Denis, Charles Goedefroit. NBLFQ: a lock-free MPMC queue optimized for low contention.
IPDPS 2025 - 39th International Parallel & Distributed Processing Symposium, IEEE, Jun 2025,
Milan, Italy. hal-04851700v2
