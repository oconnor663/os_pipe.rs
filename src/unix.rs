use crate::PipeReader;
use crate::PipeWriter;

use std::fs::File;
use std::io;
use std::os::unix::prelude::*;

use rustix::fd::OwnedFd;
use rustix::pipe;

// We need to atomically create pipes and set the CLOEXEC flag on them. This is
// done with the pipe2() API. However, macOS doesn't support pipe2. There, all
// we can do is call pipe() followed by fcntl(), and hope that no other threads
// fork() in between. The following code is copied from the nix crate, where it
// works but is deprecated.
#[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "haiku")))]
fn pipe2_cloexec() -> io::Result<(OwnedFd, OwnedFd)> {
    let pipe = pipe::pipe_with(pipe::PipeFlags::CLOEXEC)?;
    Ok(pipe)
}

#[cfg(any(target_os = "ios", target_os = "macos", target_os = "haiku"))]
fn pipe2_cloexec() -> io::Result<(OwnedFd, OwnedFd)> {
    use rustix::io::{fcntl_setfd, FdFlags};

    let (left, right) = pipe::pipe()?;
    fcntl_setfd(&left, FdFlags::CLOEXEC)?;
    fcntl_setfd(&right, FdFlags::CLOEXEC)?;
    Ok((left, right))
}

pub(crate) fn pipe() -> io::Result<(PipeReader, PipeWriter)> {
    let (read_fd, write_fd) = pipe2_cloexec()?;
    Ok((
        PipeReader::from(read_fd),
        PipeWriter::from(write_fd),
    ))
}

pub(crate) fn dup<F: AsFd>(wrapper: &F) -> io::Result<File> {
    Ok(wrapper.as_fd().try_clone_to_owned()?.into())
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

impl From<PipeReader> for OwnedFd {
    fn from(pr: PipeReader) -> Self {
        pr.0.into()
    }
}

impl AsFd for PipeReader {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

impl From<OwnedFd> for PipeReader {
    fn from(fd: OwnedFd) -> Self {
        PipeReader(fd.into())
    }
}

impl From<PipeWriter> for OwnedFd {
    fn from(pw: PipeWriter) -> Self {
        pw.0.into()
    }
}

impl AsFd for PipeWriter {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

impl From<OwnedFd> for PipeWriter {
    fn from(fd: OwnedFd) -> Self {
        PipeWriter(fd.into())
    }
}
