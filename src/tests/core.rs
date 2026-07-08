use crate::tests::test_library::{
    MaliciousCargo,
    force_push,
    len,
    len_empty_full,
    linearizable,
    mpmc,
    mpmc_ring_buffer,
    mpsc,
    smoke,
    smoke_long,
    spsc,
};
#[cfg(any(
    all(target_arch = "x86_64", feature = "unsafe-ptr48"),
    all(target_arch = "aarch64", feature = "unsafe-ptr48"),
    not(target_pointer_width = "64"),
    any(target_has_atomic = "128", feature = "atomic-fallback")
))]
use crate::tests::test_library::{drops, mpmc_ring_buf_ptr};

cfg_atomic_tagged64! {
    mod tagged_ptr64 {
        use super::*;
        use crate::{
            core::{queue::QueueCore, slot::Tagged64},
            owned::buffer::BoxedBuffer,
        };

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

        #[cfg(any(
            all(target_arch = "x86_64", feature = "unsafe-ptr48"),
            all(target_arch = "aarch64", feature = "unsafe-ptr48"),
            not(target_pointer_width = "64")
        ))]
        #[test]
        fn drops_impl() {
            let q: QueueCore<BoxedBuffer<Tagged64<_>>> = QueueCore::new_in(BoxedBuffer::new(2));
            drops(q);
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

        #[cfg(any(
            all(target_arch = "x86_64", feature = "unsafe-ptr48"),
            all(target_arch = "aarch64", feature = "unsafe-ptr48"),
            not(target_pointer_width = "64")
        ))]
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

        #[test]
        #[should_panic]
        fn malicious_cargo_impl() {
            let _q: QueueCore<BoxedBuffer<Tagged64<MaliciousCargo>>> =
                QueueCore::new_in(BoxedBuffer::new(4));
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
        fn drops_impl() {
            let q: QueueCore<BoxedBuffer<Tagged128<_>>> = QueueCore::new_in(BoxedBuffer::new(2));
            drops(q);
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

        #[test]
        #[should_panic]
        fn malicious_cargo_impl() {
            let _q: QueueCore<BoxedBuffer<Tagged128<MaliciousCargo>>> = QueueCore::new_in(BoxedBuffer::new(4));
        }
    }
}

#[cfg(feature = "pool")]
mod pool {
    use super::*;
    use crate::{array::PooledStaticQueue, tests::test_library::pooled_stores_any};

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

    #[cfg(any(
        all(target_arch = "x86_64", feature = "unsafe-ptr48"),
        all(target_arch = "aarch64", feature = "unsafe-ptr48"),
        not(target_pointer_width = "64")
    ))]
    #[test]
    fn drops_impl() {
        let q: PooledStaticQueue<_, 2> = PooledStaticQueue::default();
        drops(q);
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

    #[test]
    fn pooled_stores_any_impl() {
        let q: PooledStaticQueue<_, 1> = PooledStaticQueue::default();
        pooled_stores_any(q);
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
        use crate::{owned::PooledQueue, tests::test_library::pooled_stores_any};

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

        #[test]
        fn smoke_mismatched_size_impl() {
            let q: PooledQueue<_> = PooledQueue::new_with_arena_size(2, 3);
            smoke(q);
        }

        #[test]
        fn smoke_long_mismatched_size_impl() {
            let q: PooledQueue<_> = PooledQueue::new_with_arena_size(10, 12);
            smoke_long(q);
        }

        #[test]
        fn len_empty_full_mismatched_size_impl() {
            let q: PooledQueue<_> = PooledQueue::new_with_arena_size(2, 5);
            len_empty_full(q);
        }

        #[test]
        fn force_push_mismatched_size_impl() {
            let q: PooledQueue<_> = PooledQueue::new_with_arena_size(2, 4);
            force_push(q);
        }

        #[test]
        fn pooled_stores_any_impl() {
            let q = PooledQueue::new(1);
            pooled_stores_any(q);
        }
    }
}

