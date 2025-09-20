use core::sync::atomic::{AtomicUsize, Ordering};
use std::{boxed::Box, thread::scope, vec::Vec};

use crate::HeaplessQueue;

#[test]
fn smoke() {
    let q: HeaplessQueue<1, i32> = HeaplessQueue::new();
    q.push_ref(&7).unwrap();
    assert_eq!(q.pop_ref(), Some(&7));
    q.push_ref(&8).unwrap();
    assert_eq!(q.pop_ref(), Some(&8));
    assert!(q.pop_ref().is_none());
}

#[test]
fn smoke_long() {
    let q: HeaplessQueue<10, i32> = HeaplessQueue::new();
    q.push_ref(&7).unwrap();
    assert_eq!(q.pop_ref(), Some(&7));
    q.push_ref(&8).unwrap();
    q.push_ref(&9).unwrap();
    assert_eq!(q.pop_ref(), Some(&8));
    assert_eq!(q.pop_ref(), Some(&9));
    assert!(q.pop_ref().is_none());
}

#[test]
fn len_empty_full() {
    let q: HeaplessQueue<2, _> = HeaplessQueue::new();

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

#[test]
fn len() {
    const COUNT: usize = 25_000;
    const CAP: usize = 1000;
    const ITERS: usize = CAP / 20;

    let q: HeaplessQueue<CAP, _> = HeaplessQueue::new();
    assert_eq!(q.len(), 0);
    assert_eq!(q.capacity(), CAP);

    for _ in 0..CAP / 10 {
        for i in 0..ITERS {
            q.push(i).unwrap();
            assert_eq!(q.len(), i + 1);
        }

        for i in 0..ITERS {
            q.pop().unwrap();
            assert_eq!(q.len(), ITERS - i - 1);
        }
    }
    assert_eq!(q.len(), 0);

    for i in 0..CAP {
        q.push(i).unwrap();
        assert_eq!(q.len(), i + 1);
    }

    for _ in 0..CAP {
        q.pop().unwrap();
    }
    assert_eq!(q.len(), 0);

    scope(|scope| {
        scope.spawn(|| {
            for i in 0..COUNT {
                loop {
                    if let Some(x) = q.pop() {
                        assert_eq!(x, i);
                        break;
                    }
                }
                let len = q.len();
                assert!(len <= CAP);
            }
        });

        scope.spawn(|| {
            for i in 0..COUNT {
                while q.push(i).is_err() {}
                let len = q.len();
                assert!(len <= CAP);
            }
        });
    });
    assert_eq!(q.len(), 0);
}

#[test]
fn spsc() {
    const COUNT: usize = 100_000;

    let q: HeaplessQueue<3, _> = HeaplessQueue::new();

    scope(|scope| {
        scope.spawn(|| {
            for i in 0..COUNT {
                loop {
                    if let Some(x) = q.pop_ref() {
                        assert_eq!(*x, i);
                        break;
                    }
                }
            }
            assert!(q.pop_ref().is_none());
        });

        scope.spawn(|| {
            for i in 0..COUNT {
                let i: &'static usize = Box::leak(Box::new(i));
                while q.push_ref(i).is_err() {}
            }
        });
    })
}

#[test]
fn mpsc() {
    const COUNT: usize = 10_000;
    const THREADS: usize = 4;

    let q: HeaplessQueue<3, usize> = HeaplessQueue::new();
    let v = (0..COUNT).map(|_| AtomicUsize::new(0)).collect::<Vec<_>>();

    scope(|scope| {
        for _ in 0..THREADS {
            scope.spawn(|| {
                for i in 0..COUNT {
                    let i: &'static usize = Box::leak(Box::new(i));
                    while q.push_ref(i).is_err() {}
                }
            });
        }
        for _ in 0..THREADS {
            for _ in 0..COUNT {
                let n = loop {
                    if let Some(x) = q.pop_ref() {
                        break x;
                    }
                };
                v[*n].fetch_add(1, Ordering::SeqCst);
            }
        }
    });

    for c in v {
        assert_eq!(c.load(Ordering::SeqCst), THREADS);
    }
}

#[test]
fn mpmc() {
    const COUNT: usize = 25_000;
    const THREADS: usize = 4;

    let q: HeaplessQueue<3, usize> = HeaplessQueue::new();
    let v = (0..COUNT).map(|_| AtomicUsize::new(0)).collect::<Vec<_>>();

    scope(|scope| {
        for _ in 0..THREADS {
            scope.spawn(|| {
                for _ in 0..COUNT {
                    let n = loop {
                        if let Some(x) = q.pop_ref() {
                            break x;
                        }
                    };
                    v[*n].fetch_add(1, Ordering::SeqCst);
                }
            });
        }
        for _ in 0..THREADS {
            scope.spawn(|| {
                for i in 0..COUNT {
                    let i: &'static usize = Box::leak(Box::new(i));
                    while q.push_ref(i).is_err() {}
                }
            });
        }
    });

    for c in v {
        assert_eq!(c.load(Ordering::SeqCst), THREADS);
    }
}

#[test]
fn linearizable() {
    const COUNT: usize = 25_000;
    const THREADS: usize = 4;

    let q: HeaplessQueue<THREADS, _> = HeaplessQueue::new();

    scope(|scope| {
        for _ in 0..THREADS / 2 {
            scope.spawn(|| {
                for _ in 0..COUNT {
                    while q.push_ref(&0).is_err() {}
                    q.pop_ref().unwrap();
                }
            });

            scope.spawn(|| {
                for _ in 0..COUNT {
                    if q.force_push_ref(&0).is_none() {
                        q.pop_ref().unwrap();
                    }
                }
            });
        }
    })
}
