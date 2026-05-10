extern crate errno;
extern crate libc;

use crate::FileLockGuard;
use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

pub struct FileLock<T: ?Sized> {
    filename: PathBuf,
    fd: libc::c_int,
    pub(crate) data: Box<T>,
}

impl<T> FileLock<T> {
    pub fn new<P: AsRef<Path>>(filename: P, data: T) -> FileLock<T> {
        FileLock {
            filename: filename.as_ref().to_path_buf(),
            fd: 0,
            data: Box::new(data),
        }
    }
}

impl<T: ?Sized> FileLock<T> {
    pub fn lock(&mut self) -> Result<FileLockGuard<'_, T>, errno::Errno> {
        unsafe {
            let c_filename = CString::new(self.filename.as_os_str().as_bytes()).unwrap();
            let fd = libc::open(c_filename.as_ptr(), libc::O_RDWR | libc::O_CREAT, 0o644);
            if fd < 0 {
                return Err(errno::errno());
            }
            self.fd = fd;

            if libc::flock(fd, libc::LOCK_EX) != 0 {
                return Err(errno::errno());
            }
            Ok(FileLockGuard::new(self))
        }
    }

    pub(crate) fn unlock(&mut self) -> Result<(), errno::Errno> {
        let fd = self.fd;

        unsafe {
            if libc::flock(fd, libc::LOCK_UN) != 0 {
                return Err(errno::errno());
            }

            if libc::close(fd) != 0 {
                return Err(errno::errno());
            }
        }

        self.fd = 0;

        Ok(())
    }
}

impl<T: ?Sized> Drop for FileLock<T> {
    fn drop(&mut self) {
        if self.fd > 0 {
            unsafe {
                libc::close(self.fd);
            }
            self.fd = 0;
        }

        // Try to delete the lock file, allow failure
        let _ = std::fs::remove_file(&self.filename);
    }
}
