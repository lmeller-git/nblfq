use crate::tests::test_library::{linearizable, mpmc, mpmc_ring_buffer, mpsc, spsc};

cfg_atomic_tagged64! {
    mod taggedptr64 {
        use super::*;
        use crate::{Queue, core::slots::Tagged64};

        #[test]
        fn spsc_impl() {
            shuttle::check_pct(
                || {
                    let q = Queue::with_slot::<Tagged64>(3);
                    spsc(q);
                },
                100,
                4,
            );
        }

        #[test]
        fn mpmc_impl() {
            shuttle::check_pct(
                || {
                    let q = Queue::with_slot::<Tagged64>(3);
                    mpmc(q);
                },
                100,
                4,
            );
        }

        #[test]
        fn mpmc_ring_buffer_impl() {
            shuttle::check_pct(
                || {
                    let q = Queue::with_slot::<Tagged64>(3);
                    mpmc_ring_buffer(q);
                },
                100,
                4,
            );
        }

        #[test]
        fn mpsc_impl() {
            shuttle::check_pct(
                || {
                    let q = Queue::with_slot::<Tagged64>(3);
                    mpsc(q);
                },
                100,
                4,
            );
        }

        #[test]
        fn linearizable_impl() {
            shuttle::check_pct(
                || {
                    let q = Queue::with_slot::<Tagged64>(4);
                    linearizable(q);
                },
                100,
                4,
            );
        }
    }
}

cfg_atomic_tagged128! {
    mod taggedptr128 {
        use super::*;
        use crate::{Queue, core::slots::Tagged128};

        #[test]
        fn spsc_impl() {
            shuttle::check_pct(
                || {
                    let q = Queue::with_slot::<Tagged128>(3);
                    spsc(q);
                },
                100,
                4,
            );
        }

        #[test]
        fn mpmc_impl() {
            shuttle::check_pct(
                || {
                    let q = Queue::with_slot::<TaggedPtr64>(3);
                    mpmc(q);
                },
                100,
                4,
            );
        }

        #[test]
        fn mpmc_ring_buffer_impl() {
            shuttle::check_pct(
                || {
                    let q = Queue::with_slot::<Tagged128>(3);
                    mpmc_ring_buffer(q);
                },
                100,
                4,
            );
        }

        #[test]
        fn mpsc_impl() {
            shuttle::check_pct(
                || {
                    let q = Queue::with_slot::<Tagged128>(3);
                    mpsc(q);
                },
                100,
                4,
            );
        }

        #[test]
        fn linearizable_impl() {
            shuttle::check_pct(
                || {
                    let q = Queue::with_slot::<Tagged128>(4);
                    linearizable(q);
                },
                100,
                4,
            );
        }
    }
}

#[cfg(feature = "pool")]
mod pool {
    use super::*;
    use crate::PooledQueue;

    #[test]
    fn spsc_impl() {
        shuttle::check_pct(
            || {
                let q = PooledQueue::new(3);
                spsc(q);
            },
            100,
            4,
        );
    }

    #[test]
    fn mpmc_impl() {
        shuttle::check_pct(
            || {
                let q = PooledQueue::new(3);
                mpmc(q);
            },
            100,
            4,
        );
    }

    #[test]
    fn mpmc_ring_buffer_impl() {
        shuttle::check_pct(
            || {
                let q = PooledQueue::new(3);
                mpmc_ring_buffer(q);
            },
            100,
            4,
        );
    }

    #[test]
    fn mpsc_impl() {
        shuttle::check_pct(
            || {
                let q = PooledQueue::new(3);
                mpsc(q);
            },
            100,
            4,
        );
    }

    #[test]
    fn linearizable_impl() {
        shuttle::check_pct(
            || {
                let q = PooledQueue::new(4);
                linearizable(q);
            },
            100,
            4,
        );
    }
}

#[cfg(feature = "dynamic")]
mod growable {
    use super::*;
    use crate::{
        DynamicQueue,
        tests::test_library::{
            grow_storm,
            len_grow,
            linearizable_during_resize,
            mpmc_resize,
            mpsc_grow,
            oscillation_grow,
            push_pop_resize,
            suppl_methods_chaos,
        },
    };

    #[test]
    fn spsc_impl() {
        shuttle::check_pct(
            || {
                let q = DynamicQueue::new(3);
                spsc(q);
            },
            100,
            4,
        );
    }

    #[test]
    fn mpmc_impl() {
        shuttle::check_pct(
            || {
                let q = DynamicQueue::new(3);
                mpmc(q);
            },
            100,
            4,
        );
    }

    #[test]
    fn mpmc_ring_buffer_impl() {
        shuttle::check_pct(
            || {
                let q = DynamicQueue::new(3);
                mpmc_ring_buffer(q);
            },
            100,
            4,
        );
    }

    #[test]
    fn mpsc_impl() {
        shuttle::check_pct(
            || {
                let q = DynamicQueue::new(3);
                mpsc(q);
            },
            100,
            4,
        );
    }

