extern crate nix;

use std::fs::File;
use std::io;
use std::mem::ManuallyDrop;
use std::os::unix::prelude::*;

use PipeReader;
use PipeWriter;

pub fn pipe() -> io::Result<(PipeReader, PipeWriter)> {
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

pub fn dup_stdin() -> io::Result<PipeReader> {
    Ok(PipeReader(dup_fd(nix::libc::STDIN_FILENO)?))
}

pub fn dup_stdout() -> io::Result<PipeWriter> {
    Ok(PipeWriter(dup_fd(nix::libc::STDOUT_FILENO)?))
}

pub fn dup_stderr() -> io::Result<PipeWriter> {
    Ok(PipeWriter(dup_fd(nix::libc::STDERR_FILENO)?))
}

fn dup_fd(fd: RawFd) -> io::Result<File> {
    // We wrap the original file descriptor in a File, so that we can use
    // try_clone. Dropping the File would close the original descriptor,
    // though, so we wrap it in ManuallyDrop to prevent that.
    let temp_file = ManuallyDrop::new(unsafe { File::from_raw_fd(fd) });
    temp_file.try_clone()
}

fn nix_err_to_io_err(err: nix::Error) -> io::Error {
    if let nix::Error::Sys(err_no) = err {
        io::Error::from(err_no)
    } else {
        panic!("unexpected nix error type: {:?}", err)
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
