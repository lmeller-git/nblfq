use crate::{
    cfg_taggedptr64, cfg_taggedptr128,
    tests::test_library::{
        len, len_empty_full, linearizable, mpmc, mpmc_ring_buffer, mpsc, smoke, smoke_long, spsc,
    },
};

cfg_taggedptr64! {
mod tagged_ptr64 {
    use crate::{owned::buffer::BoxedBuffer, queue::QueueCore, slot::TaggedPtr64};

    use super::*;

    #[test]
    fn smoke_impl() {
        let q: QueueCore<BoxedBuffer<TaggedPtr64<_>>> = QueueCore::new_in(BoxedBuffer::new(2));
        smoke(q);
    }

    #[test]
    fn smoke_long_impl() {
        let q: QueueCore<BoxedBuffer<TaggedPtr64<_>>> = QueueCore::new_in(BoxedBuffer::new(10));
        smoke_long(q);
    }

    #[test]
    fn len_empty_full_impl() {
        let q: QueueCore<BoxedBuffer<TaggedPtr64<_>>> = QueueCore::new_in(BoxedBuffer::new(2));
        len_empty_full(q);
    }

    #[test]
    fn len_impl() {
        #[cfg(miri)]
        const CAP: usize = 40;
        #[cfg(not(miri))]
        const CAP: usize = 1000;

        let q: QueueCore<BoxedBuffer<TaggedPtr64<_>>> = QueueCore::new_in(BoxedBuffer::new(CAP));
        len(q);
    }

    #[test]
    fn spsc_impl() {
        let q: QueueCore<BoxedBuffer<TaggedPtr64<_>>> = QueueCore::new_in(BoxedBuffer::new(3));
        spsc(q);
    }

    #[test]
    fn mpsc_impl() {
        let q: QueueCore<BoxedBuffer<TaggedPtr64<_>>> = QueueCore::new_in(BoxedBuffer::new(3));
        mpsc(q);
    }

    #[test]
    fn mpmc_impl() {
        let q: QueueCore<BoxedBuffer<TaggedPtr64<_>>> = QueueCore::new_in(BoxedBuffer::new(3));
        mpmc(q);
    }

    #[test]
    fn mpmc_ring_buffer_impl() {
        let q: QueueCore<BoxedBuffer<TaggedPtr64<_>>> = QueueCore::new_in(BoxedBuffer::new(3));
        mpmc_ring_buffer(q);
    }

    #[test]
    fn linearizable_impl() {
        let q: QueueCore<BoxedBuffer<TaggedPtr64<_>>> = QueueCore::new_in(BoxedBuffer::new(4));
        linearizable(q);
    }
}
}

cfg_taggedptr128! {
mod tagged_ptr128 {
    use crate::{owned::buffer::BoxedBuffer, queue::QueueCore, slot::TaggedPtr128};

    use super::*;

    #[test]
    fn smoke_impl() {
        let q: QueueCore<BoxedBuffer<TaggedPtr128<_>>> = QueueCore::new_in(BoxedBuffer::new(2));
        smoke(q);
    }

    #[test]
    fn smoke_long_impl() {
        let q: QueueCore<BoxedBuffer<TaggedPtr128<_>>> = QueueCore::new_in(BoxedBuffer::new(10));
        smoke_long(q);
    }

    #[test]
    fn len_empty_full_impl() {
        let q: QueueCore<BoxedBuffer<TaggedPtr128<_>>> = QueueCore::new_in(BoxedBuffer::new(2));
        len_empty_full(q);
    }

    #[test]
    fn len_impl() {
        #[cfg(miri)]
        const CAP: usize = 40;
        #[cfg(not(miri))]
        const CAP: usize = 1000;

        let q: QueueCore<BoxedBuffer<TaggedPtr128<_>>> = QueueCore::new_in(BoxedBuffer::new(CAP));
        len(q);
    }

    #[test]
    fn spsc_impl() {
        let q: QueueCore<BoxedBuffer<TaggedPtr128<_>>> = QueueCore::new_in(BoxedBuffer::new(3));
        spsc(q);
    }

    #[test]
    fn mpsc_impl() {
        let q: QueueCore<BoxedBuffer<TaggedPtr128<_>>> = QueueCore::new_in(BoxedBuffer::new(3));
        mpsc(q);
    }

    #[test]
    fn mpmc_impl() {
        let q: QueueCore<BoxedBuffer<TaggedPtr128<_>>> = QueueCore::new_in(BoxedBuffer::new(3));
        mpmc(q);
    }

    #[test]
    fn mpmc_ring_buffer_impl() {
        let q: QueueCore<BoxedBuffer<TaggedPtr128<_>>> = QueueCore::new_in(BoxedBuffer::new(3));
        mpmc_ring_buffer(q);
    }

    #[test]
    fn linearizable_impl() {
        let q: QueueCore<BoxedBuffer<TaggedPtr128<_>>> = QueueCore::new_in(BoxedBuffer::new(4));
        linearizable(q);
    }
}
}

#[cfg(feature = "pool")]
mod pool {
    use super::*;
    use crate::array::StaticPooledQueue;

    #[test]
    fn smoke_impl() {
        let q: StaticPooledQueue<_, 2> = StaticPooledQueue::new();
        smoke(q);
    }

