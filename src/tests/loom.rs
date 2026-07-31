#[cfg(feature = "dynamic")]
use crate::Resize;
#[cfg(feature = "dynamic")]
use crate::sync::atomic::AtomicBool;
use crate::{
    MPMCQueue,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
        thread,
    },
};

pub(crate) fn linearizable<Q>(q: Q)
where
    Q: MPMCQueue<Item = u32> + Sync + 'static,
{
    const COUNT: usize = 1;
    const THREADS: usize = 2;
    let q = Arc::new(q);

    let mut threads = Vec::new();

    for _ in 0..THREADS / 2 {
        let q2 = q.clone();
        threads.push(thread::spawn(move || {
            for _ in 0..COUNT {
                while q2.push(42).is_err() {
                    thread::yield_now();
                }
                q2.pop().unwrap();
            }
        }));

        let q = q.clone();
        threads.push(thread::spawn(move || {
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
        }));
    }

    for t in threads {
        t.join().unwrap();
    }
}

pub(crate) fn spsc<Q>(q: Q)
where
    Q: MPMCQueue<Item = u32> + Sync + Send + 'static,
{
    const COUNT: usize = 2;

    let q = Arc::new(q);

    let q_consumer = q.clone();
    let consumer = thread::spawn(move || {
        for i in 0..COUNT {
            loop {
                if let Some(x) = q_consumer.pop() {
                    assert_eq!(x, i as u32);
                    break;
                }
                crate::utils::Backoff::new().backoff();
            }
        }
        assert!(q_consumer.pop().is_none());
    });

    let q_producer = q.clone();
    let producer = thread::spawn(move || {
        for i in 0..COUNT {
            while q_producer.push(i as u32).is_err() {
                crate::utils::Backoff::new().backoff();
            }
        }
    });

    consumer.join().unwrap();
    producer.join().unwrap();
}

pub(crate) fn mpsc<Q>(q: Q)
where
    Q: MPMCQueue<Item = u32> + Sync + Send + 'static,
{
    const COUNT: usize = 2;
    const THREADS: usize = 2;

    let q = Arc::new(q);
    let v = Arc::new((0..COUNT).map(|_| AtomicUsize::new(0)).collect::<Vec<_>>());

    let handles: Vec<_> = (0..THREADS)
        .map(|_| {
            let q = q.clone();
            thread::spawn(move || {
                for i in 0..COUNT {
                    while q.push(i as u32).is_err() {
                        crate::utils::Backoff::new().backoff();
                    }
                }
            })
        })
        .collect();

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

    for h in handles {
        h.join().unwrap();
    }

    for c in v.iter() {
        assert_eq!(c.load(Ordering::SeqCst), THREADS);
    }
}

#[cfg(feature = "dynamic")]
pub(crate) fn push_pop_resize<Q>(q: Q)
where
    Q: MPMCQueue<Item = i32> + Resize + Sync + Send + 'static,
{
    const ITER: usize = 2;
    const RESIZE_ITER: usize = 1;

    let q = Arc::new(q);
    let received = Arc::new(
        (0..ITER)
            .map(|_| AtomicBool::new(false))
            .collect::<Vec<_>>(),
    );

    let q1 = q.clone();
    let push = thread::Builder::new()
        .name("push".into())
        .spawn(move || {
            for i in 0..ITER {
                while q1.push(i as i32).is_err() {
                    thread::yield_now();
                }
            }
        })
        .unwrap();

    let q2 = q.clone();
    let resize = thread::Builder::new()
        .name("resize".into())
        .spawn(move || {
            for _ in 0..RESIZE_ITER {
                _ = q2.resize(q2.capacity() + 1);
                thread::yield_now();
            }
        })
        .unwrap();

    let q3 = q.clone();
    let rec = received.clone();
    let pop = thread::Builder::new()
        .name("pop".into())
        .spawn(move || {
            for i in 0..ITER {
                let item = loop {
                    if let Some(x) = q3.pop() {
                        break x;
                    }
                    thread::yield_now();
                };
                let prev_seen = rec[item as usize].swap(true, Ordering::SeqCst);
                assert!(!prev_seen, "Duplicate item popped: {}", item);
            }
        })
        .unwrap();

    push.join().unwrap();
    pop.join().unwrap();
    resize.join().unwrap();

    assert!(received.iter().all(|seen| seen.load(Ordering::SeqCst)));
    assert_eq!(q.len(), 0);
}

