use crate::{
    MPMCQueue,
    sync::{
        atomic::{AtomicUsize, Ordering},
        thread,
    },
};

// TODO integrate this in test_library.rs

pub(crate) fn spsc<Q>(q: Q)
where
    Q: MPMCQueue<Item = u32> + Sync,
{
    const COUNT: usize = 50;

    thread::scope(|scope| {
        scope.spawn(|| {
            for i in 0..COUNT {
                loop {
                    if let Some(x) = q.pop() {
                        assert_eq!(x, i as u32);
                        break;
                    }
                }
            }
            assert!(q.pop().is_none());
        });

        scope.spawn(|| {
            for i in 0..COUNT {
                while q.push(i as u32).is_err() {}
            }
        });
    })
}

pub(crate) fn mpsc<Q>(q: Q)
where
    Q: MPMCQueue<Item = u32> + Sync,
{
    const COUNT: usize = 20;
    const THREADS: usize = 4;

    let v = (0..COUNT).map(|_| AtomicUsize::new(0)).collect::<Vec<_>>();

    thread::scope(|scope| {
        for _ in 0..THREADS {
            scope.spawn(|| {
                for i in 0..COUNT {
                    while q.push(i as u32).is_err() {}
                }
            });
        }
        for _ in 0..THREADS {
            for _ in 0..COUNT {
                let n = loop {
                    if let Some(x) = q.pop() {
                        break x;
                    }
                };
                v[n as usize].fetch_add(1, Ordering::SeqCst);
            }
        }
    });

    for c in v {
        assert_eq!(c.load(Ordering::SeqCst), THREADS);
    }
}

pub(crate) fn mpmc<Q>(q: Q)
where
    Q: MPMCQueue<Item = u32> + Sync,
{
    const COUNT: usize = 20;
    const THREADS: usize = 4;
    let v = (0..COUNT).map(|_| AtomicUsize::new(0)).collect::<Vec<_>>();

    thread::scope(|scope| {
        for _ in 0..THREADS {
            scope.spawn(|| {
                for _ in 0..COUNT {
                    let n = loop {
                        if let Some(x) = q.pop() {
                            break x;
                        }
                    };
                    v[n as usize].fetch_add(1, Ordering::SeqCst);
                }
            });
        }
        for _ in 0..THREADS {
            scope.spawn(|| {
                for i in 0..COUNT {
                    while q.push(i as u32).is_err() {}
                }
            });
        }
    });

    for c in v {
        assert_eq!(c.load(Ordering::SeqCst), THREADS);
    }
}

pub(crate) fn mpmc_ring_buffer<Q>(q: Q)
where
    Q: MPMCQueue<Item = u32> + Sync,
{
    const COUNT: usize = 20;
    const THREADS: usize = 2;

    let t = AtomicUsize::new(THREADS);
    let v = (0..COUNT).map(|_| AtomicUsize::new(0)).collect::<Vec<_>>();

    thread::scope(|scope| {
        for _ in 0..THREADS {
            scope.spawn(|| {
                loop {
                    match t.load(Ordering::SeqCst) {
                        0 => {
                            while let Some(n) = q.pop() {
                                v[n as usize].fetch_add(1, Ordering::SeqCst);
                            }
                            break;
                        }

                        _ => {
                            while let Some(n) = q.pop() {
                                v[n as usize].fetch_add(1, Ordering::SeqCst);
                            }
                            crate::utils::Backoff::new().backoff();
                        }
                    }
                }
            });
        }

        for _ in 0..THREADS {
            scope.spawn(|| {
                for i in 0..COUNT {
                    q.force_push_and_do(i as u32, |n| {
                        v[n as usize].fetch_add(1, Ordering::SeqCst);
                    });
                }

                t.fetch_sub(1, Ordering::SeqCst);
            });
        }
    });

    for c in v {
        assert_eq!(c.load(Ordering::SeqCst), THREADS);
    }
}

