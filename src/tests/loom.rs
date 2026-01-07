use crate::{MPMCQueue, cfg_taggedptr64, cfg_taggedptr128};

use crate::sync::{Arc, thread};

// TODO add more tests, however even simple tests are already too large...

// way to small i think
pub(crate) fn linearizable<Q>(q: Q)
where
    Q: MPMCQueue<Item = &'static i32> + Sync + 'static,
{
    const COUNT: usize = 1;
    const THREADS: usize = 2;
    let q = Arc::new(q);

    let mut threads = Vec::new();

    for _ in 0..THREADS / 2 {
        let q2 = q.clone();
        threads.push(thread::spawn(move || {
            for _ in 0..COUNT {
                while q2.push(&0).is_err() {}
                thread::yield_now();
                q2.pop().unwrap();
            }
        }));

        let q = q.clone();
        threads.push(thread::spawn(move || {
            for _ in 0..COUNT {
                if q.force_push(&0).is_none() {
                    q.pop().unwrap();
                }
            }
        }));
    }

    for t in threads.into_iter() {
        t.join().unwrap()
    }
}

cfg_taggedptr64! {
    mod taggedptr64 {
        use crate::{Queue, core::slots::TaggedPtr64};

        use super::*;

        #[test]
        fn linearizable_impl() {
            loom::model(|| {
                let q = Queue::with_slot::<TaggedPtr64>(4);
                linearizable(q);
            });
        }
    }
}

cfg_taggedptr128! {
    mod taggedptr128 {
        use crate::{Queue, core::slots::TaggedPtr128};

        use super::*;

        #[test]
        fn linearizable_impl() {
            loom::model(|| {
                let q = Queue::with_slot::<TaggedPtr128>(2);
                linearizable(q);
                drop(q)
            });
        }
    }
}

#[cfg(feature = "pool")]
mod pooled {
    use crate::PooledQueue;

    use super::*;

    #[test]
    fn linearizable_impl() {
        loom::model(|| {
            let q = PooledQueue::new(2);
            linearizable(q);
        })
    }
}
