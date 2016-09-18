extern crate nix;

use std::fs::File;
use std::io;
use std::os::unix::prelude::*;
use std::process::Stdio;

use Pair;
use ParentHandle;

pub fn stdio_from_file(file: File) -> Stdio {
    unsafe { Stdio::from_raw_fd(file.into_raw_fd()) }
}

pub fn pipe() -> io::Result<Pair> {
    // O_CLOEXEC prevents children from inheriting these pipes. Nix's pipe2() will make a best
    // effort to make that atomic on platforms that support it, to avoid the case where another
    // thread forks right after the pipes are created but before O_CLOEXEC is set.
    let (read_fd, write_fd) = try!(nix::unistd::pipe2(nix::fcntl::O_CLOEXEC));

    unsafe {
        Ok(Pair {
            read: File::from_raw_fd(read_fd),
            write: File::from_raw_fd(write_fd),
        })
    }
}

pub fn parent_handle_to_stdio(handle: ParentHandle) -> io::Result<Stdio> {
    let fileno = match handle {
        ParentHandle::Stdin => nix::libc::STDIN_FILENO,
        ParentHandle::Stdout => nix::libc::STDOUT_FILENO,
        ParentHandle::Stderr => nix::libc::STDERR_FILENO,
    };
    dup_fd_cloexec(fileno)
}

fn dup_fd_cloexec(fd: RawFd) -> io::Result<Stdio> {
    // Atomically set O_CLOEXEC on the new fd.
    let new_fd = try!(nix::fcntl::fcntl(fd, nix::fcntl::FcntlArg::F_DUPFD_CLOEXEC(0)));
    unsafe { Ok(Stdio::from_raw_fd(new_fd)) }
}