pub(crate) fn linearizable<Q>(q: Q)
where
    Q: MPMCQueue<Item = u32> + Sync,
{
    const COUNT: usize = 50;
    const THREADS: usize = 4;

    thread::scope(|scope| {
        for _ in 0..THREADS / 2 {
            scope.spawn(|| {
                for _ in 0..COUNT {
                    while q.push(42).is_err() {}
                    q.pop().unwrap();
                }
            });

            scope.spawn(|| {
                for _ in 0..COUNT {
                    if q.force_push(42).is_none() {
                        q.pop().unwrap();
                    }
                }
            });
        }
    })
}

#[cfg(feature = "dynamic")]
pub(crate) use growth::*;

#[cfg(feature = "dynamic")]
mod growth {
    use shuttle::sync::Arc;

    use super::*;
    use crate::Growable;

    pub(crate) fn mpsc_grow<Q>(q: Q)
    where
        Q: Growable + MPMCQueue<Item = u32> + Sync,
    {
        const COUNT: usize = 20;
        const THREADS: usize = 4;
        const GROW_STEP: usize = 10;

        let v = (0..COUNT).map(|_| AtomicUsize::new(0)).collect::<Vec<_>>();

        thread::scope(|scope| {
            for _ in 0..THREADS {
                scope.spawn(|| {
                    for i in 0..COUNT {
                        loop {
                            if q.push(i as u32).is_ok() {
                                break;
                            }
                            _ = q.grow_by(GROW_STEP);
                            crate::utils::Backoff::new().backoff();
                        }
                    }
                });
            }

            for _ in 0..THREADS {
                for _ in 0..COUNT {
                    let n = loop {
                        if let Some(x) = q.pop() {
                            break x;
                        }
                    };
                    v[n as usize].fetch_add(1, Ordering::SeqCst);
                }
            }
        });

        for c in v {
            assert_eq!(c.load(Ordering::SeqCst), THREADS);
        }
    }

    pub(crate) fn mpmc_grow<Q>(q: Q)
    where
        Q: Growable + MPMCQueue<Item = u32> + Sync,
    {
        const COUNT: usize = 30;
        const RESIZERS: usize = 2;
        const THREADS: usize = 2;

        let v = (0..COUNT).map(|_| AtomicUsize::new(0)).collect::<Vec<_>>();

        thread::scope(|scope| {
            for _ in 0..THREADS {
                scope.spawn(|| {
                    for i in 0..COUNT {
                        while q.push(i as u32).is_err() {
                            _ = q.grow_by(1);
                            crate::utils::Backoff::new().backoff();
                        }
                    }
                });
            }

            for _ in 0..THREADS {
                scope.spawn(|| {
                    for _ in 0..COUNT {
                        let n = loop {
                            if let Some(x) = q.pop() {
                                break x;
                            }
                        };
                        v[n as usize].fetch_add(1, Ordering::SeqCst);
                    }
                });
            }

            for _ in 0..RESIZERS {
                scope.spawn(|| {
                    let mut backoff = crate::utils::Backoff::new();
                    for _ in 0..5 {
                        q.grow_by(2);
                        backoff.backoff();
                    }
                });
            }
        });

        for c in v {
            assert_eq!(c.load(Ordering::SeqCst), THREADS);
        }
    }

    pub(crate) fn grow_storm<Q>(q: Q)
    where
        Q: Growable + MPMCQueue<Item = u32> + Sync,
    {
        const THREADS: usize = 4;
        const ITERS: usize = 20;

        let tracking_vector = (0..ITERS).map(|_| AtomicUsize::new(0)).collect::<Vec<_>>();

        thread::scope(|scope| {
            for _ in 0..THREADS {
                scope.spawn(|| {
                    for i in 0..ITERS {
                        if i % 5 == 0 {
                            let _ = q.grow_by(2);
                        }

                        let mut backoff = crate::utils::Backoff::new();
                        loop {
                            if q.push(i as u32).is_ok() {
                                break;
                            }
                            backoff.backoff();
                        }
                    }
                });

                scope.spawn(|| {
                    for i in 0..ITERS {
                        if i % 3 == 0 {
                            let _ = q.grow_by(1);
                        }

                        let mut backoff = crate::utils::Backoff::new();
                        let item = loop {
                            if let Some(x) = q.pop() {
                                break x;
                            }
                            backoff.backoff();
                        };
                        tracking_vector[item as usize].fetch_add(1, Ordering::SeqCst);
                    }
                });
            }
        });

        for count in tracking_vector.into_iter() {
            assert_eq!(count.load(Ordering::SeqCst), THREADS);
        }
    }

