extern crate nix;

use std::fs::File;
use std::io;
use std::mem::ManuallyDrop;
use std::os::unix::prelude::*;

use crate::PipeReader;
use crate::PipeWriter;

pub(crate) fn pipe() -> io::Result<(PipeReader, PipeWriter)> {
    // O_CLOEXEC prevents children from inheriting these pipes. Nix's pipe2() will make a best
    // effort to make that atomic on platforms that support it, to avoid the case where another
    // thread forks right after the pipes are created but before O_CLOEXEC is set.
    let (read_fd, write_fd) =
        nix::unistd::pipe2(nix::fcntl::OFlag::O_CLOEXEC).map_err(nix_err_to_io_err)?;

    unsafe {
        Ok((
            PipeReader::from_raw_fd(read_fd),
            PipeWriter::from_raw_fd(write_fd),
        ))
    }
}

fn nix_err_to_io_err(err: nix::Error) -> io::Error {
    if let nix::Error::Sys(err_no) = err {
        io::Error::from(err_no)
    } else {
        panic!("unexpected nix error type: {:?}", err)
    }
}

pub(crate) fn dup<T: AsRawFd>(wrapper: T) -> io::Result<File> {
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
