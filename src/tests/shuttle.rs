use crate::{
    MPMCQueue, cfg_taggedptr64,
    sync::{
        atomic::{AtomicUsize, Ordering},
        thread,
    },
};

pub(crate) fn mpmc<Q>(q: Q)
where
    Q: MPMCQueue<Item = Box<usize>> + Sync,
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
                    v[*n].fetch_add(1, Ordering::SeqCst);
                }
            });
        }
        for _ in 0..THREADS {
            scope.spawn(|| {
                for i in 0..COUNT {
                    while q.push(Box::new(i)).is_err() {}
                }
            });
        }
    });

    for c in v {
        assert_eq!(c.load(Ordering::SeqCst), THREADS);
    }
}
pub(crate) fn mpsc<Q>(q: Q)
where
    Q: MPMCQueue<Item = Box<usize>> + Sync,
{
    const COUNT: usize = 20;
    const THREADS: usize = 4;

    let v = (0..COUNT).map(|_| AtomicUsize::new(0)).collect::<Vec<_>>();

    thread::scope(|scope| {
        for _ in 0..THREADS {
            scope.spawn(|| {
                for i in 0..COUNT {
                    while q.push(Box::new(i)).is_err() {}
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
                v[*n].fetch_add(1, Ordering::SeqCst);
            }
        }
    });

    for c in v {
        assert_eq!(c.load(Ordering::SeqCst), THREADS);
    }
}

pub(crate) fn mpmc_ring_buffer<Q>(q: Q)
where
    Q: MPMCQueue<Item = Box<usize>> + Sync,
{
    const COUNT: usize = 10;
    const THREADS: usize = 2;

    let t = AtomicUsize::new(THREADS);
    let v = (0..COUNT).map(|_| AtomicUsize::new(0)).collect::<Vec<_>>();

    thread::scope(|scope| {
        for _ in 0..THREADS {
            scope.spawn(|| {
                loop {
                    match t.load(Ordering::SeqCst) {
                        0 if q.is_empty() => break,

                        _ => {
                            while let Some(n) = q.pop() {
                                v[*n].fetch_add(1, Ordering::SeqCst);
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
                    if let Some(n) = q.force_push(Box::new(i)) {
                        v[*n].fetch_add(1, Ordering::SeqCst);
                    }
                }

                t.fetch_sub(1, Ordering::SeqCst);
            });
        }
    });

    for c in v {
        assert!(c.load(Ordering::SeqCst) <= THREADS);
    }
}

pub(crate) fn linearizable<Q>(q: Q)
where
    Q: MPMCQueue<Item = &'static i32> + Sync,
{
    const COUNT: usize = 50;
    const THREADS: usize = 4;

    thread::scope(|scope| {
        for _ in 0..THREADS / 2 {
            scope.spawn(|| {
                for _ in 0..COUNT {
                    while q.push(&0).is_err() {}
                    q.pop().unwrap();
                }
            });

            scope.spawn(|| {
                for _ in 0..COUNT {
                    if q.force_push(&0).is_none() {
                        q.pop().unwrap();
                    }
                }
            });
        }
    })
}

cfg_taggedptr64! {
    mod taggedptr64 {
        use crate::{Queue, core::slots::TaggedPtr64};

        use super::*;

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
                    let q = Queue::with_slot::<TaggedPtr64>(3);
                    mpmc_ring_buffer(q);
                },
                50,
            );
        }

        #[test]
        fn mpsc_impl() {
            shuttle::check_random(
                || {
                    let q = Queue::with_slot::<TaggedPtr64>(3);
                    mpsc(q);
                },
                100,
            );
        }

        #[test]
        fn linearizable_impl() {
            shuttle::check_random(
                || {
                    let q = Queue::with_slot::<TaggedPtr64>(4);
                    linearizable(q);
                },
                100,
            );
        }
    }
}

#[cfg(feature = "pool")]
mod pool {
    use crate::PooledQueue;

    use super::*;

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
            50,
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
