use std::{
    hint::{black_box, spin_loop},
    sync::atomic::{AtomicU64, Ordering},
    thread,
};

use criterion::{Criterion, criterion_group, criterion_main};
#[cfg(all(bench_crossbeam, feature = "alloc"))]
use crossbeam_::*;
#[cfg(all(bench_crossbeam, feature = "alloc"))]
use crossbeam_queue::ArrayQueue;
use nblf_queue::{MPMCQueue, PooledStaticQueue, StaticQueue};
#[cfg(feature = "alloc")]
use nblf_queue::{PooledQueue, Queue};

#[cfg(all(bench_crossbeam, feature = "alloc"))]
mod crossbeam_ {
    use super::*;

    pub struct CrossbeamWrapper<T>(ArrayQueue<T>);

    impl<T> CrossbeamWrapper<T> {
        pub fn new(size: usize) -> Self {
            Self(ArrayQueue::new(size))
        }
    }

    impl<T> MPMCQueue for CrossbeamWrapper<T> {
        type Item = T;

        fn push(&self, item: Self::Item) -> Result<(), Self::Item> {
            self.0.push(item)
        }

        fn pop(&self) -> Option<Self::Item> {
            self.0.pop()
        }

        fn len(&self) -> usize {
            self.0.len()
        }

        fn capacity(&self) -> usize {
            self.0.capacity()
        }
    }
}

const TOTAL_ITEMS: u64 = 100_000;
const N_PRODUCER: u64 = 2;
const ITER_PER_THREAD: u64 = TOTAL_ITEMS / N_PRODUCER;

fn run_queue_single_thread<Q>(q: Q)
where
    Q: MPMCQueue<Item = &'static i32>,
{
    for _ in 0..TOTAL_ITEMS {
        q.push(black_box(&0)).unwrap();
        black_box(q.pop()).unwrap();
    }
}

fn run_queue_mpsc<Q>(q: Q)
where
    Q: MPMCQueue<Item = &'static usize> + Sync,
{
    assert_eq!(TOTAL_ITEMS % N_PRODUCER, 0);

    thread::scope(|scope| {
        for _ in 0..N_PRODUCER {
            scope.spawn(|| {
                for _ in 0..ITER_PER_THREAD {
                    while q.push(black_box(&1)).is_err() {
                        spin_loop();
                    }
                }
            });
        }

        for _ in 0..TOTAL_ITEMS {
            loop {
                if let Some(item) = q.pop() {
                    black_box(item);
                    break;
                }
                spin_loop();
            }
        }
    })
}

fn run_queue_mpmc<Q>(q: Q)
where
    Q: MPMCQueue<Item = &'static usize> + Sync,
{
    assert_eq!(TOTAL_ITEMS % N_PRODUCER, 0);

    let is_done = AtomicU64::new(TOTAL_ITEMS);

    thread::scope(|scope| {
        for _ in 0..N_PRODUCER {
            scope.spawn(|| {
                for _ in 0..ITER_PER_THREAD {
                    while q.push(black_box(&1)).is_err() {
                        spin_loop();
                    }
                }
            });
        }

        for _ in 0..N_PRODUCER {
            scope.spawn(|| {
                loop {
                    if is_done.load(Ordering::Acquire) == 0 {
                        break;
                    }
                    if let Some(item) = q.pop() {
                        black_box(item);
                        is_done.fetch_sub(1, Ordering::Release);
                    }
                    spin_loop();
                }
            });
        }
    });
}

