cfg_atomic_tagged64! {
    use crate::{
        MPMCQueue,
        Queue,
        core::slots::Tagged64,
        sync::atomic::{AtomicUsize, Ordering},
    };

    #[test]
    fn pops() {
        let total_pops = AtomicUsize::new(0);
        echeneis::check_pairwise(
            || Queue::with_slot::<Tagged64>(5),
            |q| {
                for i in 0..5 {
                    _ = q.push(i as u32);
                }
            },
            |q| {
                if q.pop().is_some() {
                    total_pops.fetch_add(1, Ordering::Release);
                }
            },
        );
        assert_eq!(total_pops.load(Ordering::Acquire), 5);
    }

    #[test]
    fn pushes() {
        let total_pops = AtomicUsize::new(0);

        echeneis::check_pairwise(
            || Queue::with_slot::<Tagged64>(1),
            |q| {
                for _ in 0..5 {
                    while q.pop().is_none() {}
                    total_pops.fetch_add(1, Ordering::Release);
                }
            },
            |q| _ = q.push(42),
        );

        assert_eq!(total_pops.load(Ordering::Acquire), 5);
    }
}

cfg_atomic_tagged128! {
    use crate::{
        MPMCQueue,
        Queue,
        core::slots::Tagged128,
        sync::atomic::{AtomicUsize, Ordering},
    };

    #[test]
    fn pops() {
        let total_pops = AtomicUsize::new(0);
        echeneis::check_pairwise(
            || Queue::with_slot::<Tagged128>(5),
            |q| {
                for i in 0..5 {
                    _ = q.push(i as u32);
                }
            },
            |q| {
                if q.pop().is_some() {
                    total_pops.fetch_add(1, Ordering::Release);
                }
            },
        );
        assert_eq!(total_pops.load(Ordering::Acquire), 5);
    }

    #[test]
    fn pushes() {
        let total_pops = AtomicUsize::new(0);

        echeneis::check_pairwise(
            || Queue::with_slot::<Tagged128>(1),
            |q| {
                for _ in 0..5 {
                    while q.pop().is_none() {}
                    total_pops.fetch_add(1, Ordering::Release);
                }
            },
            |q| _ = q.push(42),
        );

        assert_eq!(total_pops.load(Ordering::Acquire), 5);
    }
}

#[cfg(feature = "pool")]
mod pooled {
    use crate::{
        MPMCQueue,
        PooledQueue,
        sync::atomic::{AtomicUsize, Ordering},
    };

    #[test]
    fn pops() {
        let total_pops = AtomicUsize::new(0);
        echeneis::check_pairwise(
            || PooledQueue::new(5),
            |q| {
                for i in 0..5 {
                    _ = q.push(i as u32);
                }
            },
            |q| {
                if q.pop().is_some() {
                    total_pops.fetch_add(1, Ordering::Release);
                }
            },
        );
        assert_eq!(total_pops.load(Ordering::Acquire), 5);
    }

    #[test]
    fn pushes() {
        let total_pops = AtomicUsize::new(0);

        echeneis::check_pairwise(
            || PooledQueue::new(1),
            |q| {
                for _ in 0..5 {
                    while q.pop().is_none() {}
                    total_pops.fetch_add(1, Ordering::Release);
                }
            },
            |q| _ = q.push(42),
        );

        assert_eq!(total_pops.load(Ordering::Acquire), 5);
    }
}

#[cfg(feature = "dynamic")]
mod growable {
    use crate::{
        DynamicQueue,
        MPMCQueue,
        Resize,
        sync::atomic::{AtomicUsize, Ordering},
    };

    #[test]
    fn pops() {
        let total_pops = AtomicUsize::new(0);
        echeneis::check_pairwise(
            || DynamicQueue::new(5),
            |q| {
                for i in 0..5 {
                    _ = q.push(i as u32);
                }
            },
            |q| {
                if q.pop().is_some() {
                    total_pops.fetch_add(1, Ordering::Release);
                }
            },
        );
        assert_eq!(total_pops.load(Ordering::Acquire), 5);
    }

