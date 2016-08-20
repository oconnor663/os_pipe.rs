#[macro_use]
mod weak;
mod cvt;
mod fd;
mod pipe;

use std::fs::File;
use std::io;
use std::process::Stdio;
use std::os::unix::io::{FromRawFd, IntoRawFd};

use Pair;

pub fn stdio_from_file(file: File) -> Stdio {
    unsafe { Stdio::from_raw_fd(file.into_raw_fd()) }
}

pub fn pipe() -> io::Result<Pair> {
    unimplemented!()
}

pub fn dup_stdin() -> io::Result<Stdio> {
    unimplemented!()
}

pub fn dup_stdout() -> io::Result<Stdio> {
    unimplemented!()
}

pub fn dup_stderr() -> io::Result<Stdio> {
    unimplemented!()
}
