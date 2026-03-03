#![allow(dead_code, unused_imports)]

use std::{
    hint::{black_box, spin_loop},
    sync::atomic::{AtomicU64, Ordering},
    thread,
};

use nblf_queue::{MPMCQueue, PooledQueue, Queue};

const TOTAL_ITEMS: u64 = 300_000;
const THREADS: u64 = 3;
const ITEMS_PER_THREAD: u64 = TOTAL_ITEMS / THREADS;
const CAP: usize = 64;

fn main() {
    let q = Queue::new(CAP);
    run_mpmc(q);
}

fn run_spsc<Q>(q: Q)
where
    Q: MPMCQueue<Item = &'static i32> + Sync,
{
    static ONE: i32 = 1;
    assert_eq!(TOTAL_ITEMS % THREADS, 0);

    thread::scope(|scope| {
        scope.spawn(|| {
            for _ in 0..TOTAL_ITEMS {
                while q.push(black_box(&ONE)).is_err() {
                    spin_loop();
                }
            }
        });

        scope.spawn(|| {
            for _ in 0..TOTAL_ITEMS {
                while black_box(q.pop()).is_none() {
                    spin_loop();
                }
            }
        });
    })
}

fn run_mpsc<Q>(q: Q)
where
    Q: MPMCQueue<Item = &'static i32> + Sync,
{
    static ONE: i32 = 1;
    assert_eq!(TOTAL_ITEMS % THREADS, 0);

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

        scope.spawn(|| {
            for _ in 0..TOTAL_ITEMS {
                while black_box(q.pop()).is_none() {
                    spin_loop();
                }
            }
        });
    })
}

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
                    spin_loop();
                }
            });
        }
    })
}
