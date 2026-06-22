import nblf_queue


def test_lib_imports():
    assert nblf_queue is not None


def test_queues_import():
    assert nblf_queue.DynamicQueue is not None
    assert nblf_queue.Queue is not None