fn simple_sender<Q>(q: Q, values: &[&'static u8])
where
    Q: MPMCQueue<Item = &'static u8> + Sync,
{
    thread::scope(|scope| {
        scope.spawn(|| {
            for v in values.iter() {
                while q.push(v).is_err() {}
            }
        });

        scope.spawn(|| {
            for _ in 0..values.len() {
                while q.pop().is_none() {}
            }
        });
    });
}

fn bench_throughput_spsc(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput spsc");
    static ONE: u8 = 1;
    for size in [1024, 2048, 4096, 8192].iter() {
        let input: Vec<&'static _> = vec![&ONE; *size];

        group.throughput(criterion::Throughput::Elements(*size as u64));

        group.bench_with_input(format!("StaticQueue | size={size}"), &input, |b, i| {
            b.iter(|| simple_sender::<StaticQueue<_, 64>>(StaticQueue::new(), i))
        });
        group.bench_with_input(
            format!("PooledStaticQueue | size={size}"),
            &input,
            |b, i| {
                b.iter(|| simple_sender::<PooledStaticQueue<_, 64>>(PooledStaticQueue::new(), i))
            },
        );
        #[cfg(all(bench_crossbeam, feature = "alloc"))]
        group.bench_with_input(
            format!("crossbeam_queue::ArrayQueue | size={size}"),
            &input,
            |b, i| b.iter(|| simple_sender(CrossbeamWrapper::new(64), i)),
        );
    }

    group.finish();
}

fn bench_throughput_mpsc(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput mpsc");
    group.throughput(criterion::Throughput::Elements(TOTAL_ITEMS));

    group.bench_function("StaticQueue", |b| {
        b.iter(|| run_queue_mpsc::<StaticQueue<_, 64>>(StaticQueue::new()))
    });
    group.bench_function("PooledStaticQueue", |b| {
        b.iter(|| run_queue_mpsc::<PooledStaticQueue<_, 64>>(PooledStaticQueue::new()))
    });
    #[cfg(all(bench_crossbeam, feature = "alloc"))]
    group.bench_function("crossbeam_queue::ArrayQueue", |b| {
        b.iter(|| run_queue_mpsc(CrossbeamWrapper::new(64)))
    });
    group.finish();
}

fn bench_throughput_mpmc(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput mpmc");
    group.throughput(criterion::Throughput::Elements(TOTAL_ITEMS));

    group.bench_function("simple throughput static queue", |b| {
        b.iter(|| run_queue_mpmc::<StaticQueue<_, 64>>(StaticQueue::new()))
    });
    group.bench_function("simple throughput pooled static queue", |b| {
        b.iter(|| run_queue_mpmc::<PooledStaticQueue<_, 64>>(PooledStaticQueue::new()))
    });
    #[cfg(all(bench_crossbeam, feature = "alloc"))]
    group.bench_function("crossbeam_queue::ArrayQueue", |b| {
        b.iter(|| run_queue_mpmc(CrossbeamWrapper::new(64)))
    });

    group.finish();
}

#[cfg(feature = "alloc")]
fn bench_throughput_mpmc_cap(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput mpmc with cap variation");

    for cap in [64, 128, 256, 512] {
        group.throughput(criterion::Throughput::Elements(TOTAL_ITEMS));

        group.bench_function(format!("Queue | cap={cap}"), |b| {
            b.iter(|| run_queue_mpmc(Queue::new(cap)))
        });
        group.bench_function(format!("PooledQueue | cap={cap}"), |b| {
            b.iter(|| run_queue_mpmc(PooledQueue::new(cap)))
        });
        #[cfg(all(bench_crossbeam, feature = "alloc"))]
        group.bench_function(format!("crossbeam_queue::ArrayQueue | cap={cap}"), |b| {
            b.iter(|| run_queue_mpmc(CrossbeamWrapper::new(cap)))
        });
    }
    group.finish();
}

fn bench_push_pop(c: &mut Criterion) {
    let mut group = c.benchmark_group("push pop single thread");
    group.bench_function("StaticQueue", |b| {
        b.iter(|| run_queue_single_thread::<StaticQueue<_, 2>>(StaticQueue::new()))
    });
    group.bench_function("PooledStaticQueue", |b| {
        b.iter(|| run_queue_single_thread::<PooledStaticQueue<_, 2>>(PooledStaticQueue::new()))
    });
    #[cfg(all(bench_crossbeam, feature = "alloc"))]
    group.bench_function("crossbeam_queue::ArrayQueue", |b| {
        b.iter(|| run_queue_single_thread(CrossbeamWrapper::new(2)))
    });

    group.finish();
}

#[cfg(feature = "alloc")]
criterion_group!(benches_alloc, bench_throughput_mpmc_cap);

criterion_group!(
    benches_base,
    bench_push_pop,
    bench_throughput_spsc,
    bench_throughput_mpsc,
    bench_throughput_mpmc,
);

#[cfg(not(feature = "alloc"))]
criterion_main!(benches_base);

#[cfg(feature = "alloc")]
criterion_main!(benches_base, benches_alloc);
