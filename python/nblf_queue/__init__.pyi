from typing import Callable, Generic, TypeVar


_T = TypeVar("_T")


class DynamicQueue(Generic[_T]):
    """
    A dynamically growable, lock-free non-blocking MPMC queue.

    Core operations detach from the python GIL, to ensure concurrent performance.
    """

    def __init__(self, size: int) -> None:
        """
        Constructs a new `DynamicQueue` with initial capacity `size`.
        """
        ...

    def push(self, item: _T) -> _T | None:
        """
        Attempts to push an element into the queue.

        Returns the item, if the queue was full.
        """
        ...

    def pop(self) -> _T | None:
        """
        Attempts to pop an item from the queue.

        Returns `None` if the queue was empty.
        """
        ...

    def len(self) -> int:
        """
        Returns the current length of the queue.

        This method should not be used for synchronization.
        """
        ...

    def capacity(self) -> int:
        """
        Returns the current capacity of the queue.

        This method should not be used for synchronization.
        """
        ...

    def is_empty(self) -> bool:
        """
        Indicates whether the queue is currently empty.

        This method should not be used for synchronization.
        """
        ...

    def is_full(self) -> bool:
        """
        Indicates whether the queue is currently full.

        This method should not be used for synchronization.
        """
        ...

    def force_push(self, item: _T) -> _T | None:
        """
        Pushes an item into the queue.
        This method may take some time under heavy contention.
        This method may pop an arbitrary amount of items from the queue.

        Returns the last popped item, if the queue was full. All other items are dropped.
        """
        ...

    def force_push_and_do(self, item: _T, f: Callable[[_T], None]) -> None:
        """
        Pushes an item into the queue.
        This method may take some time under heavy contention.
        This method may pop an arbitrary amount of items from the queue.
        Applies `f` to each popped item.
        """
        ...

    def grow_by(self, by: int) -> bool:
        """
        Attempts to grow the queue's capacity by `by` items.

        This method may spuriously fail.

        Returns `True` if the capacity was successfully grown.
        """
        ...

    def grow(self) -> bool:
        """
        Attempts to grow the queue's capacity using limited exponential growth.

        This method may spuriously fail.

        Returns `True` if the capacity was successfully grown.
        """
        ...


class Queue(Generic[_T]):
    """
    A lock-free non-blocking MPMC queue.

    Core operations detach from the python GIL, to ensure concurrent performance.
    """

    def __init__(self, size: int) -> None:
        """
        Constructs a new `Queue` with initial capacity `size`.
        """
        ...

    def push(self, item: _T) -> _T | None:
        """
        Attempts to push an element into the queue.

        Returns the item, if the queue was full.
        """
        ...

    def pop(self) -> _T | None:
        """
        Attempts to pop an item from the queue.

        Returns `None` if the queue was empty.
        """
        ...

    def len(self) -> int:
        """
        Returns the current length of the queue.

        This method should not be used for synchronization.
        """
        ...

    def capacity(self) -> int:
        """
        Returns the current capacity of the queue.

        This method should not be used for synchronization.
        """
        ...

    def is_empty(self) -> bool:
        """
        Indicates whether the queue is currently empty.

        This method should not be used for synchronization.
        """
        ...

    def is_full(self) -> bool:
        """
        Indicates whether the queue is currently full.

        This method should not be used for synchronization.
        """
        ...

    def force_push(self, item: _T) -> _T | None:
        """
        Pushes an item into the queue.
        This method may take some time under heavy contention.
        This method may pop an arbitrary amount of items from the queue.

        Returns the last popped item, if the queue was full. All other items are dropped.
        """
        ...

    def force_push_and_do(self, item: _T, f: Callable[[_T], None]) -> None:
        """
        Pushes an item into the queue.
        This method may take some time under heavy contention.
        This method may pop an arbitrary amount of items from the queue.
        Applies `f` to each popped item.
        """
        ...
