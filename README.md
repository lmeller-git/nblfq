# nblfq

An atomic wait-free MPMC queue based on the nblfq algorithm.

This repository provides two queue implementations:

- `HeaplessQueue`: a bounded, stack-allocated queue

- `HeapBackedQueue`: a bounded, heap allocated queue

## Usage

TODO


## Features

Default-features include `std` and `tagged-ptr`.

Both queues are compatible with no_std environmtns.

`HeapBackedQueue` requires the `alloc` feature.

## References
Alexandre Denis, Charles Goedefroit. NBLFQ: a lock-free MPMC queue optimized for low contention.
IPDPS 2025 - 39th International Parallel & Distributed Processing Symposium, IEEE, Jun 2025,
Milan, Italy. hal-04851700v2
