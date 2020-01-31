use crate::future::poll_fn::poll_fn;
//use tokio::sync::semaphore_ll::{AcquireError, Permit, Semaphore};
use std::sync::atomic::{AtomicUsize, AtomicPtr};
use tokio::sync::Semaphore;
use std::cell::UnsafeCell;
use std::ops;
use std::task::{Context, Poll};

#[cfg(not(loom))]
const MAX_READS: usize = 32;

#[cfg(loom)]
const MAX_READS: usize = 10;

/// An asynchronous reader-writer lock
///
/// This type of lock allows a number of readers or at most one writer at any
/// point in time. The write portion of this lock typically allows modification
/// of the underlying data (exclusive access) and the read portion of this lock
/// typically allows for read-only access (shared access).
///
/// In comparison, a [`Mutex`] does not distinguish between readers or writers
/// that acquire the lock, therefore blocking any tasks waiting for the lock to
/// become available. An `SharedLock` will allow any number of readers to acquire the
/// lock as long as a writer is not holding the lock.
///
/// The priority policy of the lock is dependent on the underlying operating
/// system's implementation, and this type does not guarantee that any
/// particular policy will be used.
///
/// The type parameter `T` represents the data that this lock protects. It is
/// required that `T` satisfies [`Send`] to be shared across threads. The RAII guards
/// returned from the locking methods implement [`Deref`](https://doc.rust-lang.org/std/ops/trait.Deref.html)
/// (and [`DerefMut`](https://doc.rust-lang.org/std/ops/trait.DerefMut.html)
/// for the `write` methods) to allow access to the content of the lock.
///
/// # Examples
///
/// ```
/// use tokio::sync::SharedLock;
///
/// #[tokio::main]
/// async fn main() {
///     let lock = SharedLock::new(5);
///
/// // many reader locks can be held at once
///     {
///         let r1 = lock.read().await;
///         let r2 = lock.read().await;
///         assert_eq!(*r1, 5);
///         assert_eq!(*r2, 5);
///     } // read locks are dropped at this point
///
/// // only one write lock may be held, however
///     {
///         let mut w = lock.write().await;
///         *w += 1;
///         assert_eq!(*w, 6);
///     } // write lock is dropped here
/// }
/// ```
///
/// [`Mutex`]: struct.Mutex.html
/// [`SharedLock`]: struct.SharedLock.html
/// [`SharedLockSharedGuard`]: struct.SharedLockSharedGuard.html
/// [`SharedLockExclusiveGuard`]: struct.SharedLockExclusiveGuard.html
/// [`Send`]: https://doc.rust-lang.org/std/marker/trait.Send.html
#[derive(Debug)]
pub struct SharedLock<T> {
    //semaphore to coordinate read and write access to T
    s: Semaphore,

    //inner data T
    c: UnsafeCell<T>,
}

/// RAII structure used to release the shared read access of a lock when
/// dropped.
///
/// This structure is created by the [`read`] method on
/// [`SharedLock`].
///
/// [`read`]: struct.SharedLock.html#method.read
#[derive(Debug)]
pub struct SharedLockSharedGuard<'a, T> {
    permit: ReleasingPermit<'a, T>,
    lock: &'a SharedLock<T>,
}

/// RAII structure used to release the exclusive write access of a lock when
/// dropped.
///
/// This structure is created by the [`write`] and method
/// on [`SharedLock`].
///
/// [`write`]: struct.SharedLock.html#method.write
/// [`SharedLock`]: struct.SharedLock.html
#[derive(Debug)]
pub struct SharedLockExclusiveGuard<'a, T> {
    permit: ReleasingPermit<'a, T>,
    lock: &'a SharedLock<T>,
}

/// A semaphore permit
///
/// Tracks the lifecycle of a semaphore permit.
///
/// An instance of `Permit` is intended to be used with a **single** instance of
/// `Semaphore`. Using a single instance of `Permit` with multiple semaphore
/// instances will result in unexpected behavior.
///
/// `Permit` does **not** release the permit back to the semaphore on drop. It
/// is the user's responsibility to ensure that `Permit::release` is called
/// before dropping the permit.
#[derive(Debug)]
pub(crate) struct Permit {
    waiter: Option<Box<Waiter>>,
    state: PermitState,
}

/// Error returned by `Permit::poll_acquire`.
#[derive(Debug)]
pub(crate) struct AcquireError(());

/// Error returned by `Permit::try_acquire`.
#[derive(Debug)]
pub(crate) enum TryAcquireError {
    Closed,
    NoPermits,
}

/// Node used to notify the semaphore waiter when permit is available.
#[derive(Debug)]
struct Waiter {
    /// Stores waiter state.
    ///
    /// See `WaiterState` for more details.
    state: AtomicUsize,

    /// Task to wake when a permit is made available.
    //waker: AtomicWaker,

    /// Next pointer in the queue of waiting senders.
    next: AtomicPtr<Waiter>,
}

/// Semaphore state
///
/// The 2 low bits track the modes.
///
/// - Closed
/// - Full
///
/// When not full, the rest of the `usize` tracks the total number of messages
/// in the channel. When full, the rest of the `usize` is a pointer to the tail
/// of the "waiting senders" queue.
#[derive(Copy, Clone)]
struct SemState(usize);

