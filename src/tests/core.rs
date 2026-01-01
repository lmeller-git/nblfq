use portable_atomic::cfg_has_atomic_128;

use crate::{
    owned::buffer::BoxedBuffer,
    queue::QueueCore,
    tests::test_library::{
        len, len_empty_full, linearizable, mpmc, mpmc_ring_buffer, mpsc, smoke, smoke_long, spsc,
    },
};

#[cfg(all(feature = "tagged-ptr", target_has_atomic = "64"))]
mod tagged_ptr64 {
    use crate::slot::TaggedPtr64;

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

cfg_has_atomic_128! {
    mod tagged_ptr128 {
        use crate::slot::TaggedPtr128;

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

// TODO fix pool
// currently it fails lineraizable + len -> complete failure
#[cfg(false)]
mod pool {
    use super::*;
    use crate::pool::StaticPooledQueue;

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