#[cfg(feature = "dynamic")]
pub(crate) fn linearizable_during_resize<Q>(q: Q)
where
    Q: MPMCQueue<Item = u32> + Resize + Sync + 'static,
{
    const COUNT: usize = 1;
    const THREADS: usize = 2;
    let q = Arc::new(q);

    let mut threads = Vec::new();

    for _ in 0..THREADS / 2 {
        let q2 = q.clone();
        threads.push(thread::spawn(move || {
            for _ in 0..COUNT {
                while q2.push(42).is_err() {
                    thread::yield_now();
                }
                q2.pop().unwrap();
            }
        }));

        let q = q.clone();
        threads.push(thread::spawn(move || {
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
        }));
    }

    let q = q.clone();
    threads.push(thread::spawn(move || {
        _ = q.resize(q.capacity() + 1);
    }));

    for t in threads {
        t.join().unwrap();
    }
}

cfg_atomic_tagged64! {
    mod taggedptr64 {
        use super::*;
        use crate::{Queue, core::slots::Tagged64};

        #[test]
        fn linearizable_impl() {
            loom::model(|| {
                let q = Queue::with_slot::<Tagged64>(2);
                linearizable(q);
            });
        }

        #[test]
        fn spsc_impl() {
            loom::model(|| {
                let q = Queue::with_slot::<Tagged64>(3);
                spsc(q);
            });
        }

        #[test]
        fn mpsc_impl() {
            loom::model(|| {
                let q = Queue::with_slot::<Tagged64>(3);
                mpsc(q);
            });
        }
    }
}

cfg_atomic_tagged128! {
    mod taggedptr128 {
        use crate::{Queue, core::slots::Tagged128};

        use super::*;

        #[test]
        fn linearizable_impl() {
            loom::model(|| {
                let q = Queue::with_slot::<Tagged128>(2);
                linearizable(q);
            });
        }

        #[test]
        fn spsc_impl() {
            loom::model(|| {
                let q = Queue::with_slot::<Tagged128>(3);
                spsc(q);
            });
        }

        #[test]
        fn mpsc_impl() {
            loom::model(|| {
                let q = Queue::with_slot::<Tagged128>(3);
                mpsc(q);
            });
        }
    }
}

#[cfg(all(feature = "pool", feature = "alloc"))]
mod pooled {
    use super::*;
    use crate::PooledQueue;

    #[test]
    fn linearizable_impl() {
        loom::model(|| {
            let q = PooledQueue::new(2);
            linearizable(q);
        });
    }

    #[test]
    fn spsc_impl() {
        loom::model(|| {
            let q = PooledQueue::new(3);
            spsc(q);
        });
    }

    #[test]
    fn mpsc_impl() {
        loom::model(|| {
            let q = PooledQueue::new(3);
            mpsc(q);
        });
    }
}

#[cfg(feature = "dynamic")]
mod growable {
    use super::*;
    use crate::DynamicQueue;

    #[test]
    fn linearizable_impl() {
        loom::model(|| {
            let q = DynamicQueue::new(2);
            linearizable(q);
        });
    }

    #[test]
    fn spsc_impl() {
        loom::model(|| {
            let q = DynamicQueue::new(3);
            spsc(q);
        });
    }

    #[test]
    fn mpsc_impl() {
        loom::model(|| {
            let q = DynamicQueue::new(3);
            mpsc(q);
        });
    }

    #[test]
    fn push_pop_resize_impl() {
        loom::model(|| {
            let q = DynamicQueue::new(1);
            push_pop_resize(q);
        });
    }

    #[test]
    fn linearizable_during_resize_impl() {
        loom::model(|| {
            let q = DynamicQueue::new(2);
            linearizable_during_resize(q);
        });
    }

    #[cfg(feature = "pool")]
    mod pool {
        use super::*;
        use crate::PooledDynamicQueue;

        #[test]
        fn linearizable_impl() {
            loom::model(|| {
                let q = PooledDynamicQueue::new(2);
                linearizable(q);
            });
        }

        #[test]
        fn spsc_impl() {
            loom::model(|| {
                let q = PooledDynamicQueue::new(3);
                spsc(q);
            });
        }

        #[test]
        fn mpsc_impl() {
            loom::model(|| {
                let q = PooledDynamicQueue::new(3);
                mpsc(q);
            });
        }

        #[test]
        fn push_pop_resize_impl() {
            loom::model(|| {
                let q = PooledDynamicQueue::new(1);
                push_pop_resize(q);
            });
        }

        #[test]
        fn linearizable_during_resize_impl() {
            loom::model(|| {
                let q = PooledDynamicQueue::new(2);
                linearizable_during_resize(q);
            });
        }
    }
}