/// Permit state
#[derive(Debug, Copy, Clone)]
enum PermitState {
    /// Currently waiting for permits to be made available and assigned to the
    /// waiter.
    Waiting(u16),

    /// The number of acquired permits
    Acquired(u16),
}

// Wrapper arround Permit that releases on Drop
#[derive(Debug)]
struct ReleasingPermit<'a, T> {
    num_permits: u16,
    permit: Permit,
    lock: &'a SharedLock<T>,
}

impl<'a, T> ReleasingPermit<'a, T> {
    fn poll_acquire(
        &mut self,
        cx: &mut Context<'_>,
        s: &Semaphore,
    ) -> Poll<Result<(), AcquireError>> {
        self.permit.poll_acquire(cx, self.num_permits, s)
    }
}

impl<'a, T> Drop for ReleasingPermit<'a, T> {
    fn drop(&mut self) {
        self.permit.release(self.num_permits, &self.lock.s);
    }
}

// As long as T: Send + Sync, it's fine to send and share SharedLock<T> between threads.
// If T were not Send, sending and sharing a SharedLock<T> would be bad, since you can access T through
// SharedLock<T>.
unsafe impl<T> Send for SharedLock<T> where T: Send {}
unsafe impl<T> Sync for SharedLock<T> where T: Send + Sync {}
unsafe impl<'a, T> Sync for SharedLockSharedGuard<'a, T> where T: Send + Sync {}
unsafe impl<'a, T> Sync for SharedLockExclusiveGuard<'a, T> where T: Send + Sync {}

impl<T> SharedLock<T> {
    /// Creates a new instance of an `SharedLock<T>` which is unlocked.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokio::sync::SharedLock;
    ///
    /// let lock = SharedLock::new(5);
    /// ```
    pub fn new(value: T) -> SharedLock<T> {
        SharedLock {
            c: UnsafeCell::new(value),
            s: Semaphore::new(MAX_READS),
        }
    }

    /// Locks this rwlock with shared read access, blocking the current task
    /// until it can be acquired.
    ///
    /// The calling task will be blocked until there are no more writers which
    /// hold the lock. There may be other readers currently inside the lock when
    /// this method returns.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    /// use tokio::sync::SharedLock;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let lock = Arc::new(SharedLock::new(1));
    ///     let c_lock = lock.clone();
    ///
    ///     let n = lock.read().await;
    ///     assert_eq!(*n, 1);
    ///
    ///     tokio::spawn(async move {
    ///         let r = c_lock.read().await;
    ///         assert_eq!(*r, 1);
    ///     });
    ///}
    /// ```
    pub async fn read(&self) -> SharedLockSharedGuard<'_, T> {
        let mut permit = ReleasingPermit {
            num_permits: 1,
            permit: Permit::new(),
            lock: self,
        };

        poll_fn(|cx| permit.poll_acquire(cx, &self.s))
            .await
            .unwrap_or_else(|_| {
                // The semaphore was closed. but, we never explicitly close it, and we have a
                // handle to it through the Arc, which means that this can never happen.
                unreachable!()
            });
        SharedLockSharedGuard { lock: self, permit }
    }

    /// Locks this rwlock with exclusive write access, blocking the current
    /// task until it can be acquired.
    ///
    /// This function will not return while other writers or other readers
    /// currently have access to the lock.
    ///
    /// Returns an RAII guard which will drop the write access of this rwlock
    /// when dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokio::sync::SharedLock;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///   let lock = SharedLock::new(1);
    ///
    ///   let mut n = lock.write().await;
    ///   *n = 2;
    ///}
    /// ```
    pub async fn write(&self) -> SharedLockSharedGuard<'_, T> {
        let mut permit = ReleasingPermit {
            num_permits: MAX_READS as u16,
            permit: Permit::new(),
            lock: self,
        };

        poll_fn(|cx| permit.poll_acquire(cx, &self.s))
            .await
            .unwrap_or_else(|_| {
                // The semaphore was closed. but, we never explicitly close it, and we have a
                // handle to it through the Arc, which means that this can never happen.
                unreachable!()
            });

        SharedLockExclusiveGuard { lock: self, permit }
    }

    pub async fn exclusiveWrite(&self) -> SharedLockExclusiveGuard<'_, T> {
        let mut permit = ReleasingPermit {
            num_permits: MAX_READS as u16,
            permit: Permit::new(),
            lock: self,
        };

        poll_fn(|cx| permit.poll_acquire(cx, &self.s))
            .await
            .unwrap_or_else(|_| {
                // The semaphore was closed. but, we never explicitly close it, and we have a
                // handle to it through the Arc, which means that this can never happen.
                unreachable!()
            });

        SharedLockExclusiveGuard { lock: self, permit }
    }
}

impl<T> ops::Deref for SharedLockSharedGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.lock.c.get() }
    }
}

impl<T> ops::DerefMut for SharedLockSharedGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.c.get() }
    }
}

impl<T> ops::Deref for SharedLockExclusiveGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.lock.c.get() }
    }
}

impl<T> ops::DerefMut for SharedLockExclusiveGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.c.get() }
    }
}
