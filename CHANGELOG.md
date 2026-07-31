# Version 0.3.0

- [ADDED] [BREAKING] feature `unsafe-ptr48`, tagged 64 bit pointers are now opt-in.
- [FIXED] Changed semantics of `DynamicQueue` to properly exhibit `k-FIFO` ordering and `empty-linearizabilty`.
- [FIXED] empty-linearizabilty violations in `DynamicQueue`, introduced in `0.2.1`.
- [FIXED] potential use-after-free in `DynamicQueue` during epoch reclamation.


