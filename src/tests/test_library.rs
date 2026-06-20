//! Testing for nblfqueue
//!
//! Tests adapted from crossbeam-queue's test suite.
//! https://github.com/crossbeam-rs/crossbeam/tree/master/crossbeam-queue

use core::sync::atomic::{AtomicUsize, Ordering};
use std::{thread::scope, vec::Vec};

use crate::{Growable, MPMCQueue, core::AsPackedValue};

pub(crate) fn smoke<Q>(q: Q)
where
    Q: MPMCQueue<Item = u32>,
{
    q.push(7).unwrap();
    assert_eq!(q.pop(), Some(7));
    q.push(8).unwrap();
    assert_eq!(q.pop(), Some(8));
    assert!(q.pop().is_none());
}

pub(crate) fn smoke_long<Q>(q: Q)
where
    Q: MPMCQueue<Item = u32>,
{
    q.push(7).unwrap();
    assert_eq!(q.pop(), Some(7));
    q.push(8).unwrap();
    q.push(9).unwrap();
    assert_eq!(q.pop(), Some(8));
    assert_eq!(q.pop(), Some(9));
    assert!(q.pop().is_none());
}

pub(crate) fn len_empty_full<Q>(q: Q)
where
    Q: MPMCQueue<Item = ()>,
{
    assert_eq!(q.len(), 0);
    assert!(q.is_empty());
    assert!(!q.is_full());

    q.push(()).unwrap();

    assert_eq!(q.len(), 1);
    assert!(!q.is_empty());
    assert!(!q.is_full());

    q.push(()).unwrap();

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
    Q: MPMCQueue<Item = u32> + Sync,
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
            q.push(i as u32).unwrap();
            assert_eq!(q.len(), i + 1);
        }

        for i in 0..ITERS {
            q.pop().unwrap();
            assert_eq!(q.len(), ITERS - i - 1);
        }
    }
    assert_eq!(q.len(), 0);

    for i in 0..CAP {
        q.push(i as u32).unwrap();
        assert_eq!(q.len(), i + 1);
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
                        assert_eq!(x, i as u32);
                        break;
                    }
                }
                let len = q.len();
                assert!(len <= CAP);
            }
        });

        scope.spawn(|| {
            for i in 0..COUNT {
                while q.push(i as u32).is_err() {}
                let len = q.len();
                assert!(len <= CAP);
            }
        });
    });
    assert_eq!(q.len(), 0);
}

pub fn force_push<Q>(q: Q)
where
    Q: MPMCQueue<Item = u32>,
{
    assert!(q.is_empty());

    for i in 0..q.capacity() {
        assert!(q.push(i as u32).is_ok());
    }

    assert!(q.is_full());

    assert!(q.push(42).is_err());

    for i in 0..q.capacity() {
        assert!(q.force_push(42).is_some_and(|item| item == i as u32))
    }

    assert!(q.is_full())
}

