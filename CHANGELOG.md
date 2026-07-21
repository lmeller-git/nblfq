# Version 0.3.0

- [ADDED] [BREAKING] feature `unsafe-ptr48`, tagged 64 bit pointers are now opt-in.
- [CHANGED] [BREAKING] Revert a regression introduced in `0.2.1`, which changed `DynamicQueue::pop` to be strictly non-blocking, but violated linearizability
- [FIXED] linearizabilty violations in `DynamicQueue`, introduced in `0.2.1`.
- [FIXED] potential use-after-free in `DynamicQueue` during epoch reclamation.
- [CHANGED] [BREAKING] changed API for `PooledStaticQueue` construction.
- [CHANGED] changed the backend for pool storage in `Pooled` queues. Index storage is now based on crate `lf-slots`, improving both memory footprint and performance.
- [ADDED] added API to define storage backends for `Pooled` queues in line with `lf-slots` API.
