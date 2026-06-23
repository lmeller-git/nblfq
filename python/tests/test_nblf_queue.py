from nblf_queue import Queue, DynamicQueue


def test_queue_basic_push_pop():
    q: Queue[int] = Queue(5)

    assert q.is_empty()
    assert not q.is_full()
    assert q.len() == 0
    assert q.capacity() == 5

    assert q.push(10) is None
    assert q.push(20) is None

    assert q.len() == 2
    assert not q.is_empty()

    assert q.pop() == 10
    assert q.pop() == 20

    assert q.pop() is None
    assert q.is_empty()


def test_queue_capacity_limits():
    q: Queue[str] = Queue(2)

    assert q.push("A") is None
    assert q.push("B") is None

    assert q.is_full()
    assert q.len() == 2

    rejected = q.push("C")
    assert rejected == "C"


def test_queue_force_push():
    q: Queue[int] = Queue(2)

    assert q.push(1) is None
    assert q.push(2) is None

    popped = q.force_push(3)

    assert popped == 1
    assert q.len() == 2
    assert q.pop() == 2
    assert q.pop() == 3


def test_queue_force_push_and_do():
    q: Queue[int] = Queue(2)
    assert q.push(1) is None
    assert q.push(2) is None

    evicted: list[int] = []

    def callback(item: int) -> None:
        evicted.append(item)

    q.force_push_and_do(3, callback)
    q.force_push_and_do(4, callback)

    assert evicted == [1, 2]
    assert q.pop() == 3
    assert q.pop() == 4


def test_dynamic_queue_growth():
    q: DynamicQueue[int] = DynamicQueue(2)

    assert q.push(1) is None
    assert q.push(2) is None
    assert q.is_full()
    assert q.capacity() == 2

    grown = q.grow()
    assert grown is True
    assert q.capacity() > 2
    assert not q.is_full()

    assert q.push(3) is None
    assert q.push(4) is None
    assert q.len() == 4


def test_dynamic_queue_grow_by():
    q: DynamicQueue[int] = DynamicQueue(2)
    assert q.capacity() == 2

    grown = q.resize(5 + q.capacity())
    assert grown is True
    assert q.capacity() == 7