pub(crate) fn spsc<Q>(q: Q)
where
    Q: MPMCQueue<Item = u32> + Sync,
{
    #[cfg(miri)]
    const COUNT: usize = 50;
    #[cfg(not(miri))]
    const COUNT: usize = 300_000;

    scope(|scope| {
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
    #[cfg(miri)]
    const COUNT: usize = 10;
    #[cfg(not(miri))]
    const COUNT: usize = 30_000;
    const THREADS: usize = 4;

    let v = (0..COUNT).map(|_| AtomicUsize::new(0)).collect::<Vec<_>>();

    scope(|scope| {
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
    #[cfg(miri)]
    const COUNT: usize = 50;
    #[cfg(not(miri))]
    const COUNT: usize = 75_000;
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
    #[cfg(miri)]
    const COUNT: usize = 50;
    #[cfg(not(miri))]
    const COUNT: usize = 75_000;
    const THREADS: usize = 2;

    let t = AtomicUsize::new(THREADS);
    let v = (0..COUNT).map(|_| AtomicUsize::new(0)).collect::<Vec<_>>();

    scope(|scope| {
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
                    })
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
    #[cfg(miri)]
    const COUNT: usize = 100;
    #[cfg(not(miri))]
    const COUNT: usize = 25_000;
    const THREADS: usize = 4;

    scope(|scope| {
        for _ in 0..THREADS / 2 {
            scope.spawn(|| {
                for _ in 0..COUNT {
                    while q.push(42).is_err() {}
                    q.pop().unwrap();
                }
            });

            scope.spawn(|| {
                for _ in 0..COUNT {
                    let popped = &mut false;
                    q.force_push_and_do(42, |_| {
                        if *popped {
                            panic!("popped multiple items")
                        }
                        *popped = true;
                    });
                    if !*popped {
                        q.pop().unwrap();
                    }
                }
            });
        }
    })
}

pub(crate) fn mpmc_ring_buf_ptr<Q>(q: Q)
where
    Q: MPMCQueue<Item = Box<usize>> + Sync,
{
    #[cfg(miri)]
    const COUNT: usize = 50;
    #[cfg(not(miri))]
    const COUNT: usize = 75_000;
    const THREADS: usize = 2;

    let t = AtomicUsize::new(THREADS);
    let v = (0..COUNT).map(|_| AtomicUsize::new(0)).collect::<Vec<_>>();

    scope(|scope| {
        for _ in 0..THREADS {
            scope.spawn(|| {
                loop {
                    match t.load(Ordering::SeqCst) {
                        0 => {
                            while let Some(n) = q.pop() {
                                v[*n].fetch_add(1, Ordering::SeqCst);
                            }
                            break;
                        }

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
                    q.force_push_and_do(Box::new(i), |n| {
                        v[*n].fetch_add(1, Ordering::SeqCst);
                    })
                }

                t.fetch_sub(1, Ordering::SeqCst);
            });
        }
    });

    for c in v {
        assert_eq!(c.load(Ordering::SeqCst), THREADS);
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct MaliciousCargo(pub(crate) u128);

// Safety:
// This is not safe. Its intent is simply to check if queues allow using this clearly unsafe type.
unsafe impl AsPackedValue for MaliciousCargo {
    const MIN_BIT_WIDTH: usize = 48;

    fn encode(zelf: Self) -> crate::core::TruncatedU64<Self> {
        crate::core::TruncatedU64::new(zelf.0 as u64)
    }

    unsafe fn decode(raw: crate::core::TruncatedU64<Self>) -> Self {
        Self(raw.read() as u128)
    }

    fn is_rt_safe() -> bool {
        let zelf = Self(u128::MAX);

        let encoded = Self::encode(zelf);
        // Safety:
        // this is safe, becasue we do not do any memory accesses with this value. It is simply some number.
        let decoded = unsafe { Self::decode(encoded) };

        decoded == zelf
    }
}

pub(crate) fn smoke_grow<Q>(q: Q)
where
    Q: Growable + MPMCQueue<Item = u32>,
{
    let initial_cap = q.capacity();

    for i in 0..initial_cap {
        assert!(q.push(i as u32).is_ok());
    }

    assert!(q.is_full());
    assert!(q.push(42).is_err());

    assert!(q.grow_by(initial_cap));
    assert_eq!(q.capacity(), initial_cap * 2);

    for i in initial_cap..(initial_cap * 2) {
        assert!(q.push(i as u32).is_ok());
    }

    for i in 0..(initial_cap * 2) {
        assert_eq!(q.pop(), Some(i as u32));
    }
}

pub(crate) fn mpsc_grow<Q>(q: Q)
where
    Q: Growable + MPMCQueue<Item = u32> + Sync,
{
    #[cfg(miri)]
    const COUNT: usize = 100;
    #[cfg(not(miri))]
    const COUNT: usize = 10_000;
    const THREADS: usize = 4;
    const GROW_STEP: usize = 10;

    let v = (0..COUNT).map(|_| AtomicUsize::new(0)).collect::<Vec<_>>();

    scope(|scope| {
        for _ in 0..THREADS {
            scope.spawn(|| {
                for i in 0..COUNT {
                    loop {
                        if q.push(i as u32).is_ok() {
                            break;
                        }
                        _ = q.grow_by(GROW_STEP);
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
                    crate::utils::Backoff::new().backoff();
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
    #[cfg(miri)]
    const COUNT: usize = 50;
    #[cfg(not(miri))]
    const COUNT: usize = 75_000;
    const RESIZERS: usize = 2;
    const THREADS: usize = 4;

    let v = (0..COUNT).map(|_| AtomicUsize::new(0)).collect::<Vec<_>>();

    scope(|scope| {
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
                for _ in 0..100 {
                    q.grow_by(10);
                    crate::utils::Backoff::new().backoff();
                }
            });
        }
    });

    for c in v {
        assert_eq!(c.load(Ordering::SeqCst), THREADS);
    }
}