    #[test]
    fn pushes() {
        let total_pops = AtomicUsize::new(0);

        echeneis::check_pairwise(
            || DynamicQueue::new(1),
            |q| {
                for _ in 0..5 {
                    while q.pop().is_none() {}
                    total_pops.fetch_add(1, Ordering::Release);
                }
            },
            |q| _ = q.push(42),
        );

        assert_eq!(total_pops.load(Ordering::Acquire), 5);
    }

    #[test]
    fn pushes_in_resize() {
        let total_pops = AtomicUsize::new(0);

        echeneis::check_pairwise(
            || DynamicQueue::new(1),
            |q| {
                for _ in 0..5 {
                    _ = q.resize(q.capacity() + 1);
                    while q.pop().is_none() {}
                    total_pops.fetch_add(1, Ordering::Release);
                }
            },
            |q| _ = q.push(42),
        );

        assert_eq!(total_pops.load(Ordering::Acquire), 5);
    }

    #[test]
    fn pops_in_resize() {
        let total_pops = AtomicUsize::new(0);
        echeneis::check_pairwise(
            || DynamicQueue::new(5),
            |q| {
                for i in 0..5 {
                    _ = q.push(i as u32);
                    _ = q.resize(q.capacity() + 1);
                }
            },
            |q| {
                if q.pop().is_some() {
                    total_pops.fetch_add(1, Ordering::Release);
                }
            },
        );
        assert_eq!(total_pops.load(Ordering::Acquire), 5);
    }

    #[cfg(feature = "dynamic")]
    mod pooled {
        use crate::{
            MPMCQueue,
            PooledDynamicQueue,
            Resize,
            sync::atomic::{AtomicUsize, Ordering},
        };

        #[test]
        fn pops() {
            let total_pops = AtomicUsize::new(0);
            echeneis::check_pairwise(
                || PooledDynamicQueue::new(5),
                |q| {
                    for i in 0..5 {
                        _ = q.push(i as u32);
                    }
                },
                |q| {
                    if q.pop().is_some() {
                        total_pops.fetch_add(1, Ordering::Release);
                    }
                },
            );
            assert_eq!(total_pops.load(Ordering::Acquire), 5);
        }

        #[test]
        fn pushes() {
            let total_pops = AtomicUsize::new(0);

            echeneis::check_pairwise(
                || PooledDynamicQueue::new(1),
                |q| {
                    for _ in 0..5 {
                        while q.pop().is_none() {}
                        total_pops.fetch_add(1, Ordering::Release);
                    }
                },
                |q| _ = q.push(42),
            );

            assert_eq!(total_pops.load(Ordering::Acquire), 5);
        }

        #[test]
        fn pushes_in_resize() {
            let total_pops = AtomicUsize::new(0);

            echeneis::check_pairwise(
                || PooledDynamicQueue::new(1),
                |q| {
                    for _ in 0..5 {
                        _ = q.resize(q.capacity() + 1);
                        while q.pop().is_none() {}
                        total_pops.fetch_add(1, Ordering::Release);
                    }
                },
                |q| _ = q.push(42),
            );

            assert_eq!(total_pops.load(Ordering::Acquire), 5);
        }

        #[test]
        fn pops_in_resize() {
            let total_pops = AtomicUsize::new(0);
            echeneis::check_pairwise(
                || PooledDynamicQueue::new(5),
                |q| {
                    for i in 0..5 {
                        _ = q.push(i as u32);
                        _ = q.resize(q.capacity() + 1);
                    }
                },
                |q| {
                    if q.pop().is_some() {
                        total_pops.fetch_add(1, Ordering::Release);
                    }
                },
            );
            assert_eq!(total_pops.load(Ordering::Acquire), 5);
        }
    }
}
