extern crate libc;

use std::fs::File;
use std::io;
use std::io::ErrorKind;
use std::os::unix::io::FromRawFd;
use libc::c_int;

#[macro_use]
mod weak;

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
        // TODO: In stdlib, this uses unstable features to be static. We can use lazy_static.
        let pipe2: weak::Weak<unsafe extern fn(*mut c_int, c_int) -> c_int> = weak::Weak::new("pipe2");
        if let Some(pipe) = pipe2.get() {
            match cvt_r(|| unsafe { pipe(fds.as_mut_ptr(), libc::O_CLOEXEC) }) {
                Ok(_) => {
                    return Ok(unsafe { pair_from_fds(fds) });
                }
                Err(ref e) if e.raw_os_error() == Some(libc::ENOSYS) => {}
                Err(e) => return Err(e),
            }
        }
    }
    if unsafe { libc::pipe(fds.as_mut_ptr()) == 0 } {
        Ok(unsafe { pair_from_fds(fds) })
    } else {
        Err(io::Error::last_os_error())
    }
}

pub fn cvt(t: c_int) -> io::Result<c_int> {
    if t == -1 {
        Err(io::Error::last_os_error())
    } else {
        Ok(t)
    }
}

pub fn cvt_r<F>(mut f: F) -> io::Result<c_int>
    where F: FnMut() -> c_int
{
    loop {
        match cvt(f()) {
            Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
            other => return other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::pipe;
    use std::io::prelude::*;

    #[test]
    fn pipe_some_data() {
        let mut pair = pipe().unwrap();
        pair.write.write_all(b"some stuff").unwrap();
        let mut out = String::new();
        pair.read.read_to_string(&mut out).unwrap();
        assert_eq!(out, "some stuff");
    }
}
