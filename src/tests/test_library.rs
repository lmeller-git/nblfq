//! Testing for nblfqueue
//!
//! Tests adapted from crossbeam-queue's test suite.
//! https://github.com/crossbeam-rs/crossbeam/tree/master/crossbeam-queue

use core::sync::atomic::{AtomicUsize, Ordering};
use std::{boxed::Box, thread::scope, vec::Vec};

use crate::{MPMCQueue, queue::ForcePushQueue};

pub(crate) fn smoke<Q>(q: Q)
where
    Q: MPMCQueue<Item = &'static usize>,
{
    q.push(&7).unwrap();
    assert_eq!(q.pop(), Some(&7));
    q.push(&8).unwrap();
    assert_eq!(q.pop(), Some(&8));
    assert!(q.pop().is_none());
}

pub(crate) fn smoke_long<Q>(q: Q)
where
    Q: MPMCQueue<Item = &'static i32>,
{
    q.push(&7).unwrap();
    assert_eq!(q.pop(), Some(&7));
    q.push(&8).unwrap();
    q.push(&9).unwrap();
    assert_eq!(q.pop(), Some(&8));
    assert_eq!(q.pop(), Some(&9));
    assert!(q.pop().is_none());
}

pub(crate) fn len_empty_full<Q>(q: Q)
where
    Q: MPMCQueue<Item = &'static ()>,
{
    assert_eq!(q.len(), 0);
    assert!(q.is_empty());
    assert!(!q.is_full());

    q.push(&()).unwrap();

    assert_eq!(q.len(), 1);
    assert!(!q.is_empty());
    assert!(!q.is_full());

    q.push(&()).unwrap();

    assert_eq!(q.len(), 2);
    assert!(!q.is_empty());
    assert!(q.is_full());

    q.pop().unwrap();

    assert_eq!(q.len(), 1);
    assert!(!q.is_empty());
    assert!(!q.is_full());
}

pub(crate) fn len<Q>(q: Q)
where
    Q: MPMCQueue<Item = Box<usize>> + Sync,
{
    #[cfg(miri)]
    const COUNT: usize = 30;
    #[cfg(not(miri))]
    const COUNT: usize = 25_000;
    #[cfg(miri)]
    const CAP: usize = 40;
    #[cfg(not(miri))]
    const CAP: usize = 1000;
    const ITERS: usize = CAP / 20;

    assert_eq!(q.len(), 0);
    assert_eq!(q.capacity(), CAP);

    for _ in 0..CAP / 10 {
        for i in 0..ITERS {
            let i = Box::new(i);
            q.push(i.clone()).unwrap();
            assert_eq!(q.len(), *i + 1);
        }

        for i in 0..ITERS {
            q.pop().unwrap();
            assert_eq!(q.len(), ITERS - i - 1);
        }
    }
    assert_eq!(q.len(), 0);

    for i in 0..CAP {
        let i = Box::new(i);
        q.push(i.clone()).unwrap();
        assert_eq!(q.len(), *i + 1);
    }

    assert!(q.is_full());
    assert_eq!(q.len(), CAP);

    for _ in 0..CAP {
        q.pop().unwrap();
    }
    assert_eq!(q.len(), 0);

    scope(|scope| {
        scope.spawn(|| {
            for i in 0..COUNT {
                loop {
                    if let Some(x) = q.pop() {
                        assert_eq!(*x, i);
                        break;
                    }
                }
                let len = q.len();
                assert!(len <= CAP);
            }
        });

        scope.spawn(|| {
            for i in 0..COUNT {
                let i = Box::new(i);
                while q.push(i.clone()).is_err() {}
                let len = q.len();
                assert!(len <= CAP);
            }
        });
    });
    assert_eq!(q.len(), 0);
}

pub(crate) fn spsc<Q>(q: Q)
where
    Q: MPMCQueue<Item = Box<usize>> + Sync,
{
    #[cfg(miri)]
    const COUNT: usize = 50;
    #[cfg(not(miri))]
    const COUNT: usize = 100_000;

    scope(|scope| {
        scope.spawn(|| {
            for i in 0..COUNT {
                loop {
                    if let Some(x) = q.pop() {
                        assert_eq!(*x, i);
                        break;
                    }
                }
            }
            assert!(q.pop().is_none());
        });

        scope.spawn(|| {
            for i in 0..COUNT {
                while q.push(Box::new(i)).is_err() {}
            }
        });
    })
}

pub(crate) fn mpsc<Q>(q: Q)
where
    Q: MPMCQueue<Item = Box<usize>> + Sync,
{
    #[cfg(miri)]
    const COUNT: usize = 10;
    #[cfg(not(miri))]
    const COUNT: usize = 10_000;
    const THREADS: usize = 4;

    let v = (0..COUNT).map(|_| AtomicUsize::new(0)).collect::<Vec<_>>();

    scope(|scope| {
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

pub(crate) fn mpmc<Q>(q: Q)
where
    Q: MPMCQueue<Item = Box<usize>> + Sync,
{
    #[cfg(miri)]
    const COUNT: usize = 50;
    #[cfg(not(miri))]
    const COUNT: usize = 25_000;
    const THREADS: usize = 4;

    let v = (0..COUNT).map(|_| AtomicUsize::new(0)).collect::<Vec<_>>();

    scope(|scope| {
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

pub(crate) fn mpmc_ring_buffer<Q>(q: Q)
where
    Q: MPMCQueue<Item = Box<usize>> + ForcePushQueue + Sync,
{
    #[cfg(miri)]
    const COUNT: usize = 50;
    #[cfg(not(miri))]
    const COUNT: usize = 25_000;
    const THREADS: usize = 2;

    let t = AtomicUsize::new(THREADS);
    let v = (0..COUNT).map(|_| AtomicUsize::new(0)).collect::<Vec<_>>();

    scope(|scope| {
        for _ in 0..THREADS {
            scope.spawn(|| {
                loop {
                    match t.load(Ordering::SeqCst) {
                        0 if q.is_empty() => break,

                        _ => {
                            while let Some(n) = q.pop() {
                                v[*n].fetch_add(1, Ordering::SeqCst);
                            }
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
    Q: ForcePushQueue<Item = &'static i32> + Sync,
{
    #[cfg(miri)]
    const COUNT: usize = 100;
    #[cfg(not(miri))]
    const COUNT: usize = 25_000;
    const THREADS: usize = 4;

    scope(|scope| {
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

// pub(crate) fn into_iter<Q>(q: Q)
// wherepub
//     Q: MPMCQueue<Item = &'static usize>,
// {
//     for i in 0..100 {
//         let i: &'static _ = Box::leak(Box::new(i));
//         q.push(i).unwrap();
//     }
//     for (i, j) in q.into_iter().enumerate() {
//         assert_eq!(i, *j);
//     }
// }
