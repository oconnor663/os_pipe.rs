extern crate nix;

use std::fs::File;
use std::io;
use std::mem;
use std::os::unix::prelude::*;
use std::process::Stdio;

use Pair;

pub fn stdio_from_file(file: File) -> Stdio {
    unsafe { Stdio::from_raw_fd(file.into_raw_fd()) }
}

pub fn pipe() -> io::Result<Pair> {
    // O_CLOEXEC prevents children from inheriting these pipes. Nix's pipe2() will make a best
    // effort to make that atomic on platforms that support it, to avoid the case where another
    // thread forks right after the pipes are created but before O_CLOEXEC is set.
    let (read_fd, write_fd) = nix::unistd::pipe2(nix::fcntl::O_CLOEXEC)?;

    unsafe {
        Ok(Pair {
            read: File::from_raw_fd(read_fd),
            write: File::from_raw_fd(write_fd),
        })
    }
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
    mem::forget(temp_file);  // Prevent drop() to avoid closing fd.
    dup_result.map(stdio_from_file)
}
