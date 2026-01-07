use std::{
    hint::{black_box, spin_loop},
    sync::atomic::{AtomicU64, Ordering},
    thread,
};

use nblf_queue::{MPMCQueue, PooledQueue, Queue};

const TOTAL_ITEMS: u64 = 30_000;
const THREADS: u64 = 3;
const ITEMS_PER_THREAD: u64 = TOTAL_ITEMS / THREADS;

fn main() {
    let q = PooledQueue::new(64);
    run_mpmc(q);
}

// fn run_spsc() {}

// fn run_mpsc() {}

fn run_mpmc<Q>(q: Q)
where
    Q: MPMCQueue<Item = &'static i32> + Sync,
{
    static ONE: i32 = 1;
    assert_eq!(TOTAL_ITEMS % THREADS, 0);

    let counter = AtomicU64::new(TOTAL_ITEMS);

    thread::scope(|scope| {
        for _ in 0..THREADS {
            scope.spawn(|| {
                for _ in 0..ITEMS_PER_THREAD {
                    while q.push(black_box(&ONE)).is_err() {
                        spin_loop();
                    }
                }
            });
        }

        for _ in 0..THREADS {
            scope.spawn(|| {
                loop {
                    if counter.load(Ordering::Acquire) == 0 {
                        break;
                    }
                    if let Some(item) = q.pop() {
                        black_box(item);
                        counter.fetch_sub(1, Ordering::Release);
                    }
                }
            });
        }
    })
}
