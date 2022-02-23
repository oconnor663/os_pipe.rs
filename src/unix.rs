use crate::PipeReader;
use crate::PipeWriter;
use libc::c_int;
use std::fs::File;
use std::io;
use std::mem::ManuallyDrop;
use std::os::unix::prelude::*;

// We need to atomically create pipes and set the CLOEXEC flag on them. This is
// done with the pipe2() API. However, macOS doesn't support pipe2. There, all
// we can do is call pipe() followed by fcntl(), and hope that no other threads
// fork() in between. The following code is copied from the nix crate, where it
// works but is deprecated.
#[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "haiku")))]
fn pipe2_cloexec() -> io::Result<(c_int, c_int)> {
    let mut fds: [c_int; 2] = [0; 2];
    let res = unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC) };
    if res != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok((fds[0], fds[1]))
}

#[cfg(any(target_os = "ios", target_os = "macos", target_os = "haiku"))]
fn pipe2_cloexec() -> io::Result<(c_int, c_int)> {
    let mut fds: [c_int; 2] = [0; 2];
    let res = unsafe { libc::pipe(fds.as_mut_ptr()) };
    if res != 0 {
        return Err(io::Error::last_os_error());
    }
    let res = unsafe { libc::fcntl(fds[0], libc::F_SETFD, libc::FD_CLOEXEC) };
    if res != 0 {
        return Err(io::Error::last_os_error());
    }
    let res = unsafe { libc::fcntl(fds[1], libc::F_SETFD, libc::FD_CLOEXEC) };
    if res != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok((fds[0], fds[1]))
}

pub(crate) fn pipe() -> io::Result<(PipeReader, PipeWriter)> {
    let (read_fd, write_fd) = pipe2_cloexec()?;
    unsafe {
        Ok((
            PipeReader::from_raw_fd(read_fd),
            PipeWriter::from_raw_fd(write_fd),
        ))
    }
}

pub(crate) fn dup<F: AsRawFd>(wrapper: &F) -> io::Result<File> {
    let fd = wrapper.as_raw_fd();
    let temp_file = ManuallyDrop::new(unsafe { File::from_raw_fd(fd) });
    temp_file.try_clone()
}

impl IntoRawFd for PipeReader {
    fn into_raw_fd(self) -> RawFd {
        self.0.into_raw_fd()
    }
}

impl AsRawFd for PipeReader {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl FromRawFd for PipeReader {
    unsafe fn from_raw_fd(fd: RawFd) -> PipeReader {
        PipeReader(File::from_raw_fd(fd))
    }
}

impl IntoRawFd for PipeWriter {
    fn into_raw_fd(self) -> RawFd {
        self.0.into_raw_fd()
    }
}

impl AsRawFd for PipeWriter {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl FromRawFd for PipeWriter {
    unsafe fn from_raw_fd(fd: RawFd) -> PipeWriter {
        PipeWriter(File::from_raw_fd(fd))
    }
}