#[cfg(feature = "dynamic")]
mod growable {
    use super::*;
    #[cfg(any(
        all(target_arch = "x86_64", feature = "unsafe-ptr48"),
        all(target_arch = "aarch64", feature = "unsafe-ptr48"),
        not(target_pointer_width = "64")
    ))]
    use crate::tests::test_library::drops_resized;
    use crate::{
        DynamicQueue,
        tests::test_library::{
            grow_storm,
            len_grow,
            mpmc_resize,
            mpsc_grow,
            oscillation_grow,
            smoke_grow,
            smoke_shrink,
            suppl_methods_chaos,
        },
    };

    #[test]
    fn smoke_impl() {
        let q: DynamicQueue<_> = DynamicQueue::new(2);
        smoke(q);
    }

    #[test]
    fn smoke_shrink_impl() {
        let q = DynamicQueue::new(2);
        smoke_shrink(q);
    }

    #[test]
    fn smoke_long_impl() {
        let q: DynamicQueue<_> = DynamicQueue::new(10);
        smoke_long(q);
    }

    #[test]
    fn len_empty_full_impl() {
        let q: DynamicQueue<_> = DynamicQueue::new(2);
        len_empty_full(q);
    }

    #[cfg(any(
        all(target_arch = "x86_64", feature = "unsafe-ptr48"),
        all(target_arch = "aarch64", feature = "unsafe-ptr48"),
        not(target_pointer_width = "64")
    ))]
    #[test]
    fn drops_impl() {
        let q: DynamicQueue<_> = DynamicQueue::new(2);
        drops(q);
    }

    #[cfg(any(
        all(target_arch = "x86_64", feature = "unsafe-ptr48"),
        all(target_arch = "aarch64", feature = "unsafe-ptr48"),
        not(target_pointer_width = "64")
    ))]
    #[test]
    fn drops_resized_impl() {
        let q: DynamicQueue<_> = DynamicQueue::new(2);
        drops_resized(q);
    }

    #[test]
    fn len_impl() {
        #[cfg(miri)]
        const CAP: usize = 40;
        #[cfg(not(miri))]
        const CAP: usize = 1000;

        let q: DynamicQueue<_> = DynamicQueue::new(CAP);
        len(q);
    }

    #[test]
    fn spsc_impl() {
        let q: DynamicQueue<_> = DynamicQueue::new(3);
        spsc(q);
    }

    #[test]
    fn mpsc_impl() {
        let q: DynamicQueue<_> = DynamicQueue::new(3);
        mpsc(q);
    }

    #[test]
    fn mpmc_impl() {
        let q: DynamicQueue<_> = DynamicQueue::new(3);
        mpmc(q);
    }

    #[test]
    fn mpmc_ring_buffer_impl() {
        let q: DynamicQueue<_> = DynamicQueue::new(3);
        mpmc_ring_buffer(q);
    }

    #[test]
    fn linearizable_impl() {
        let q: DynamicQueue<_> = DynamicQueue::new(4);
        linearizable(q);
    }

    #[cfg(any(
        all(target_arch = "x86_64", feature = "unsafe-ptr48"),
        all(target_arch = "aarch64", feature = "unsafe-ptr48"),
        not(target_pointer_width = "64")
    ))]
    #[test]
    fn mpmc_ring_buf_ptr_impl() {
        let q: DynamicQueue<_> = DynamicQueue::new(4);
        mpmc_ring_buf_ptr(q);
    }

    #[test]
    fn force_push_impl() {
        let q: DynamicQueue<_> = DynamicQueue::new(4);
        force_push(q);
    }

    #[test]
    #[should_panic]
    fn malicious_cargo_impl() {
        let _q: DynamicQueue<MaliciousCargo> = DynamicQueue::new(4);
    }

    #[test]
    fn smoke_grow_impl() {
        let q = DynamicQueue::new(4);
        smoke_grow(q);
    }

    #[test]
    fn mpsc_grow_impl() {
        let q = DynamicQueue::new(4);
        mpsc_grow(q);
    }

    #[test]
    fn mpmc_resize_impl() {
        let q = DynamicQueue::new(4);
        mpmc_resize(q);
    }

    #[test]
    fn len_grow_impl() {
        #[cfg(miri)]
        const CAP: usize = 40;
        #[cfg(not(miri))]
        const CAP: usize = 500;
        let q = DynamicQueue::new(CAP);
        len_grow(q);
    }

    #[test]
    fn grow_storm_impl() {
        let q = DynamicQueue::new(2);
        grow_storm(q);
    }

    #[test]
    fn oscillation_grow_impl() {
        let q = DynamicQueue::new(2);
        oscillation_grow(q);
    }

    #[test]
    fn suppl_methods_chaos_impl() {
        let q = DynamicQueue::new(2);
        suppl_methods_chaos(q);
    }

    #[cfg(feature = "pool")]
    mod pool {
        use super::*;
        use crate::{PooledDynamicQueue, tests::test_library::pooled_stores_any};

        #[test]
        fn smoke_impl() {
            let q: PooledDynamicQueue<_> = PooledDynamicQueue::new(2);
            smoke(q);
        }

        #[test]
        fn smoke_shrink_impl() {
            let q = PooledDynamicQueue::new(2);
            smoke_shrink(q);
        }

        #[test]
        fn smoke_long_impl() {
            let q: PooledDynamicQueue<_> = PooledDynamicQueue::new(10);
            smoke_long(q);
        }

        #[cfg(any(
            all(target_arch = "x86_64", feature = "unsafe-ptr48"),
            all(target_arch = "aarch64", feature = "unsafe-ptr48"),
            not(target_pointer_width = "64")
        ))]
        #[test]
        fn drops_impl() {
            let q: PooledDynamicQueue<_> = PooledDynamicQueue::new(2);
            drops(q);
        }

        #[cfg(any(
            all(target_arch = "x86_64", feature = "unsafe-ptr48"),
            all(target_arch = "aarch64", feature = "unsafe-ptr48"),
            not(target_pointer_width = "64")
        ))]
        #[test]
        fn drops_resized_impl() {
            let q: PooledDynamicQueue<_> = PooledDynamicQueue::new(2);
            drops_resized(q);
        }

        #[test]
        fn len_empty_full_impl() {
            let q: PooledDynamicQueue<_> = PooledDynamicQueue::new(2);
            len_empty_full(q);
        }

        #[test]
        fn force_push_impl() {
            let q: PooledDynamicQueue<_> = PooledDynamicQueue::new(2);
            force_push(q);
        }

        #[test]
        fn smoke_grow_impl() {
            let q = PooledDynamicQueue::new(4);
            smoke_grow(q);
        }

        #[test]
        fn mpsc_grow_impl() {
            let q = PooledDynamicQueue::new(4);
            mpsc_grow(q);
        }

        #[test]
        fn mpmc_resize_impl() {
            let q = PooledDynamicQueue::new(4);
            mpmc_resize(q);
        }

        #[test]
        fn len_grow_impl() {
            #[cfg(miri)]
            const CAP: usize = 40;
            #[cfg(not(miri))]
            const CAP: usize = 500;
            let q = PooledDynamicQueue::new(CAP);
            len_grow(q);
        }

        #[test]
        fn grow_storm_impl() {
            let q = PooledDynamicQueue::new(2);
            grow_storm(q);
        }

        #[test]
        fn oscillation_grow_impl() {
            let q = PooledDynamicQueue::new(2);
            oscillation_grow(q);
        }

        #[test]
        fn suppl_methods_chaos_impl() {
            let q = PooledDynamicQueue::new(2);
            suppl_methods_chaos(q);
        }

        #[test]
        fn pooled_stores_any_impl() {
            let q = PooledDynamicQueue::new(1);
            pooled_stores_any(q);
        }
    }
}
