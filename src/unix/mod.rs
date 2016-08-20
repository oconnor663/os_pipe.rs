#[macro_use]
mod weak;
mod cvt;
mod fd;
mod pipe;

use libc;
use std::fs::File;
use std::io;
use std::mem;
use std::process::Stdio;
use std::os::unix::io::{FromRawFd, IntoRawFd};

use Pair;

pub fn stdio_from_file(file: File) -> Stdio {
    unsafe { Stdio::from_raw_fd(file.into_raw_fd()) }
}

pub fn pipe() -> io::Result<Pair> {
    let (anon_read, anon_write) = try!(pipe::anon_pipe());
    unsafe {
        Ok(Pair {
            read: File::from_raw_fd(anon_read.into_fd().into_raw()),
            write: File::from_raw_fd(anon_write.into_fd().into_raw()),
        })
    }
}

fn dup_fd(fd: libc::c_int) -> io::Result<Stdio> {
    let temp = unsafe { File::from_raw_fd(fd) };
    // Note that we don't return early if this duplicate fails. We *must*
    // forget the temp File, to avoid closing the original handle.
    let dup_result = temp.try_clone();
    mem::forget(temp);
    dup_result.map(stdio_from_file)
}

pub fn dup_stdin() -> io::Result<Stdio> {
    dup_fd(libc::STDIN_FILENO)
}

pub fn dup_stdout() -> io::Result<Stdio> {
    dup_fd(libc::STDOUT_FILENO)
}

pub fn dup_stderr() -> io::Result<Stdio> {
    dup_fd(libc::STDERR_FILENO)
}
