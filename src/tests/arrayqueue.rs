use core::sync::atomic::{AtomicUsize, Ordering};
use std::{thread::scope, vec::Vec};

use crate::HeapBackedQueue;

#[test]
fn smoke() {
    let q = HeapBackedQueue::new(1);
    q.push(7).unwrap();
    assert_eq!(q.pop(), Some(7));

    q.push(8).unwrap();
    assert_eq!(q.pop(), Some(8));
    assert!(q.pop().is_none());
}

#[test]
fn smoke_long() {
    let q = HeapBackedQueue::new(10);
    q.push(7).unwrap();
    assert_eq!(q.pop(), Some(7));

    q.push(8).unwrap();
    q.push(9).unwrap();
    assert_eq!(q.pop(), Some(8));
    assert_eq!(q.pop(), Some(9));
    assert!(q.pop().is_none());
}

#[test]
fn capacity() {
    todo!()
}

#[test]
fn len_empty_full() {
    todo!()
}

#[test]
fn len() {
    todo!()
}

#[test]
fn spsc() {
    const COUNT: usize = 100_000;

    let q = HeapBackedQueue::new(3);

    scope(|scope| {
        scope.spawn(|| {
            for i in 0..COUNT {
                loop {
                    if let Some(x) = q.pop() {
                        assert_eq!(x, i);
                        break;
                    }
                }
            }
            assert!(q.pop().is_none());
        });

        scope.spawn(|| {
            for i in 0..COUNT {
                while q.push(i).is_err() {}
            }
        });
    })
}

#[test]
fn mpsc() {
    const COUNT: usize = 10_000;
    const THREADS: usize = 4;

    let q: HeapBackedQueue<usize> = HeapBackedQueue::new(3);
    let v = (0..COUNT).map(|_| AtomicUsize::new(0)).collect::<Vec<_>>();

    scope(|scope| {
        for _ in 0..THREADS {
            scope.spawn(|| {
                for i in 0..COUNT {
                    while q.push(i).is_err() {}
                }
            });
        }
        for _ in 0..THREADS {
            for _ in 0..COUNT {
                let n = loop {
                    if let Some(x) = q.pop() {
                        break x;
                    }
                };
                v[n].fetch_add(1, Ordering::SeqCst);
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
    let q: HeapBackedQueue<usize> = HeapBackedQueue::new(3);
    let v = (0..COUNT).map(|_| AtomicUsize::new(0)).collect::<Vec<_>>();

    scope(|scope| {
        for _ in 0..THREADS {
            scope.spawn(|| {
                for _ in 0..COUNT {
                    let n = loop {
                        if let Some(x) = q.pop() {
                            break x;
                        }
                    };
                    v[n].fetch_add(1, Ordering::SeqCst);
                }
            });
        }
        for _ in 0..THREADS {
            scope.spawn(|| {
                for i in 0..COUNT {
                    while q.push(i).is_err() {}
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

    let q = HeapBackedQueue::new(THREADS);

    scope(|scope| {
        for _ in 0..THREADS / 2 {
            scope.spawn(|| {
                for _ in 0..COUNT {
                    while q.push(0).is_err() {}
                    q.pop().unwrap();
                }
            });

            scope.spawn(|| {
                for _ in 0..COUNT {
                    if q.force_push(0).is_none() {
                        q.pop().unwrap();
                    }
                }
            });
        }
    })
}

#[test]
fn into_iter() {
    todo!()
}