    pub(crate) fn oscillation_grow<Q>(q: Q)
    where
        Q: Growable + MPMCQueue<Item = u32> + Sync,
    {
        const ITER: usize = 10;

        let total_popped = Arc::new(AtomicUsize::new(0));
        let total_pushed = Arc::new(AtomicUsize::new(0));

        thread::scope(|scope| {
            scope.spawn(|| {
                for _ in 0..10 {
                    _ = q.grow_by(0);
                    thread::yield_now();
                }
            });

            scope.spawn(|| {
                let mut backoff = crate::utils::Backoff::new();
                for _ in 1..ITER {
                    let mut pushes = 0;
                    let mut backoff_inner = crate::utils::Backoff::new();

                    let cap = q.capacity();

                    while pushes < cap {
                        if q.push(42).is_ok() {
                            pushes = total_pushed.fetch_add(1, Ordering::SeqCst) + 1;
                        }
                        backoff_inner.backoff();
                    }

                    backoff.backoff();

                    while q.pop().is_some() {
                        total_popped.fetch_add(1, Ordering::SeqCst);
                    }
                }
            });
        });

        assert!(q.is_empty());
        assert_eq!(q.len(), 0);
        assert_eq!(
            total_popped.load(Ordering::SeqCst),
            total_pushed.load(Ordering::SeqCst)
        );
    }

    pub(crate) fn len_grow<Q>(q: Q)
    where
        Q: MPMCQueue<Item = u32> + Sync + Growable,
    {
        const COUNT: usize = 30;
        const CAP: usize = 40;

        thread::scope(|scope| {
            scope.spawn(|| {
                for i in 0..COUNT {
                    loop {
                        if let Some(x) = q.pop() {
                            assert_eq!(x, i as u32);
                            break;
                        }
                    }
                    let _len = q.len();
                }
            });

            scope.spawn(|| {
                for i in 0..COUNT {
                    while q.push(i as u32).is_err() {}
                    let _len = q.len();
                }
            });

            scope.spawn(|| {
                const GROW_ITERS: usize = 3;

                let mut backoff = crate::utils::Backoff::new();
                for _ in 0..GROW_ITERS {
                    let _ = q.grow_by(CAP / 2);
                    backoff.backoff();
                }
            });
        });

        assert_eq!(q.len(), 0);
    }

    pub(crate) fn suppl_methods_chaos<Q>(q: Q)
    where
        Q: Growable + MPMCQueue<Item = u32> + Sync,
    {
        const ITERS: usize = 30;
        const GROW_CYCLES: usize = 30;
        const GROW_STEP: usize = 10;

        let initial_cap = q.capacity();

        let total_grows = Arc::new(AtomicUsize::new(0));

        thread::scope(|scope| {
            scope.spawn(|| {
                let mut last_cap = initial_cap;
                for _ in 0..ITERS {
                    let current_cap = q.capacity();

                    assert!(
                        current_cap >= last_cap,
                        "Monotonicity broken: Capacity shrank from {} to {}!",
                        last_cap,
                        current_cap
                    );
                    last_cap = current_cap;

                    let _ = q.is_full();
                }
            });

            scope.spawn(|| {
                for _ in 0..ITERS {
                    let _len = q.len();
                    let _empty = q.is_empty();
                }
            });

            scope.spawn(|| {
                for i in 0..ITERS {
                    let _ = q.push(i as u32);
                    let _ = q.pop();
                }
            });

            scope.spawn(|| {
                let mut backoff = crate::utils::Backoff::new();

                for _ in 0..GROW_CYCLES {
                    if q.grow_by(GROW_STEP) {
                        total_grows.fetch_add(1, Ordering::SeqCst);
                    }
                    backoff.backoff();
                }
            });
        });

        let final_cap = q.capacity();
        let expected_min_cap = initial_cap + (total_grows.load(Ordering::SeqCst) * GROW_STEP);
        assert!(
            final_cap >= expected_min_cap,
            "Structural integrity failed: Expected capacity >= {}, but got {}",
            expected_min_cap,
            final_cap
        );
    }
}

