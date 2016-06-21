extern crate libc;

use std::fs::File;
use std::io;
use std::os::unix::io::FromRawFd;

#[macro_use]
mod weak;

struct PipePair {
    read: File,
    write: File,
}

fn pair_from_fds(fds: [libc::c_int; 2]) -> PipePair {
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
        weak! { fn pipe2(*mut c_int, c_int) -> c_int }
        if let Some(pipe) = pipe2.get() {
            match cvt_r(|| unsafe { pipe(fds.as_mut_ptr(), libc::O_CLOEXEC) }) {
                Ok(_) => {
                    return Ok(pair_from_fds(fds));
                }
                Err(ref e) if e.raw_os_error() == Some(libc::ENOSYS) => {}
                Err(e) => return Err(e),
            }
        }
    }
    if unsafe { libc::pipe(fds.as_mut_ptr()) == 0 } {
        Ok(pair_from_fds(fds))
    } else {
        Err(io::Error::last_os_error())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
