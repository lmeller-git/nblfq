from __future__ import annotations

import threading
from queue import Queue as StdQueue, SimpleQueue
from typing import Callable
import pytest
from nblf_queue import Queue, DynamicQueue
import time

ITEMS_PER_THREAD = 30_000
NUM_THREADS = 4
QUEUE_CAPACITY = 1_000


def run_mpmc(push_callback: Callable[[int], None], pop_callback: Callable[[], None]):
    def producer():
        for i in range(ITEMS_PER_THREAD):
            push_callback(i)

    def consumer():
        for _ in range(ITEMS_PER_THREAD):
            pop_callback()

    producers = [threading.Thread(target=producer) for _ in range(NUM_THREADS)]
    consumers = [threading.Thread(target=consumer) for _ in range(NUM_THREADS)]

    for t in producers + consumers:
        t.start()

    for t in producers + consumers:
        t.join()


def pop_loop(callback: Callable[[], None | int]):
    while callback() is None:
        time.sleep(0)
        pass


def push_loop(item: int, callback: Callable[[int], int | None]):
    while callback(item) is not None:
        time.sleep(0)
        pass


@pytest.mark.benchmark(group="mpmc-contention")
@pytest.mark.parametrize("queue_name", ["SimpleQueue", "StdQueue", "Queue", "DynamicQueue"])
def test_queue_throughput(benchmark, queue_name: str):
    if queue_name == "SimpleQueue":
        s_queue: SimpleQueue[int] = SimpleQueue()
        benchmark(run_mpmc, s_queue.put, s_queue.get)

    elif queue_name == "StdQueue":
        st_queue: StdQueue[int] = StdQueue(maxsize=QUEUE_CAPACITY)
        benchmark(run_mpmc, st_queue.put, st_queue.get)

    elif queue_name == "Queue":
        n_queue: Queue[int] = Queue(QUEUE_CAPACITY)
        benchmark(run_mpmc, lambda x: push_loop(x, n_queue.push), lambda: pop_loop(n_queue.pop))

    elif queue_name == "DynamicQueue":

        def push_and_grow(q: DynamicQueue[int], item: int) -> int | None:
            ret = q.push(item)
            if ret is not None:
                _ = q.grow()
            return ret

        d_queue: DynamicQueue[int] = DynamicQueue(QUEUE_CAPACITY)
        benchmark(
            run_mpmc,
            lambda x: push_loop(x, lambda x: push_and_grow(d_queue, x)),
            lambda: pop_loop(d_queue.pop),
        )