cfg_atomic_tagged64! {
    mod taggedptr64 {
        use crate::{Queue, core::slots::Tagged64};

        use super::*;

        #[test]
        fn spsc_impl() {
            shuttle::check_random(
                || {
                    let q = Queue::with_slot::<Tagged64>(3);
                    spsc(q);
                },
                100,
            );
        }

        #[test]
        fn mpmc_impl() {
            shuttle::check_random(
                || {
                    let q = Queue::with_slot::<Tagged64>(3);
                    mpmc(q);
                },
                100,
            );
        }

        #[test]
        fn mpmc_ring_buffer_impl() {
            shuttle::check_random(
                || {
                    let q = Queue::with_slot::<Tagged64>(3);
                    mpmc_ring_buffer(q);
                },
                100,
            );
        }

        #[test]
        fn mpsc_impl() {
            shuttle::check_random(
                || {
                    let q = Queue::with_slot::<Tagged64>(3);
                    mpsc(q);
                },
                100,
            );
        }

        #[test]
        fn linearizable_impl() {
            shuttle::check_random(
                || {
                    let q = Queue::with_slot::<Tagged64>(4);
                    linearizable(q);
                },
                100,
            );
        }
    }
}

cfg_atomic_tagged128! {
    mod taggedptr128 {
        use crate::{Queue, core::slots::Tagged128};

        use super::*;

        #[test]
        fn spsc_impl() {
            shuttle::check_random(
                || {
                    let q = Queue::with_slot::<Tagged128>(3);
                    spsc(q);
                },
                100,
            );
        }

        #[test]
        fn mpmc_impl() {
            shuttle::check_random(
                || {
                    let q = Queue::with_slot::<TaggedPtr64>(3);
                    mpmc(q);
                },
                100,
            );
        }

        #[test]
        fn mpmc_ring_buffer_impl() {
            shuttle::check_random(
                || {
                    let q = Queue::with_slot::<Tagged128>(3);
                    mpmc_ring_buffer(q);
                },
                100,
            );
        }

        #[test]
        fn mpsc_impl() {
            shuttle::check_random(
                || {
                    let q = Queue::with_slot::<Tagged128>(3);
                    mpsc(q);
                },
                100,
            );
        }

        #[test]
        fn linearizable_impl() {
            shuttle::check_random(
                || {
                    let q = Queue::with_slot::<Tagged128>(4);
                    linearizable(q);
                },
                100,
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
        shuttle::check_random(
            || {
                let q = PooledQueue::new(3);
                spsc(q);
            },
            100,
        );
    }

    #[test]
    fn mpmc_impl() {
        shuttle::check_random(
            || {
                let q = PooledQueue::new(3);
                mpmc(q);
            },
            100,
        );
    }

    #[test]
    fn mpmc_ring_buffer_impl() {
        shuttle::check_random(
            || {
                let q = PooledQueue::new(3);
                mpmc_ring_buffer(q);
            },
            100,
        );
    }

    #[test]
    fn mpsc_impl() {
        shuttle::check_random(
            || {
                let q = PooledQueue::new(3);
                mpsc(q);
            },
            100,
        );
    }

    #[test]
    fn linearizable_impl() {
        shuttle::check_random(
            || {
                let q = PooledQueue::new(4);
                linearizable(q);
            },
            100,
        );
    }
}

#[cfg(feature = "dynamic")]
mod growable {
    use super::*;
    use crate::DynamicQueue;

    #[test]
    fn spsc_impl() {
        shuttle::check_random(
            || {
                let q = DynamicQueue::new(3);
                spsc(q);
            },
            100,
        );
    }

    #[test]
    fn mpmc_impl() {
        shuttle::check_random(
            || {
                let q = DynamicQueue::new(3);
                mpmc(q);
            },
            100,
        );
    }

    #[test]
    fn mpmc_ring_buffer_impl() {
        shuttle::check_random(
            || {
                let q = DynamicQueue::new(3);
                mpmc_ring_buffer(q);
            },
            100,
        );
    }

    #[test]
    fn mpsc_impl() {
        shuttle::check_random(
            || {
                let q = DynamicQueue::new(3);
                mpsc(q);
            },
            100,
        );
    }

    #[test]
    fn linearizable_impl() {
        shuttle::check_random(
            || {
                let q = DynamicQueue::new(4);
                linearizable(q);
            },
            100,
        );
    }

    #[test]
    fn mpsc_grow_impl() {
        shuttle::check_random(
            || {
                let q = DynamicQueue::new(4);
                mpsc_grow(q);
            },
            100,
        );
    }

    #[test]
    fn mpmc_grow_impl() {
        shuttle::check_random(
            || {
                let q = DynamicQueue::new(4);
                mpmc_grow(q);
            },
            100,
        );
    }

    #[test]
    fn len_grow_impl() {
        const CAP: usize = 40;
        shuttle::check_random(
            || {
                let q = DynamicQueue::new(CAP);
                len_grow(q);
            },
            100,
        );
    }

    #[test]
    fn grow_storm_impl() {
        shuttle::check_random(
            || {
                let q = DynamicQueue::new(4);
                grow_storm(q);
            },
            100,
        );
    }

    #[test]
    fn oscillation_grow_impl() {
        shuttle::check_random(
            || {
                let q = DynamicQueue::new(4);
                oscillation_grow(q);
            },
            100,
        );
    }

    #[test]
    fn suppl_methods_chaos_impl() {
        shuttle::check_random(
            || {
                let q = DynamicQueue::new(4);
                suppl_methods_chaos(q);
            },
            100,
        );
    }

    #[cfg(feature = "pool")]
    mod pool {
        use super::*;
        use crate::PooledDynamicQueue;

        #[test]
        fn spsc_impl() {
            shuttle::check_random(
                || {
                    let q = PooledDynamicQueue::new(3);
                    spsc(q);
                },
                100,
            );
        }

        #[test]
        fn mpmc_impl() {
            shuttle::check_random(
                || {
                    let q = PooledDynamicQueue::new(3);
                    mpmc(q);
                },
                100,
            );
        }

        #[test]
        fn mpmc_ring_buffer_impl() {
            shuttle::check_random(
                || {
                    let q = PooledDynamicQueue::new(3);
                    mpmc_ring_buffer(q);
                },
                100,
            );
        }

        #[test]
        fn mpsc_impl() {
            shuttle::check_random(
                || {
                    let q = PooledDynamicQueue::new(3);
                    mpsc(q);
                },
                100,
            );
        }

        #[test]
        fn linearizable_impl() {
            shuttle::check_random(
                || {
                    let q = PooledDynamicQueue::new(4);
                    linearizable(q);
                },
                100,
            );
        }

        #[test]
        fn mpsc_grow_impl() {
            shuttle::check_random(
                || {
                    let q = PooledDynamicQueue::new(4);
                    mpsc_grow(q);
                },
                100,
            );
        }

        #[test]
        fn mpmc_grow_impl() {
            shuttle::check_random(
                || {
                    let q = PooledDynamicQueue::new(4);
                    mpmc_grow(q);
                },
                100,
            );
        }

        #[test]
        fn len_grow_impl() {
            const CAP: usize = 40;
            shuttle::check_random(
                || {
                    let q = PooledDynamicQueue::new(CAP);
                    len_grow(q);
                },
                100,
            );
        }

        #[test]
        fn grow_storm_impl() {
            shuttle::check_random(
                || {
                    let q = PooledDynamicQueue::new(4);
                    grow_storm(q);
                },
                100,
            );
        }

        #[test]
        fn oscillation_grow_impl() {
            shuttle::check_random(
                || {
                    let q = PooledDynamicQueue::new(4);
                    oscillation_grow(q);
                },
                100,
            );
        }

        #[test]
        fn suppl_methods_chaos_impl() {
            shuttle::check_random(
                || {
                    let q = PooledDynamicQueue::new(4);
                    suppl_methods_chaos(q);
                },
                100,
            );
        }
    }
}
