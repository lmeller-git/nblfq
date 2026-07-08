#![allow(dead_code)]
//! Testing for nblfqueue
//!
//! Tests adapted from crossbeam-queue's test suite.
//! <https://github.com/crossbeam-rs/crossbeam/tree/master/crossbeam-queue>

use alloc::vec::Vec;

use crate::{
    MPMCQueue,
    core::AsPackedValue,
    sync::{
        atomic::{AtomicUsize, Ordering},
        thread::scope,
    },
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

#[cfg(any(
    all(target_arch = "x86_64", feature = "unsafe-ptr48"),
    all(target_arch = "aarch64", feature = "unsafe-ptr48"),
    not(target_pointer_width = "64"),
    any(target_has_atomic = "128", feature = "atomic-fallback")
))]
pub(crate) struct Drops(std::rc::Rc<AtomicUsize>);

#[cfg(any(
    all(target_arch = "x86_64", feature = "unsafe-ptr48"),
    all(target_arch = "aarch64", feature = "unsafe-ptr48"),
    not(target_pointer_width = "64"),
    any(target_has_atomic = "128", feature = "atomic-fallback")
))]
impl Drop for Drops {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::Release);
    }
}

#[cfg(any(
    all(target_arch = "x86_64", feature = "unsafe-ptr48"),
    all(target_arch = "aarch64", feature = "unsafe-ptr48"),
    not(target_pointer_width = "64"),
    any(target_has_atomic = "128", feature = "atomic-fallback")
))]
pub(crate) fn drops<Q>(q: Q)
where
    Q: MPMCQueue<Item = Box<Drops>>,
{
    let counter = std::rc::Rc::new(AtomicUsize::new(q.capacity()));

    for _ in 0..q.capacity() {
        assert!(q.push(Box::new(Drops(counter.clone()))).is_ok());
    }

    drop(q);

    assert_eq!(counter.load(Ordering::Acquire), 0);
}

pub(crate) fn len<Q>(q: Q)
where
    Q: MPMCQueue<Item = u32> + Sync,
{
    #[cfg(any(miri, loom, shuttle))]
    const COUNT: usize = 30;
    #[cfg(not(any(miri, loom, shuttle)))]
    const COUNT: usize = 25_000;
    #[cfg(any(miri, loom, shuttle))]
    const CAP: usize = 40;
    #[cfg(not(any(miri, loom, shuttle)))]
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
    #[cfg(any(miri, loom, shuttle))]
    const COUNT: usize = 50;
    #[cfg(not(any(miri, loom, shuttle)))]
    const COUNT: usize = 300_000;

    scope(|scope| {
        scope.spawn(|| {
            for i in 0..COUNT {
                loop {
                    if let Some(x) = q.pop() {
                        assert_eq!(x, i as u32);
                        break;
                    }
                    crate::utils::Backoff::new().backoff();
                }
            }
            assert!(q.pop().is_none());
        });

        scope.spawn(|| {
            for i in 0..COUNT {
                while q.push(i as u32).is_err() {
                    crate::utils::Backoff::new().backoff();
                }
            }
        });
    });
}

