extern crate nix;

use std::fs::File;
use std::io;
use std::os::unix::prelude::*;
use std::process::Stdio;

use PipeReader;
use PipeWriter;
use IntoStdio;

pub fn pipe() -> io::Result<(PipeReader, PipeWriter)> {
    // O_CLOEXEC prevents children from inheriting these pipes. Nix's pipe2() will make a best
    // effort to make that atomic on platforms that support it, to avoid the case where another
    // thread forks right after the pipes are created but before O_CLOEXEC is set.
    let (read_fd, write_fd) = nix::unistd::pipe2(nix::fcntl::O_CLOEXEC)?;

    unsafe { Ok((PipeReader::from_raw_fd(read_fd), PipeWriter::from_raw_fd(write_fd))) }
}

pub fn parent_stdin() -> io::Result<Stdio> {
    dup_fd(nix::libc::STDIN_FILENO)
}

pub fn parent_stdout() -> io::Result<Stdio> {
    dup_fd(nix::libc::STDOUT_FILENO)
}

pub fn parent_stderr() -> io::Result<Stdio> {
    dup_fd(nix::libc::STDERR_FILENO)
}

fn dup_fd(fd: RawFd) -> io::Result<Stdio> {
    let temp_file = unsafe { File::from_raw_fd(fd) };
    let dup_result = temp_file.try_clone();  // No short-circuit here!
    temp_file.into_raw_fd();  // Prevent closing fd on drop().
    dup_result.map(File::into_stdio)
}

impl<T: IntoRawFd> IntoStdio for T {
    fn into_stdio(self) -> Stdio {
        let fd = self.into_raw_fd();
        unsafe { Stdio::from_raw_fd(fd) }
    }
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
