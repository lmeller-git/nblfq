use crate::tests::test_library::{
    force_push,
    len,
    len_empty_full,
    linearizable,
    mpmc,
    mpmc_ring_buf_ptr,
    mpmc_ring_buffer,
    mpsc,
    smoke,
    smoke_long,
    spsc,
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

        #[cfg(any(target_arch = "x86_64", target_arch = "aarch64", not(target_pointer_width = "64")))]
        #[test]
        fn mpmc_ring_buf_ptr_impl() {
            let q: QueueCore<BoxedBuffer<Tagged64<_>>> = QueueCore::new_in(BoxedBuffer::new(4));
            mpmc_ring_buf_ptr(q);
        }

        #[test]
        fn force_push_impl() {
            let q: QueueCore<BoxedBuffer<Tagged64<_>>> = QueueCore::new_in(BoxedBuffer::new(4));
            force_push(q);
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

        #[test]
        fn mpmc_ring_buf_ptr_impl() {
            let q: QueueCore<BoxedBuffer<Tagged128<_>>> = QueueCore::new_in(BoxedBuffer::new(4));
            mpmc_ring_buf_ptr(q);
        }

        #[test]
        fn force_push_impl() {
            let q: QueueCore<BoxedBuffer<Tagged128<_>>> = QueueCore::new_in(BoxedBuffer::new(4));
            force_push(q);
        }

    }
}

#[cfg(feature = "pool")]
mod pool {
    use super::*;
    use crate::array::PooledStaticQueue;

    #[test]
    fn smoke_impl() {
        let q: PooledStaticQueue<_, 2> = PooledStaticQueue::default();
        smoke(q);
    }

    #[test]
    fn smoke_long_impl() {
        let q: PooledStaticQueue<_, 10> = PooledStaticQueue::default();
        smoke_long(q);
    }

    #[test]
    fn len_empty_full_impl() {
        let q: PooledStaticQueue<_, 2> = PooledStaticQueue::default();
        len_empty_full(q);
    }

    #[test]
    fn len_impl() {
        #[cfg(miri)]
        const CAP: usize = 40;
        #[cfg(not(miri))]
        const CAP: usize = 1000;

        let q: PooledStaticQueue<_, CAP> = PooledStaticQueue::default();
        len(q);
    }

    #[test]
    fn spsc_impl() {
        let q: PooledStaticQueue<_, 3> = PooledStaticQueue::default();
        spsc(q);
    }

    #[test]
    fn mpsc_impl() {
        let q: PooledStaticQueue<_, 3> = PooledStaticQueue::default();
        mpsc(q);
    }

    #[test]
    fn mpmc_impl() {
        let q: PooledStaticQueue<_, 3> = PooledStaticQueue::default();
        mpmc(q);
    }

    #[test]
    fn mpmc_ring_buffer_impl() {
        let q: PooledStaticQueue<_, 3> = PooledStaticQueue::default();
        mpmc_ring_buffer(q);
    }

    #[test]
    fn linearizable_impl() {
        let q: PooledStaticQueue<_, 4> = PooledStaticQueue::default();
        linearizable(q);
    }

    #[test]
    fn force_push_impl() {
        let q: PooledStaticQueue<_, 4> = PooledStaticQueue::default();
        force_push(q);
    }
}

mod array {
    use super::*;
    use crate::array::StaticQueue;

    #[test]
    fn smoke_impl() {
        let q: StaticQueue<_, 2> = StaticQueue::default();
        smoke(q);
    }

    #[test]
    fn smoke_long_impl() {
        let q: StaticQueue<_, 10> = StaticQueue::default();
        smoke_long(q);
    }

    #[test]
    fn len_empty_full_impl() {
        let q: StaticQueue<_, 2> = StaticQueue::default();
        len_empty_full(q);
    }

    #[test]
    fn force_push_impl() {
        let q: StaticQueue<_, 2> = StaticQueue::default();
        force_push(q);
    }
}

#[cfg(feature = "alloc")]
mod owned {
    use super::*;
    use crate::owned::Queue;

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

    #[test]
    fn force_push_impl() {
        let q: Queue<_> = Queue::new(2);
        force_push(q);
    }

    #[cfg(feature = "pool")]
    mod pool {
        use super::*;
        use crate::owned::PooledQueue;

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

        #[test]
        fn force_push_impl() {
            let q: PooledQueue<_> = PooledQueue::new(2);
            force_push(q);
        }
    }
}
