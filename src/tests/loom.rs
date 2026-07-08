use crate::{
    MPMCQueue,
    sync::{Arc, thread},
};

// TODO add more tests, however even simple tests are already too large...

// way to small i think
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
                while q2.push(42).is_err() {}
                thread::yield_now();
                q2.pop().unwrap();
            }
        }));

        let q = q.clone();
        threads.push(thread::spawn(move || {
            for _ in 0..COUNT {
                if q.force_push(42).is_none() {
                    q.pop().unwrap();
                }
            }
        }));
    }

    for t in threads {
        t.join().unwrap();
    }
}

cfg_atomic_tagged64! {
    mod taggedptr64 {
        use crate::{Queue, core::slots::Tagged64};

        use super::*;

        #[test]
        fn linearizable_impl() {
            loom::model(|| {
                let q = Queue::with_slot::<Tagged64>(4);
                linearizable(q);
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
    }
}
