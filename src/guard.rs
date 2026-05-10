use crate::FileLock;
use std::fmt;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// A guard that holds a file lock and automatically releases it when dropped.
///
/// This guard is returned by `FileLock::lock()` and ensures that the lock is
/// properly released when it goes out of scope.
#[must_use = "this guard holds a file lock; if not used, the lock will be immediately released"]
pub struct FileLockGuard<'a, T: ?Sized + 'a> {
    lock: &'a mut FileLock<T>,
    unlocked: bool,
    _no_send: PhantomData<*mut ()>,
}

impl<'a, T: ?Sized + 'a> FileLockGuard<'a, T> {
    pub fn new(lock: &'a mut FileLock<T>) -> Self {
        FileLockGuard {
            lock,
            unlocked: false,
            _no_send: PhantomData,
        }
    }

    /// Manually unlocks the file lock.
    ///
    /// This method consumes the guard and returns a result indicating whether
    /// the unlock operation was successful.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use filelock::FileLock;
    ///
    /// let mut lock = FileLock::new("myfile.lock", ());
    /// let guard = lock.lock().unwrap();
    ///
    /// // Perform critical operations
    ///
    /// // Manually unlock with error handling
    /// guard.unlock().unwrap();
    /// ```
    pub fn unlock(mut self) -> Result<(), errno::Errno> {
        if !self.unlocked {
            self.lock.unlock()?;
            self.unlocked = true;
        }
        Ok(())
    }
}

impl<T: ?Sized> Deref for FileLockGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &*self.lock.data
    }
}

impl<T: ?Sized> DerefMut for FileLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut *self.lock.data
    }
}

impl<T: ?Sized> Drop for FileLockGuard<'_, T> {
    fn drop(&mut self) {
        if !self.unlocked {
            let _ = self.lock.unlock();
        }
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for FileLockGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T: ?Sized + fmt::Display> fmt::Display for FileLockGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

// A `FileLockGuard` is not `Send` to match the standard library's `MutexGuard`.
// File locks are tied to the file descriptor in the acquiring thread, and
// releasing them from a different thread can lead to platform-specific issues.
unsafe impl<T: ?Sized + Sync> Sync for FileLockGuard<'_, T> {}