pub(crate) fn mpsc<Q>(q: Q)
where
    Q: MPMCQueue<Item = u32> + Sync,
{
    #[cfg(any(miri, loom, shuttle))]
    const COUNT: usize = 10;
    #[cfg(not(any(miri, loom, shuttle)))]
    const COUNT: usize = 30_000;
    const THREADS: usize = 4;

    let v = (0..COUNT).map(|_| AtomicUsize::new(0)).collect::<Vec<_>>();

    scope(|scope| {
        for _ in 0..THREADS {
            scope.spawn(|| {
                for i in 0..COUNT {
                    while q.push(i as u32).is_err() {
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

pub(crate) fn mpmc<Q>(q: Q)
where
    Q: MPMCQueue<Item = u32> + Sync,
{
    #[cfg(any(miri, loom, shuttle))]
    const COUNT: usize = 20;
    #[cfg(not(any(miri, loom, shuttle)))]
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
                        crate::utils::Backoff::new().backoff();
                    };
                    v[n as usize].fetch_add(1, Ordering::SeqCst);
                }
            });
        }
        for _ in 0..THREADS {
            scope.spawn(|| {
                for i in 0..COUNT {
                    while q.push(i as u32).is_err() {
                        crate::utils::Backoff::new().backoff();
                    }
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
    #[cfg(any(miri, loom, shuttle))]
    const COUNT: usize = 20;
    #[cfg(not(any(miri, loom, shuttle)))]
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
                    crate::utils::Backoff::new().backoff();
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
    #[cfg(any(miri, loom, shuttle))]
    const COUNT: usize = 50;
    #[cfg(not(any(miri, loom, shuttle)))]
    const COUNT: usize = 25_000;
    const THREADS: usize = 4;

    scope(|scope| {
        for _ in 0..THREADS / 2 {
            scope.spawn(|| {
                for _ in 0..COUNT {
                    while q.push(42).is_err() {
                        crate::utils::Backoff::new().backoff();
                    }
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

#[cfg(any(
    all(target_arch = "x86_64", feature = "unsafe-ptr48"),
    all(target_arch = "aarch64", feature = "unsafe-ptr48"),
    not(target_pointer_width = "64"),
    any(target_has_atomic = "128", feature = "atomic-fallback")
))]
pub(crate) fn mpmc_ring_buf_ptr<Q>(q: Q)
where
    Q: MPMCQueue<Item = Box<usize>> + Sync,
{
    #[cfg(any(miri, loom, shuttle))]
    const COUNT: usize = 50;
    #[cfg(not(any(miri, loom, shuttle)))]
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
                    crate::utils::Backoff::new().backoff();
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
    use crate::Resize;

    pub(crate) fn smoke_grow<Q>(q: Q)
    where
        Q: Resize + MPMCQueue<Item = u32>,
    {
        let initial_cap = q.capacity();

        for i in 0..initial_cap {
            assert!(q.push(i as u32).is_ok());
        }

        assert!(q.is_full());
        assert!(q.push(42).is_err());

        assert!(q.resize(initial_cap * 2));
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

    pub(crate) fn smoke_shrink<Q>(q: Q)
    where
        Q: Resize + MPMCQueue<Item = u32>,
    {
        let initial_cap = q.capacity();

        for i in 0..initial_cap {
            assert!(q.push(i as u32).is_ok());
        }

        assert!(q.is_full());
        assert!(q.push(42).is_err());

        assert!(q.resize(initial_cap / 2));
        assert_eq!(q.capacity(), initial_cap / 2);

        assert!(!q.is_empty());

        let current_len = q.len();

        for _ in 0..q.len() {
            assert!(q.pop().is_some());
        }

        assert!(q.pop().is_none());

        assert!(q.is_empty());
        assert!(q.len() < current_len);

        assert!(q.resize(1));
        assert_eq!(q.capacity(), 1);
        assert!(q.push(42).is_ok());
        assert!(q.is_full());
    }

    pub(crate) fn mpsc_grow<Q>(q: Q)
    where
        Q: Resize + MPMCQueue<Item = u32> + Sync,
    {
        #[cfg(any(miri, loom, shuttle))]
        const COUNT: usize = 20;
        #[cfg(not(any(miri, loom, shuttle)))]
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
                            _ = q.resize(GROW_STEP + q.capacity());
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

    pub(crate) fn mpmc_resize<Q>(q: Q)
    where
        Q: Resize + MPMCQueue<Item = u32> + Sync,
    {
        #[cfg(any(miri, loom, shuttle))]
        const COUNT: usize = 30;
        #[cfg(not(any(miri, loom, shuttle)))]
        const COUNT: usize = 75_000;
        #[cfg(any(miri, loom, shuttle))]
        const RESIZE_ITER: usize = 5;
        #[cfg(not(any(miri, loom, shuttle)))]
        const RESIZE_ITER: usize = 100;
        const RESIZERS: usize = 2;
        const THREADS: usize = 4;

        let v = (0..COUNT).map(|_| AtomicUsize::new(0)).collect::<Vec<_>>();

        scope(|scope| {
            for _ in 0..THREADS {
                scope.spawn(|| {
                    for i in 0..COUNT {
                        while q.push(i as u32).is_err() {
                            _ = q.resize(10 + q.capacity());
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
                            crate::utils::Backoff::new().backoff();
                        };
                        v[n as usize].fetch_add(1, Ordering::SeqCst);
                    }
                });
            }

            for _ in 0..RESIZERS {
                scope.spawn(|| {
                    let mut backoff = crate::utils::Backoff::new();
                    for _ in 0..RESIZE_ITER {
                        q.resize(2 + q.capacity());
                        backoff.backoff();
                    }
                });
            }

            for _ in 0..RESIZERS {
                scope.spawn(|| {
                    let mut backoff = crate::utils::Backoff::new();
                    for _ in 0..RESIZE_ITER {
                        q.resize(q.capacity().max(2) - 2);
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
        Q: Resize + MPMCQueue<Item = u32> + Sync,
    {
        #[cfg(any(miri, loom, shuttle))]
        const THREADS: usize = 2;
        #[cfg(not(any(miri, loom, shuttle)))]
        const THREADS: usize = 8;
        #[cfg(any(miri, loom, shuttle))]
        const ITERS: usize = 10;
        #[cfg(not(any(miri, loom, shuttle)))]
        const ITERS: usize = 2000;

        let tracking_vector = (0..ITERS).map(|_| AtomicUsize::new(0)).collect::<Vec<_>>();

        scope(|scope| {
            for _ in 0..THREADS {
                scope.spawn(|| {
                    for i in 0..ITERS {
                        if i % 5 == 0 {
                            let _ = q.resize(2 + q.capacity());
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
                            let _ = q.resize(1 + q.capacity());
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
        Q: Resize + MPMCQueue<Item = u32> + Sync,
    {
        #[cfg(not(any(miri, loom, shuttle)))]
        const ITER: usize = 100;
        #[cfg(any(miri, loom, shuttle))]
        const ITER: usize = 10;

        let total_popped = Arc::new(AtomicUsize::new(0));
        let total_pushed = Arc::new(AtomicUsize::new(0));

        scope(|scope| {
            scope.spawn(|| {
                for _ in 0..10 {
                    let mut backoff = crate::utils::Backoff::new();
                    for _ in 0..50 {
                        if q.resize(10 + q.capacity()) {
                            break;
                        }
                        backoff.backoff();
                    }
                    thread::yield_now();
                }
            });

            scope.spawn(|| {
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
        Q: MPMCQueue<Item = u32> + Sync + Resize,
    {
        #[cfg(any(miri, loom, shuttle))]
        const COUNT: usize = 30;
        #[cfg(not(any(miri, loom, shuttle)))]
        const COUNT: usize = 20_000;
        #[cfg(any(miri, loom, shuttle))]
        const CAP: usize = 40;
        #[cfg(not(any(miri, loom, shuttle)))]
        const CAP: usize = 500;
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
                        crate::utils::Backoff::new().backoff();
                    }
                    let _len = q.len();
                }
            });

            scope.spawn(|| {
                for i in 0..COUNT {
                    let mut backoff = crate::utils::Backoff::new();
                    while q.push(i as u32).is_err() {
                        backoff.backoff();
                    }
                    let _len = q.len();
                }
            });

            scope.spawn(|| {
                #[cfg(any(miri, loom, shuttle))]
                const GROW_ITERS: usize = 3;
                #[cfg(not(any(miri, loom, shuttle)))]
                const GROW_ITERS: usize = 25;

                let mut backoff = crate::utils::Backoff::new();
                for _ in 0..GROW_ITERS {
                    _ = q.resize(CAP / 2 + q.capacity());
                    backoff.backoff();
                }
            });
        });

        assert_eq!(q.len(), 0);
    }

    pub(crate) fn suppl_methods_chaos<Q>(q: Q)
    where
        Q: Resize + MPMCQueue<Item = u32> + Sync,
    {
        #[cfg(not(any(miri, loom, shuttle)))]
        const ITERS: usize = 10_000;
        #[cfg(any(miri, loom, shuttle))]
        const ITERS: usize = 30;
        #[cfg(not(any(miri, loom, shuttle)))]
        const GROW_CYCLES: usize = 500;
        #[cfg(any(miri, loom, shuttle))]
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

                    _ = q.is_full();
                }
            });

            scope.spawn(|| {
                for _ in 0..ITERS {
                    _ = q.len();
                    _ = q.is_empty();
                }
            });

            scope.spawn(|| {
                for i in 0..ITERS {
                    _ = q.push(i as u32);
                    _ = q.pop();
                }
            });

            scope.spawn(|| {
                for _ in 0..GROW_CYCLES {
                    if q.resize(GROW_STEP + q.capacity()) {
                        total_grows.fetch_add(1, Ordering::SeqCst);
                    }
                    thread::yield_now();
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

    #[cfg(any(
        all(target_arch = "x86_64", feature = "unsafe-ptr48"),
        all(target_arch = "aarch64", feature = "unsafe-ptr48"),
        not(target_pointer_width = "64")
    ))]
    pub(crate) fn drops_resized<Q>(q: Q)
    where
        Q: MPMCQueue<Item = Box<Drops>> + Resize,
    {
        let counter = std::rc::Rc::new(AtomicUsize::new(q.capacity() + 5));

        for _ in 0..q.capacity() {
            assert!(q.push(Box::new(Drops(counter.clone()))).is_ok());
        }

        assert!(q.resize(5));

        for _ in 0..5 {
            assert!(q.push(Box::new(Drops(counter.clone()))).is_ok());
        }

        drop(q);

        assert_eq!(counter.load(Ordering::Acquire), 0);
    }
}
