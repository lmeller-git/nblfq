use crate::{
    MPMCQueue,
    Queue,
    sync::atomic::{AtomicUsize, Ordering},
};

cfg_atomic_tagged64! {
    use crate::core::slots::Tagged64;

    #[test]
    fn pops64() {
        let total_pops = AtomicUsize::new(0);
        echeneis::check_retry(
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
                std::ops::ControlFlow::Continue::<()>(())
            },
        );
        assert_eq!(total_pops.load(Ordering::Acquire), 5);
    }

    #[test]
    fn pushes64() {
        let total_pops = AtomicUsize::new(0);

        echeneis::check_retry(
            || Queue::with_slot::<Tagged64>(1),
            |q| {
                for _ in 0..5 {
                    while q.pop().is_none() {}
                    total_pops.fetch_add(1, Ordering::Release);
                }
            },
            |q| {
                _ = q.push(42);
                std::ops::ControlFlow::Continue::<()>(())
            },
        );

        assert_eq!(total_pops.load(Ordering::Acquire), 5);
    }
}

cfg_atomic_tagged128! {
    use crate::core::slots::Tagged128;

    #[test]
    fn pops128() {
        let total_pops = AtomicUsize::new(0);
        echeneis::check_retry(
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
                std::ops::ControlFlow::Continue::<()>(())
            },
        );
        assert_eq!(total_pops.load(Ordering::Acquire), 5);
    }

    #[test]
    fn pushes128() {
        let total_pops = AtomicUsize::new(0);

        echeneis::check_retry(
            || Queue::with_slot::<Tagged128>(1),
            |q| {
                for _ in 0..5 {
                    while q.pop().is_none() {}
                    total_pops.fetch_add(1, Ordering::Release);
                }
            },
            |q| {
                _ = q.push(42);
                std::ops::ControlFlow::Continue::<()>(())
            },
        );

        assert_eq!(total_pops.load(Ordering::Acquire), 5);
}
}

#[cfg(feature = "pool")]
mod pooled {
    use super::*;
    use crate::PooledQueue;

    #[test]
    fn test_pops() {
        let total_pops = AtomicUsize::new(0);
        echeneis::check_retry(
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
                std::ops::ControlFlow::Continue::<()>(())
            },
        );
        assert_eq!(total_pops.load(Ordering::Acquire), 5);
    }

    #[test]
    fn test_pushes() {
        let total_pops = AtomicUsize::new(0);

        echeneis::check_retry(
            || PooledQueue::new(1),
            |q| {
                for _ in 0..5 {
                    while q.pop().is_none() {}
                    total_pops.fetch_add(1, Ordering::Release);
                }
            },
            |q| {
                _ = q.push(42);
                std::ops::ControlFlow::Continue::<()>(())
            },
        );

        assert_eq!(total_pops.load(Ordering::Acquire), 5);
    }
}

#[cfg(feature = "dynamic")]
mod growable {
    use super::*;
    use crate::{DynamicQueue, Resize};

    #[test]
    fn test_pops() {
        let total_pops = AtomicUsize::new(0);
        echeneis::check_retry(
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
                std::ops::ControlFlow::Continue::<()>(())
            },
        );
        assert_eq!(total_pops.load(Ordering::Acquire), 5);
    }

    #[test]
    fn test_pushes() {
        let total_pops = AtomicUsize::new(0);

        echeneis::check_retry(
            || DynamicQueue::new(1),
            |q| {
                for _ in 0..5 {
                    while q.pop().is_none() {}
                    total_pops.fetch_add(1, Ordering::Release);
                }
            },
            |q| {
                _ = q.push(42);
                std::ops::ControlFlow::Continue::<()>(())
            },
        );

        assert_eq!(total_pops.load(Ordering::Acquire), 5);
    }

    #[test]
    fn test_pushes_in_retry() {
        let total_pops = AtomicUsize::new(0);

        echeneis::check_retry(
            || DynamicQueue::new(1),
            |q| {
                for _ in 0..5 {
                    _ = q.resize(q.capacity() + 1);
                    while q.pop().is_none() {}
                    total_pops.fetch_add(1, Ordering::Release);
                }
            },
            |q| {
                _ = q.push(42);
                std::ops::ControlFlow::Continue::<()>(())
            },
        );

        assert_eq!(total_pops.load(Ordering::Acquire), 5);
    }

    #[test]
    fn test_pops_in_resize() {
        let total_pops = AtomicUsize::new(0);
        echeneis::check_retry(
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
                std::ops::ControlFlow::Continue::<()>(())
            },
        );
        assert_eq!(total_pops.load(Ordering::Acquire), 5);
    }

    #[cfg(feature = "dynamic")]
    mod pooled {
        use super::*;
        use crate::PooledDynamicQueue;

        #[test]
        fn test_pops() {
            let total_pops = AtomicUsize::new(0);
            echeneis::check_retry(
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
                    std::ops::ControlFlow::Continue::<()>(())
                },
            );
            assert_eq!(total_pops.load(Ordering::Acquire), 5);
        }

        #[test]
        fn test_pushes() {
            let total_pops = AtomicUsize::new(0);

            echeneis::check_retry(
                || PooledDynamicQueue::new(1),
                |q| {
                    for _ in 0..5 {
                        while q.pop().is_none() {}
                        total_pops.fetch_add(1, Ordering::Release);
                    }
                },
                |q| {
                    _ = q.push(42);
                    std::ops::ControlFlow::Continue::<()>(())
                },
            );

            assert_eq!(total_pops.load(Ordering::Acquire), 5);
        }

        #[test]
        fn test_pushes_in_resize() {
            let total_pops = AtomicUsize::new(0);

            echeneis::check_retry(
                || PooledDynamicQueue::new(1),
                |q| {
                    for _ in 0..5 {
                        _ = q.resize(q.capacity() + 1);
                        while q.pop().is_none() {}
                        total_pops.fetch_add(1, Ordering::Release);
                    }
                },
                |q| {
                    _ = q.push(42);
                    std::ops::ControlFlow::Continue::<()>(())
                },
            );

            assert_eq!(total_pops.load(Ordering::Acquire), 5);
        }

        #[test]
        fn test_pops_in_resize() {
            let total_pops = AtomicUsize::new(0);
            echeneis::check_retry(
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
                    std::ops::ControlFlow::Continue::<()>(())
                },
            );
            assert_eq!(total_pops.load(Ordering::Acquire), 5);
        }
    }
}
