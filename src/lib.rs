extern crate libc;

#[macro_use]
extern crate lazy_static;

use std::fs::File;
use std::io;
use std::io::ErrorKind;
use std::os::unix::io::{FromRawFd, IntoRawFd};
use std::process::Stdio;
use libc::c_int;

#[macro_use]
mod weak;
use weak::Weak;

lazy_static! {
    static ref PIPE2: Weak<unsafe extern fn(*mut c_int, c_int) -> c_int> = Weak::new("pipe2");
}

pub struct PipePair {
    pub read: File,
    pub write: File,
}

unsafe fn pair_from_fds(fds: [c_int; 2]) -> PipePair {
    PipePair {
        read: File::from_raw_fd(fds[0]),
        write: File::from_raw_fd(fds[1]),
    }
}

pub fn pipe() -> io::Result<PipePair> {
    let mut fds = [0; 2];

    // Unfortunately the only known way right now to create atomically set the
    // CLOEXEC flag is to use the `pipe2` syscall on Linux. This was added in
    // 2.6.27, however, and because we support 2.6.18 we must detect this
    // support dynamically.
    if cfg!(target_os = "linux") {
        if let Some(pipe2) = PIPE2.get() {
            match cvt_r(|| unsafe { pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC) }) {
                Ok(_) => {
                    return Ok(unsafe { pair_from_fds(fds) });
                }
                Err(ref e) if e.raw_os_error() == Some(libc::ENOSYS) => {}
                Err(e) => return Err(e),
            }
        }
    }

    if unsafe { libc::pipe(fds.as_mut_ptr()) == 0 } {
        set_cloexec(fds[0]);
        set_cloexec(fds[1]);
        Ok(unsafe { pair_from_fds(fds) })
    } else {
        Err(io::Error::last_os_error())
    }
}

fn cvt(t: c_int) -> io::Result<c_int> {
    if t == -1 {
        Err(io::Error::last_os_error())
    } else {
        Ok(t)
    }
}

fn cvt_r<F>(mut f: F) -> io::Result<c_int>
    where F: FnMut() -> c_int
{
    loop {
        match cvt(f()) {
            Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
            other => return other,
        }
    }
}

#[cfg(not(any(target_env = "newlib", target_os = "solaris", target_os = "emscripten")))]
fn set_cloexec(fd: c_int) {
    unsafe {
        let ret = libc::ioctl(fd, libc::FIOCLEX);
        debug_assert_eq!(ret, 0);
    }
}

#[cfg(any(target_env = "newlib", target_os = "solaris", target_os = "emscripten"))]
fn set_cloexec(fd: c_int) {
    unsafe {
        let previous = libc::fcntl(fd, libc::F_GETFD);
        let ret = libc::fcntl(fd, libc::F_SETFD, previous | libc::FD_CLOEXEC);
        debug_assert_eq!(ret, 0);
    }
}

pub fn stdio_from_file(file: File) -> Stdio {
    unsafe { Stdio::from_raw_fd(file.into_raw_fd()) }
}

#[cfg(test)]
mod tests {
    use std::io::prelude::*;
    use std::process;
    use std::thread;

    #[test]
    fn test_pipe_some_data() {
        let mut pair = ::pipe().unwrap();
        // A small write won't fill the pipe buffer, so it won't block this thread.
        pair.write.write_all(b"some stuff").unwrap();
        drop(pair.write);
        let mut out = String::new();
        pair.read.read_to_string(&mut out).unwrap();
        assert_eq!(out, "some stuff");
    }

    #[test]
    fn test_pipe_no_data() {
        let mut pair = ::pipe().unwrap();
        drop(pair.write);
        let mut out = String::new();
        pair.read.read_to_string(&mut out).unwrap();
        assert_eq!(out, "");
    }

    #[test]
    fn test_pipe_a_megabyte_of_data_from_another_thread() {
        let data = vec![0xff; 1_000_000];
        let data_copy = data.clone();
        let ::PipePair { mut read, mut write } = ::pipe().unwrap();
        let joiner = thread::spawn(move || {
            write.write_all(&data_copy).unwrap();
        });
        let mut out = Vec::new();
        read.read_to_end(&mut out).unwrap();
        joiner.join().unwrap();
        assert_eq!(out, data);
    }

    #[test]
    fn test_pipes_are_not_inheritable() {
        // Create pipes for a child process.
        let mut input_pipe = ::pipe().unwrap();
        let mut output_pipe = ::pipe().unwrap();
        let child_stdin = ::stdio_from_file(input_pipe.read);
        let child_stdout = ::stdio_from_file(output_pipe.write);

        // Spawn the child. Note that this temporary Command object takes ownership of our copies
        // of the child's stdin and stdout, and then closes them immediately when it drops. That
        // stops us from blocking our own read below.
        let mut child = process::Command::new("cat")
            .stdin(child_stdin)
            .stdout(child_stdout)
            .spawn()
            .unwrap();

        // Write to the child's stdin. This is a small write, so it shouldn't block.
        input_pipe.write.write_all(b"hello").unwrap();
        drop(input_pipe.write);

        // Read from the child's stdout. If this child has accidentally inherited the write end of
        // its own stdin, then it will never exit, and this read will block forever. That's the
        // what this test is all about.
        let mut output = Vec::new();
        output_pipe.read.read_to_end(&mut output).unwrap();
        child.wait().unwrap();

        // Confirm that we got the right bytes.
        assert_eq!(b"hello", &*output);
    }
}
