use crate::tests::test_library::{
    len, len_empty_full, linearizable, mpmc, mpmc_ring_buffer, mpsc, smoke, smoke_long, spsc,
};

cfg_atomic_tagged64! {
    mod tagged_ptr64 {
        use crate::{owned::buffer::BoxedBuffer, core::{queue::QueueCore, slot::Tagged64}};

        use super::*;

        #[test]
        fn smoke_impl() {
            let q: QueueCore<BoxedBuffer<Tagged64<_>>> = QueueCore::new_in(BoxedBuffer::new(2));
            smoke(q);
        }

        #[test]
        fn smoke_long_impl() {
            let q: QueueCore<BoxedBuffer<Tagged64<_>>> = QueueCore::new_in(BoxedBuffer::new(10));
            smoke_long(q);
        }

        #[test]
        fn len_empty_full_impl() {
            let q: QueueCore<BoxedBuffer<Tagged64<_>>> = QueueCore::new_in(BoxedBuffer::new(2));
            len_empty_full(q);
        }

        #[test]
        fn len_impl() {
            #[cfg(miri)]
            const CAP: usize = 40;
            #[cfg(not(miri))]
            const CAP: usize = 1000;

            let q: QueueCore<BoxedBuffer<Tagged64<_>>> = QueueCore::new_in(BoxedBuffer::new(CAP));
            len(q);
        }

        #[test]
        fn spsc_impl() {
            let q: QueueCore<BoxedBuffer<Tagged64<_>>> = QueueCore::new_in(BoxedBuffer::new(3));
            spsc(q);
        }

        #[test]
        fn mpsc_impl() {
            let q: QueueCore<BoxedBuffer<Tagged64<_>>> = QueueCore::new_in(BoxedBuffer::new(3));
            mpsc(q);
        }

        #[test]
        fn mpmc_impl() {
            let q: QueueCore<BoxedBuffer<Tagged64<_>>> = QueueCore::new_in(BoxedBuffer::new(3));
            mpmc(q);
        }

        #[test]
        fn mpmc_ring_buffer_impl() {
            let q: QueueCore<BoxedBuffer<Tagged64<_>>> = QueueCore::new_in(BoxedBuffer::new(3));
            mpmc_ring_buffer(q);
        }

        #[test]
        fn linearizable_impl() {
            let q: QueueCore<BoxedBuffer<Tagged64<_>>> = QueueCore::new_in(BoxedBuffer::new(4));
            linearizable(q);
        }
    }
}

cfg_atomic_tagged128! {
    mod tagged_ptr128 {
        use crate::{owned::buffer::BoxedBuffer, core::{queue::QueueCore, slot::Tagged128}};

        use super::*;

        #[test]
        fn smoke_impl() {
            let q: QueueCore<BoxedBuffer<Tagged128<_>>> = QueueCore::new_in(BoxedBuffer::new(2));
            smoke(q);
        }

        #[test]
        fn smoke_long_impl() {
            let q: QueueCore<BoxedBuffer<Tagged128<_>>> = QueueCore::new_in(BoxedBuffer::new(10));
            smoke_long(q);
        }

        #[test]
        fn len_empty_full_impl() {
            let q: QueueCore<BoxedBuffer<Tagged128<_>>> = QueueCore::new_in(BoxedBuffer::new(2));
            len_empty_full(q);
        }

        #[test]
        fn len_impl() {
            #[cfg(miri)]
            const CAP: usize = 40;
            #[cfg(not(miri))]
            const CAP: usize = 1000;

            let q: QueueCore<BoxedBuffer<Tagged128<_>>> = QueueCore::new_in(BoxedBuffer::new(CAP));
            len(q);
        }

        #[test]
        fn spsc_impl() {
            let q: QueueCore<BoxedBuffer<Tagged128<_>>> = QueueCore::new_in(BoxedBuffer::new(3));
            spsc(q);
        }

        #[test]
        fn mpsc_impl() {
            let q: QueueCore<BoxedBuffer<Tagged128<_>>> = QueueCore::new_in(BoxedBuffer::new(3));
            mpsc(q);
        }

        #[test]
        fn mpmc_impl() {
            let q: QueueCore<BoxedBuffer<Tagged128<_>>> = QueueCore::new_in(BoxedBuffer::new(3));
            mpmc(q);
        }

        #[test]
        fn mpmc_ring_buffer_impl() {
            let q: QueueCore<BoxedBuffer<Tagged128<_>>> = QueueCore::new_in(BoxedBuffer::new(3));
            mpmc_ring_buffer(q);
        }

        #[test]
        fn linearizable_impl() {
            let q: QueueCore<BoxedBuffer<Tagged128<_>>> = QueueCore::new_in(BoxedBuffer::new(4));
            linearizable(q);
        }
    }
}

#[cfg(feature = "pool")]
mod pool {
    use super::*;
    use crate::array::PooledStaticQueue;

    #[test]
    fn smoke_impl() {
        let q: PooledStaticQueue<_, 2> = PooledStaticQueue::new();
        smoke(q);
    }

    #[test]
    fn smoke_long_impl() {
        let q: PooledStaticQueue<_, 10> = PooledStaticQueue::new();
        smoke_long(q);
    }

    #[test]
    fn len_empty_full_impl() {
        let q: PooledStaticQueue<_, 2> = PooledStaticQueue::new();
        len_empty_full(q);
    }

    #[test]
    fn len_impl() {
        #[cfg(miri)]
        const CAP: usize = 40;
        #[cfg(not(miri))]
        const CAP: usize = 1000;

        let q: PooledStaticQueue<_, CAP> = PooledStaticQueue::new();
        len(q);
    }

    #[test]
    fn spsc_impl() {
        let q: PooledStaticQueue<_, 3> = PooledStaticQueue::new();
        spsc(q);
    }

    #[test]
    fn mpsc_impl() {
        let q: PooledStaticQueue<_, 3> = PooledStaticQueue::new();
        mpsc(q);
    }

    #[test]
    fn mpmc_impl() {
        let q: PooledStaticQueue<_, 3> = PooledStaticQueue::new();
        mpmc(q);
    }

    #[test]
    fn mpmc_ring_buffer_impl() {
        let q: PooledStaticQueue<_, 3> = PooledStaticQueue::new();
        mpmc_ring_buffer(q);
    }

    #[test]
    fn linearizable_impl() {
        let q: PooledStaticQueue<_, 4> = PooledStaticQueue::new();
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

    #[cfg(feature = "pool")]
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
