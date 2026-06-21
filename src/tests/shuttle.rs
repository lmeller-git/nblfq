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
                        while q.push(i as u32).is_err() {}
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
                    for _ in 0..5 {
                        q.grow_by(2);
                        crate::utils::Backoff::new().backoff();
                    }
                });
            }
        });

        for c in v {
            assert_eq!(c.load(Ordering::SeqCst), THREADS);
        }
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
                    let len = q.len();
                    assert!(len <= q.capacity());
                }
            });

            scope.spawn(|| {
                for i in 0..COUNT {
                    while q.push(i as u32).is_err() {}
                    let len = q.len();
                    assert!(len <= q.capacity());
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
}