    #[test]
    fn linearizable_impl() {
        shuttle::check_pct(
            || {
                let q = DynamicQueue::new(4);
                linearizable(q);
            },
            100,
            4,
        );
    }

    #[test]
    fn mpsc_grow_impl() {
        shuttle::check_pct(
            || {
                let q = DynamicQueue::new(4);
                mpsc_grow(q);
            },
            100,
            4,
        );
    }

    #[test]
    fn mpmc_resize_impl() {
        shuttle::check_pct(
            || {
                let q = DynamicQueue::new(4);
                mpmc_resize(q);
            },
            100,
            4,
        );
    }

    #[test]
    fn len_grow_impl() {
        const CAP: usize = 40;
        shuttle::check_pct(
            || {
                let q = DynamicQueue::new(CAP);
                len_grow(q);
            },
            100,
            4,
        );
    }

    #[test]
    fn grow_storm_impl() {
        shuttle::check_pct(
            || {
                let q = DynamicQueue::new(4);
                grow_storm(q);
            },
            100,
            4,
        );
    }

    #[test]
    fn oscillation_grow_impl() {
        shuttle::check_pct(
            || {
                let q = DynamicQueue::new(4);
                oscillation_grow(q);
            },
            100,
            4,
        );
    }

    #[test]
    fn suppl_methods_chaos_impl() {
        shuttle::check_pct(
            || {
                let q = DynamicQueue::new(4);
                suppl_methods_chaos(q);
            },
            100,
            4,
        );
    }

    #[test]
    fn linearizable_during_resize_impl() {
        shuttle::check_pct(
            || {
                let q = DynamicQueue::new(4);
                linearizable_during_resize(q);
            },
            100,
            4,
        );
    }

    #[test]
    fn push_pop_resize_impl() {
        shuttle::check_pct(
            || {
                let q = DynamicQueue::new(4);
                push_pop_resize(q);
            },
            100,
            4,
        )
    }

    #[cfg(feature = "pool")]
    mod pool {
        use super::*;
        use crate::PooledDynamicQueue;

        #[test]
        fn spsc_impl() {
            shuttle::check_pct(
                || {
                    let q = PooledDynamicQueue::new(3);
                    spsc(q);
                },
                100,
                4,
            );
        }

        #[test]
        fn mpmc_impl() {
            shuttle::check_pct(
                || {
                    let q = PooledDynamicQueue::new(3);
                    mpmc(q);
                },
                100,
                4,
            );
        }

        #[test]
        fn mpmc_ring_buffer_impl() {
            shuttle::check_pct(
                || {
                    let q = PooledDynamicQueue::new(3);
                    mpmc_ring_buffer(q);
                },
                100,
                4,
            );
        }

        #[test]
        fn mpsc_impl() {
            shuttle::check_pct(
                || {
                    let q = PooledDynamicQueue::new(3);
                    mpsc(q);
                },
                100,
                4,
            );
        }

        #[test]
        fn linearizable_impl() {
            shuttle::check_pct(
                || {
                    let q = PooledDynamicQueue::new(4);
                    linearizable(q);
                },
                100,
                4,
            );
        }

        #[test]
        fn mpsc_grow_impl() {
            shuttle::check_pct(
                || {
                    let q = PooledDynamicQueue::new(4);
                    mpsc_grow(q);
                },
                100,
                4,
            );
        }

        #[test]
        fn mpmc_resize_impl() {
            shuttle::check_pct(
                || {
                    let q = PooledDynamicQueue::new(4);
                    mpmc_resize(q);
                },
                100,
                4,
            );
        }

        #[test]
        fn len_grow_impl() {
            const CAP: usize = 40;
            shuttle::check_pct(
                || {
                    let q = PooledDynamicQueue::new(CAP);
                    len_grow(q);
                },
                100,
                4,
            );
        }

        #[test]
        fn grow_storm_impl() {
            shuttle::check_pct(
                || {
                    let q = PooledDynamicQueue::new(4);
                    grow_storm(q);
                },
                100,
                4,
            );
        }

        #[test]
        fn oscillation_grow_impl() {
            shuttle::check_pct(
                || {
                    let q = PooledDynamicQueue::new(4);
                    oscillation_grow(q);
                },
                100,
                4,
            );
        }

        #[test]
        fn suppl_methods_chaos_impl() {
            shuttle::check_pct(
                || {
                    let q = PooledDynamicQueue::new(4);
                    suppl_methods_chaos(q);
                },
                100,
                4,
            );
        }

        #[test]
        fn linearizable_during_resize_impl() {
            shuttle::check_pct(
                || {
                    let q = PooledDynamicQueue::new(4);
                    linearizable_during_resize(q);
                },
                100,
                4,
            );
        }

        #[test]
        fn push_pop_resize_impl() {
            shuttle::check_pct(
                || {
                    let q = PooledDynamicQueue::new(4);
                    push_pop_resize(q);
                },
                100,
                4,
            )
        }
    }
}
