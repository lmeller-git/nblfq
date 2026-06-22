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
#[cfg(all(bench_crossbeam, feature = "dynamic"))]
use crossbeam_queue::SegQueue;
#[cfg(feature = "dynamic")]
use nblf_queue::DynamicQueue;
#[cfg(feature = "dynamic")]
use nblf_queue::Growable;
#[cfg(all(feature = "dynamic", feature = "pool"))]
use nblf_queue::PooledDynamicQueue;
#[cfg(all(feature = "alloc", feature = "pool"))]
use nblf_queue::PooledQueue;
#[cfg(feature = "pool")]
use nblf_queue::PooledStaticQueue;
#[cfg(feature = "alloc")]
use nblf_queue::Queue;
use nblf_queue::{MPMCQueue, StaticQueue};

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

    #[cfg(feature = "dynamic")]
    pub use dynamic::*;

    #[cfg(feature = "dynamic")]
    mod dynamic {
        use super::*;

        pub struct SegQueueWrapper<T>(SegQueue<T>);

        impl<T> SegQueueWrapper<T> {
            pub fn new() -> Self {
                Self(SegQueue::new())
            }
        }

        impl<T> MPMCQueue for SegQueueWrapper<T> {
            type Item = T;

            fn push(&self, item: Self::Item) -> Result<(), Self::Item> {
                self.0.push(item);
                Ok(())
            }

            fn pop(&self) -> Option<Self::Item> {
                self.0.pop()
            }

            fn capacity(&self) -> usize {
                usize::MAX
            }

            fn len(&self) -> usize {
                self.0.len()
            }

            fn is_empty(&self) -> bool {
                self.0.is_empty()
            }

            fn is_full(&self) -> bool {
                false
            }
        }

        impl<T> Growable for SegQueueWrapper<T> {
            fn grow_by(&self, _by: usize) -> bool {
                true
            }
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
    });
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

#[cfg(feature = "dynamic")]
use dynamic::*;

#[cfg(feature = "dynamic")]
mod dynamic {

    use super::*;

