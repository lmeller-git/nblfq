use nblf_queue::{MPMCQueue, PooledDynamicQueue, PooledQueue, Resize};
use pyo3::prelude::*;

const MAX_SPINLOOP: usize = 1024;

pub(crate) struct Backoff {
    state: usize,
}

impl Backoff {
    pub(crate) fn new() -> Self {
        Self { state: 1 }
    }

    pub(crate) fn backoff(&mut self) {
        {
            for _ in 0..self.state {
                core::hint::spin_loop();
            }
            self.state = (self.state * 2).min(MAX_SPINLOOP);
        }
    }
}

#[derive(Debug)]
struct PythonItem(Py<PyAny>);

impl Clone for PythonItem {
    fn clone(&self) -> Self {
        Python::attach(|py| Self(self.0.clone_ref(py)))
    }
}

/// A dynamically growable, lock-free non-blocking MPMC queue.
///
/// Core operations detach from the python GIL, to ensure concurrent performance.
#[pyclass]
pub struct DynamicQueue(PooledDynamicQueue<PythonItem>);

#[pymethods]
impl DynamicQueue {
    /// Constructs a new `DynamicQueue` with initial capacity `size`.
    #[new]
    #[pyo3(signature = (size))]
    pub fn new(size: usize) -> Self {
        Self(PooledDynamicQueue::new(size))
    }

    /// Attempts to push an element into the queue.
    ///
    /// Returns the item, if the queue was full.
    pub fn push(&self, item: Py<PyAny>) -> Option<Py<PyAny>> {
        self.0
            .push(PythonItem(item))
            .map_or_else(|item| Some(item.0), |_| None)
    }

    /// Attempts to pop an item from the queue.
    ///
    /// Returns `None` if the queue was empty.
    pub fn pop(&self) -> Option<Py<PyAny>> {
        self.0.pop().map(|item| item.0)
    }

    /// Returns the current length of the queue.
    ///
    /// This method should not be used for synchronization.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns the current capacity of the queue.
    ///
    /// This method should not be used for synchronization.
    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    /// Indicates wether the queue is currently empty.
    ///
    /// This method should not be used for synchronization.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Indicates wether the queue is currently full.
    ///
    /// This method should not be used for synchronization.
    pub fn is_full(&self) -> bool {
        self.0.is_full()
    }

    /// Pushes an item into the queue.
    /// This method may take some time under heavy contention.
    /// This method may pop an arbitrary amount of items from the queue.
    ///
    /// Returns the last popped item, if the queue was full. All other items are dropped.
    pub fn force_push(&self, item: Py<PyAny>) -> Option<Py<PyAny>> {
        self.0.force_push(PythonItem(item)).map(|item| item.0)
    }

    /// Pushes an item into the queue.
    /// This method may take some time under heavy contention.
    /// This method may pop an arbitrary amount of items from the queue.
    /// Applies `f` to each popped item.
    pub fn force_push_and_do(
        &self,
        py: Python<'_>,
        mut item: Py<PyAny>,
        f: Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let mut backoff = Backoff::new();
        while let Some(item_) = self.push(item) {
            item = item_;

            backoff.backoff();
            let popped = self.pop();

            if let Some(next_popped_item) = popped {
                f.call1((next_popped_item.bind(py),))?;
            }
        }

        Ok(())
    }

    /// Attempts to resize the queues capacity to `size` items.
    ///
    /// This method may spuriously fail.
    ///
    /// Returns `true` if the capacity was succesfully grown.
    pub fn resize(&self, size: usize) -> bool {
        self.0.resize(size)
    }

    /// Attempts to grow the queues capacity using limited exponeintial growth..
    ///
    /// This method may spuriously fail.
    ///
    /// Returns `true` if the capacity was succesfully grown.
    pub fn grow(&self) -> bool {
        let cap = self.capacity();
        let next_step = cap.min(1024);
        self.resize(next_step + cap)
    }
}

/// A lock-free non-blocking MPMC queue.
///
/// Core operations detach from the python GIL, to ensure concurrent performance.
#[pyclass]
pub struct Queue(PooledQueue<PythonItem>);

#[pymethods]
impl Queue {
    /// Constructs a new `Queue` with initial capacity `size`.
    #[new]
    #[pyo3(signature = (size))]
    pub fn new(size: usize) -> Self {
        Self(PooledQueue::new(size))
    }

    /// Attempts to push an element into the queue.
    ///
    /// Returns the item, if the queue was full.
    pub fn push(&self, item: Py<PyAny>) -> Option<Py<PyAny>> {
        self.0
            .push(PythonItem(item))
            .map_or_else(|item| Some(item.0), |_| None)
    }

    /// Attempts to pop an item from the queue.
    ///
    /// Returns `None` if the queue was empty.
    pub fn pop(&self) -> Option<Py<PyAny>> {
        self.0.pop().map(|item| item.0)
    }

    /// Returns the current length of the queue.
    ///
    /// This method should not be used for synchronization.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns the current capacity of the queue.
    ///
    /// This method should not be used for synchronization.
    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    /// Indicates wether the queue is currently empty.
    ///
    /// This method should not be used for synchronization.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Indicates wether the queue is currently full.
    ///
    /// This method should not be used for synchronization.
    pub fn is_full(&self) -> bool {
        self.0.is_full()
    }

    /// Pushes an item into the queue.
    /// This method may take some time under heavy contention.
    /// This method may pop an arbitrary amount of items from the queue.
    ///
    /// Returns the last popped item, if the queue was full. All other items are dropped.
    pub fn force_push(&self, item: Py<PyAny>) -> Option<Py<PyAny>> {
        self.0.force_push(PythonItem(item)).map(|item| item.0)
    }

    /// Pushes an item into the queue.
    /// This method may take some time under heavy contention.
    /// This method may pop an arbitrary amount of items from the queue.
    /// Applies `f` to each popped item.
    pub fn force_push_and_do(
        &self,
        py: Python<'_>,
        mut item: Py<PyAny>,
        f: Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let mut backoff = Backoff::new();
        while let Some(item_) = self.push(item) {
            item = item_;

            backoff.backoff();
            let popped = self.pop();

            if let Some(next_popped_item) = popped {
                f.call1((next_popped_item.bind(py),))?;
            }
        }

        Ok(())
    }
}

#[pymodule(gil_used = false)]
#[pyo3(name = "_nblf_queue_py")]
fn nblf_queue_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Queue>()?;
    m.add_class::<DynamicQueue>()?;
    Ok(())
}