    #[test]
    fn smoke_long_impl() {
        let q: StaticPooledQueue<_, 10> = StaticPooledQueue::new();
        smoke_long(q);
    }

    #[test]
    fn len_empty_full_impl() {
        let q: StaticPooledQueue<_, 2> = StaticPooledQueue::new();
        len_empty_full(q);
    }

    #[test]
    fn len_impl() {
        #[cfg(miri)]
        const CAP: usize = 40;
        #[cfg(not(miri))]
        const CAP: usize = 1000;

        let q: StaticPooledQueue<_, CAP> = StaticPooledQueue::new();
        len(q);
    }

    #[test]
    fn spsc_impl() {
        let q: StaticPooledQueue<_, 3> = StaticPooledQueue::new();
        spsc(q);
    }

    #[test]
    fn mpsc_impl() {
        let q: StaticPooledQueue<_, 3> = StaticPooledQueue::new();
        mpsc(q);
    }

    #[test]
    fn mpmc_impl() {
        let q: StaticPooledQueue<_, 3> = StaticPooledQueue::new();
        mpmc(q);
    }

    #[test]
    fn mpmc_ring_buffer_impl() {
        let q: StaticPooledQueue<_, 3> = StaticPooledQueue::new();
        mpmc_ring_buffer(q);
    }

    #[test]
    fn linearizable_impl() {
        let q: StaticPooledQueue<_, 4> = StaticPooledQueue::new();
        linearizable(q);
    }
}

mod array {
    use crate::array::StaticQueue;

    use super::*;

    #[test]
    fn smoke_impl() {
        let q: StaticQueue<_, 2> = StaticQueue::new();
        smoke(q);
    }

    #[test]
    fn smoke_long_impl() {
        let q: StaticQueue<_, 10> = StaticQueue::new();
        smoke_long(q);
    }

    #[test]
    fn len_empty_full_impl() {
        let q: StaticQueue<_, 2> = StaticQueue::new();
        len_empty_full(q);
    }
}

#[cfg(feature = "alloc")]
mod owned {
    use crate::owned::Queue;

    use super::*;

    #[test]
    fn smoke_impl() {
        let q = Queue::new(2);
        smoke(q);
    }

    #[test]
    fn smoke_long_impl() {
        let q: Queue<_> = Queue::new(10);
        smoke_long(q);
    }

    #[test]
    fn len_empty_full_impl() {
        let q: Queue<_> = Queue::new(2);
        len_empty_full(q);
    }

    mod pool {
        use crate::owned::PooledQueue;

        use super::*;

        #[test]
        fn smoke_impl() {
            let q: PooledQueue<_> = PooledQueue::new(2);
            smoke(q);
        }

        #[test]
        fn smoke_long_impl() {
            let q: PooledQueue<_> = PooledQueue::new(10);
            smoke_long(q);
        }

        #[test]
        fn len_empty_full_impl() {
            let q: PooledQueue<_> = PooledQueue::new(2);
            len_empty_full(q);
        }
    }
}

// TODO
#[cfg(false)]
mod item_slot {
    use crate::slot::OwnedSlot;

    use super::*;

    #[test]
    fn smoke_impl() {
        let q: QueueCore<BoxedBuffer<OwnedSlot<_>>> = QueueCore::new_in(BoxedBuffer::new(2));
        smoke(q);
    }

    #[test]
    fn smoke_long_impl() {
        let q: QueueCore<BoxedBuffer<OwnedSlot<_>>> = QueueCore::new_in(BoxedBuffer::new(10));
        smoke_long(q);
    }

    #[test]
    fn len_empty_full_impl() {
        let q: QueueCore<BoxedBuffer<OwnedSlot<_>>> = QueueCore::new_in(BoxedBuffer::new(2));
        len_empty_full(q);
    }

    #[test]
    fn len_impl() {
        #[cfg(miri)]
        const CAP: usize = 40;
        #[cfg(not(miri))]
        const CAP: usize = 1000;

        let q: QueueCore<BoxedBuffer<OwnedSlot<_>>> = QueueCore::new_in(BoxedBuffer::new(CAP));
        len(q);
    }

    #[test]
    fn spsc_impl() {
        let q: QueueCore<BoxedBuffer<OwnedSlot<_>>> = QueueCore::new_in(BoxedBuffer::new(3));
        spsc(q);
    }

    #[test]
    fn mpsc_impl() {
        let q: QueueCore<BoxedBuffer<OwnedSlot<_>>> = QueueCore::new_in(BoxedBuffer::new(3));
        mpsc(q);
    }

    #[test]
    fn mpmc_impl() {
        let q: QueueCore<BoxedBuffer<OwnedSlot<_>>> = QueueCore::new_in(BoxedBuffer::new(3));
        mpmc(q);
    }

    #[test]
    fn mpmc_ring_buffer_impl() {
        let q: QueueCore<BoxedBuffer<OwnedSlot<_>>> = QueueCore::new_in(BoxedBuffer::new(3));
        mpmc_ring_buffer(q);
    }

    #[test]
    fn linearizable_impl() {
        let q: QueueCore<BoxedBuffer<OwnedSlot<_>>> = QueueCore::new_in(BoxedBuffer::new(4));
        linearizable(q);
    }
}