    pub(crate) fn run_queue_mpsc_growing<Q>(q: Q, grow_step: usize)
    where
        Q: MPMCQueue<Item = &'static usize> + Sync + Growable,
    {
        assert_eq!(TOTAL_ITEMS % N_PRODUCER, 0);

        thread::scope(|scope| {
            for _ in 0..N_PRODUCER {
                scope.spawn(|| {
                    for _ in 0..ITER_PER_THREAD {
                        while q.push(black_box(&1)).is_err() {
                            _ = q.grow_by(grow_step);
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
        });
    }

    pub(crate) fn run_queue_mpmc_growing<Q>(q: Q, grow_step: usize)
    where
        Q: MPMCQueue<Item = &'static usize> + Growable + Sync,
    {
        assert_eq!(TOTAL_ITEMS % N_PRODUCER, 0);

        let is_done = AtomicU64::new(TOTAL_ITEMS);

        thread::scope(|scope| {
            for _ in 0..N_PRODUCER {
                scope.spawn(|| {
                    for _ in 0..ITER_PER_THREAD {
                        while q.push(black_box(&1)).is_err() {
                            _ = q.grow_by(grow_step);
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
}

fn bench_throughput_spsc(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput spsc");
    static ONE: u8 = 1;
    for size in [1024, 2048, 4096, 8192].iter() {
        let input: Vec<&'static _> = vec![&ONE; *size];

        group.throughput(criterion::Throughput::Elements(*size as u64));

        group.bench_with_input(format!("StaticQueue | size={size}"), &input, |b, i| {
            b.iter(|| simple_sender::<StaticQueue<_, 64>>(StaticQueue::new(), i));
        });

        #[cfg(feature = "dynamic")]
        group.bench_with_input(format!("DynamicQueue | size={size}"), &input, |b, i| {
            b.iter(|| simple_sender::<DynamicQueue<_>>(DynamicQueue::new(64), i));
        });

        #[cfg(feature = "pool")]
        group.bench_with_input(
            format!("PooledStaticQueue | size={size}"),
            &input,
            |b, i| {
                b.iter(|| simple_sender::<PooledStaticQueue<_, 64>>(PooledStaticQueue::new(), i));
            },
        );

        #[cfg(all(feature = "dynamic", feature = "pool"))]
        group.bench_with_input(
            format!("PooledDynamicQueue | size={size}"),
            &input,
            |b, i| {
                b.iter(|| simple_sender::<PooledDynamicQueue<_>>(PooledDynamicQueue::new(64), i));
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
        b.iter(|| run_queue_mpsc::<StaticQueue<_, 64>>(StaticQueue::new()));
    });

    #[cfg(feature = "dynamic")]
    group.bench_function("DynamicQueue", |b| {
        b.iter(|| run_queue_mpsc::<DynamicQueue<_>>(DynamicQueue::new(64)));
    });

    #[cfg(feature = "pool")]
    group.bench_function("PooledStaticQueue", |b| {
        b.iter(|| run_queue_mpsc::<PooledStaticQueue<_, 64>>(PooledStaticQueue::new()));
    });

    #[cfg(all(feature = "dynamic", feature = "pool"))]
    group.bench_function("PooledDynamicQueue", |b| {
        b.iter(|| run_queue_mpsc::<PooledDynamicQueue<_>>(PooledDynamicQueue::new(64)));
    });

    #[cfg(all(bench_crossbeam, feature = "alloc"))]
    group.bench_function("crossbeam_queue::ArrayQueue", |b| {
        b.iter(|| run_queue_mpsc(CrossbeamWrapper::new(64)));
    });

    group.finish();
}

fn bench_throughput_mpmc(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput mpmc");
    group.throughput(criterion::Throughput::Elements(TOTAL_ITEMS));

    group.bench_function("simple throughput static queue", |b| {
        b.iter(|| run_queue_mpmc::<StaticQueue<_, 64>>(StaticQueue::new()));
    });

    #[cfg(feature = "dynamic")]
    group.bench_function("throughput DynamicQueue", |b| {
        b.iter(|| run_queue_mpmc::<DynamicQueue<_>>(DynamicQueue::new(64)));
    });

    #[cfg(feature = "pool")]
    group.bench_function("simple throughput pooled static queue", |b| {
        b.iter(|| run_queue_mpmc::<PooledStaticQueue<_, 64>>(PooledStaticQueue::new()));
    });

    #[cfg(all(feature = "dynamic", feature = "pool"))]
    group.bench_function("throughput PooledDynamicQueue", |b| {
        b.iter(|| run_queue_mpmc::<PooledDynamicQueue<_>>(PooledDynamicQueue::new(64)));
    });

    #[cfg(all(bench_crossbeam, feature = "alloc"))]
    group.bench_function("crossbeam_queue::ArrayQueue", |b| {
        b.iter(|| run_queue_mpmc(CrossbeamWrapper::new(64)));
    });

    group.finish();
}

#[cfg(feature = "alloc")]
fn bench_throughput_mpmc_cap(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput mpmc with cap variation");

    for cap in [64, 128, 256, 512] {
        group.throughput(criterion::Throughput::Elements(TOTAL_ITEMS));

        group.bench_function(format!("Queue | cap={cap}"), |b| {
            b.iter(|| run_queue_mpmc(Queue::new(cap)));
        });

        #[cfg(feature = "dynamic")]
        group.bench_function(format!("DynamicQueue | cap={cap}"), |b| {
            b.iter(|| run_queue_mpmc::<DynamicQueue<_>>(DynamicQueue::new(cap)));
        });

        #[cfg(feature = "pool")]
        group.bench_function(format!("PooledQueue | cap={cap}"), |b| {
            b.iter(|| run_queue_mpmc(PooledQueue::new(cap)));
        });

        #[cfg(all(feature = "dynamic", feature = "pool"))]
        group.bench_function(format!("PooledDynamicQueue | cap={cap}"), |b| {
            b.iter(|| run_queue_mpmc::<PooledDynamicQueue<_>>(PooledDynamicQueue::new(cap)));
        });

        #[cfg(bench_crossbeam)]
        group.bench_function(format!("crossbeam_queue::ArrayQueue | cap={cap}"), |b| {
            b.iter(|| run_queue_mpmc(CrossbeamWrapper::new(cap)));
        });
    }
    group.finish();
}

fn bench_push_pop(c: &mut Criterion) {
    let mut group = c.benchmark_group("push pop single thread");
    group.bench_function("StaticQueue", |b| {
        b.iter(|| run_queue_single_thread::<StaticQueue<_, 2>>(StaticQueue::new()));
    });

    #[cfg(feature = "dynamic")]
    group.bench_function("DynamicQueue", |b| {
        b.iter(|| run_queue_single_thread::<DynamicQueue<_>>(DynamicQueue::new(2)));
    });

    #[cfg(feature = "pool")]
    group.bench_function("PooledStaticQueue", |b| {
        b.iter(|| run_queue_single_thread::<PooledStaticQueue<_, 2>>(PooledStaticQueue::new()));
    });

    #[cfg(all(feature = "dynamic", feature = "pool"))]
    group.bench_function("PooledDynamicQueue", |b| {
        b.iter(|| run_queue_single_thread::<PooledDynamicQueue<_>>(PooledDynamicQueue::new(2)));
    });

    #[cfg(all(bench_crossbeam, feature = "alloc"))]
    group.bench_function("crossbeam_queue::ArrayQueue", |b| {
        b.iter(|| run_queue_single_thread(CrossbeamWrapper::new(2)));
    });

    group.finish();
}

#[cfg(feature = "dynamic")]
fn bench_throughput_mpmc_growing(c: &mut Criterion) {
    let mut group = c.benchmark_group("growing mpmc with differing growth steps");

    for step in [32, 64, 256, 512] {
        group.throughput(criterion::Throughput::Elements(TOTAL_ITEMS));

        group.bench_function(format!("DynamicQueue | step={step}"), |b| {
            b.iter(|| run_queue_mpmc_growing(DynamicQueue::new(2), step));
        });

        #[cfg(feature = "pool")]
        group.bench_function(format!("PooledDynamicQueue | step={step}"), |b| {
            b.iter(|| run_queue_mpmc_growing(PooledDynamicQueue::new(2), step));
        });

        #[cfg(bench_crossbeam)]
        group.bench_function(format!("crossbeam::SegQueue | step={step}"), |b| {
            b.iter(|| run_queue_mpmc_growing(SegQueueWrapper::new(), step));
        });
    }
    group.finish();
}

#[cfg(feature = "dynamic")]
fn bench_throughput_mpsc_growing(c: &mut Criterion) {
    let mut group = c.benchmark_group("growing mpsc with differing growth steps");

    for step in [32, 64, 256, 512] {
        group.throughput(criterion::Throughput::Elements(TOTAL_ITEMS));

        group.bench_function(format!("DynamicQueue | step={step}"), |b| {
            b.iter(|| run_queue_mpsc_growing(DynamicQueue::new(2), step));
        });

        #[cfg(feature = "pool")]
        group.bench_function(format!("PooledDynamicQueue | step={step}"), |b| {
            b.iter(|| run_queue_mpsc_growing(PooledDynamicQueue::new(2), step));
        });

        #[cfg(bench_crossbeam)]
        group.bench_function(format!("crossbeam::SegQueue | step={step}"), |b| {
            b.iter(|| run_queue_mpsc_growing(SegQueueWrapper::new(), step));
        });
    }
    group.finish();
}

#[cfg(feature = "alloc")]
criterion_group!(benches_alloc, bench_throughput_mpmc_cap);

#[cfg(feature = "dynamic")]
criterion_group!(
    benches_growth,
    bench_throughput_mpsc_growing,
    bench_throughput_mpmc_growing
);

criterion_group!(
    benches_base,
    bench_push_pop,
    bench_throughput_spsc,
    bench_throughput_mpsc,
    bench_throughput_mpmc,
);

#[cfg(not(feature = "alloc"))]
criterion_main!(benches_base);

#[cfg(all(feature = "alloc", not(feature = "dynamic")))]
criterion_main!(benches_base, benches_alloc);

#[cfg(feature = "dynamic")]
criterion_main!(benches_base, benches_alloc, benches_growth);
