//! Testing for nblfqueue
//!
//! Tests adapted from crossbeam-queue's test suite.
//! <https://github.com/crossbeam-rs/crossbeam/tree/master/crossbeam-queue>

use std::{thread::scope, vec::Vec};

use crate::{
    MPMCQueue,
    core::AsPackedValue,
    sync::atomic::{AtomicUsize, Ordering},
};

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
    assert!(q.is_empty());
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
    assert!(q.is_empty());

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
    assert!(q.is_empty());

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

pub(crate) fn force_push<Q>(q: Q)
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
        assert!(q.force_push(42).is_some_and(|item| item == i as u32));
    }

    assert!(q.is_full());
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
    });
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
    });
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

#[cfg(feature = "pool")]
pub(crate) struct Large;

#[cfg(feature = "pool")]
pub(crate) fn pooled_stores_any<Q>(q: Q)
where
    Q: MPMCQueue<Item = Large>,
{
    assert!(q.push(Large).is_ok());
    assert!(q.pop().is_some());
}

#[cfg(feature = "dynamic")]
pub(crate) use growth::*;

#[cfg(feature = "dynamic")]
mod growth {
    use std::{sync::Arc, thread};

    use super::*;
    use crate::Growable;

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
        assert!(!q.is_full());

        let current_len = q.len();

        for i in initial_cap..(initial_cap * 2) {
            assert!(q.push(i as u32).is_ok());
        }

        assert!(q.len() > current_len);

        for i in 0..(q.len()) {
            assert_eq!(q.pop(), Some(i as u32));
        }

        assert!(q.is_empty());
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
                        while q.push(i as u32).is_err() {
                            _ = q.grow_by(10);
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
                    for _ in 0..100 {
                        q.grow_by(10);
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
        #[cfg(miri)]
        const THREADS: usize = 2;
        #[cfg(not(miri))]
        const THREADS: usize = 8;
        #[cfg(miri)]
        const ITERS: usize = 15;
        #[cfg(not(miri))]
        const ITERS: usize = 2000;

        let tracking_vector = (0..ITERS).map(|_| AtomicUsize::new(0)).collect::<Vec<_>>();

        scope(|scope| {
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

        for count in tracking_vector {
            assert_eq!(count.load(Ordering::SeqCst), THREADS);
        }
    }

    pub(crate) fn oscillation_grow<Q>(q: Q)
    where
        Q: Growable + MPMCQueue<Item = u32> + Sync,
    {
        #[cfg(not(miri))]
        const ITER: usize = 100;
        #[cfg(miri)]
        const ITER: usize = 10;

        let total_popped = Arc::new(AtomicUsize::new(0));
        let total_pushed = Arc::new(AtomicUsize::new(0));

        scope(|scope| {
            scope.spawn(|| {
                for _ in 0..10 {
                    let mut backoff = crate::utils::Backoff::new();
                    for _ in 0..50 {
                        if q.grow_by(10) {
                            break;
                        }
                        backoff.backoff();
                    }
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
        assert!(q.is_empty());

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
                #[cfg(miri)]
                const GROW_ITERS: usize = 3;
                #[cfg(not(miri))]
                const GROW_ITERS: usize = 25;

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
        #[cfg(not(miri))]
        const ITERS: usize = 10_000;
        #[cfg(miri)]
        const ITERS: usize = 30;
        #[cfg(not(miri))]
        const GROW_CYCLES: usize = 500;
        #[cfg(miri)]
        const GROW_CYCLES: usize = 20;
        const GROW_STEP: usize = 10;

        let initial_cap = q.capacity();

        let total_grows = Arc::new(AtomicUsize::new(0));

        scope(|scope| {
            scope.spawn(|| {
                let mut last_cap = initial_cap;
                for _ in 0..ITERS {
                    let current_cap = q.capacity();

                    assert!(
                        current_cap >= last_cap,
                        "Monotonicity broken: Capacity shrank from {last_cap} to {current_cap}!"
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
                    thread::yield_now();
                    backoff.backoff();
                }
            });
        });

        let final_cap = q.capacity();
        let expected_min_cap = initial_cap + (total_grows.load(Ordering::SeqCst) * GROW_STEP);
        assert!(
            final_cap >= expected_min_cap,
            "Structural integrity failed: Expected capacity >= {expected_min_cap}, but got {final_cap}",
        );
    }
}
