use std::os::unix::prelude::*;

use libc::c_int;

use crate::futures::blocking;

pub use libc::{STDOUT_FILENO, STDIN_FILENO, STDERR_FILENO};

pub struct RawFd(pub std::os::unix::io::RawFd);
pub type AsyncRawFd = blocking::Blocking<RawFd>;

// https://stackoverflow.com/questions/42772307/how-do-i-handle-errors-from-libc-functions-in-an-idiomatic-rust-manner
fn check_err<T: Ord + Default>(num: T) -> Result<T, std::io::Error> {
    if num < T::default() {
        return Err(std::io::Error::last_os_error());
    }
    Ok(num)
}

impl std::io::Read for RawFd {
    fn read(&mut self, buf: &mut [u8]) -> std::result::Result<usize, std::io::Error> {
        match check_err(unsafe {
            libc::read(self.0, buf.as_mut_ptr() as *mut libc::c_void, buf.len())
        }) {
            Ok(i) => Ok(i as usize),
            Err(i) => Err(i),
        }
    }
}

impl std::io::Write for RawFd {
    fn write(&mut self, buf: &[u8]) -> std::result::Result<usize, std::io::Error> {
        match check_err(unsafe {
            libc::write(self.0, buf.as_ptr() as *mut libc::c_void, buf.len())
        }) {
            Ok(i) => Ok(i as usize),
            Err(i) => Err(i),
        }
    }
    // Flush doesn't do anything for a raw file descriptor.
    fn flush(&mut self) -> std::result::Result<(), std::io::Error> {
        Ok(())
    }
}

impl FromRawFd for RawFd {
    unsafe fn from_raw_fd(fd: std::os::unix::io::RawFd) -> Self {
        {
            RawFd(fd)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::prelude::*;
    use crate::io::AsyncWriteExt;

    async fn run() -> Result<usize, std::io::Error> {
        let fd = unsafe { RawFd::from_raw_fd(STDOUT_FNUM) };
        let mut fd = AsyncRawFd::new(fd);
        AsyncWriteExt::write(&mut fd, "Hello\n".as_bytes()).await
    }
    #[test]
    fn test_hello() {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(run()).unwrap();
    }

    #[test]
    fn swap_stdout() {
        unsafe {
            let new_sout = libc::dup(STDOUT_FNUM);
            //libc::close(STDOUT_FNUM);
            let mut fd = RawFd::from_raw_fd(new_sout);
            fd.write("Hello world!\n".as_bytes()).unwrap();
        }
    }

}
