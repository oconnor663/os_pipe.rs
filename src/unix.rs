extern crate libc;

use std::fs::File;
use std::io;
use std::io::ErrorKind;
use std::os::unix::io::{FromRawFd, IntoRawFd};
use std::process::Stdio;
use libc::c_int;

use self::weak::Weak;

use ::Pair;

lazy_static! {
    static ref PIPE2: Weak<unsafe extern fn(*mut c_int, c_int) -> c_int> = Weak::new("pipe2");
}

pub fn stdio_from_file(file: File) -> Stdio {
    unsafe { Stdio::from_raw_fd(file.into_raw_fd()) }
}

pub fn pipe() -> io::Result<Pair> {
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

pub fn dup_stdin() -> io::Result<File> {
    dup_fd(libc::STDIN_FILENO)
}

pub fn dup_stdout() -> io::Result<File> {
    dup_fd(libc::STDOUT_FILENO)
}

pub fn dup_stderr() -> io::Result<File> {
    dup_fd(libc::STDERR_FILENO)
}

fn dup_fd(fd: c_int) -> io::Result<File> {
    unsafe {
        let new_fd = try!(cvt_r(|| libc::dup(fd)));
        Ok(File::from_raw_fd(new_fd))
    }
}

unsafe fn pair_from_fds(fds: [c_int; 2]) -> Pair {
    Pair {
        read: File::from_raw_fd(fds[0]),
        write: File::from_raw_fd(fds[1]),
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

mod weak {
    // Copyright 2016 The Rust Project Developers. See the COPYRIGHT
    // file at the top-level directory of this distribution and at
    // http://rust-lang.org/COPYRIGHT.
    //
    // Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
    // http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
    // <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
    // option. This file may not be copied, modified, or distributed
    // except according to those terms.

    //! Support for "weak linkage" to symbols on Unix
    //!
    //! Some I/O operations we do in libstd require newer versions of OSes but we
    //! need to maintain binary compatibility with older releases for now. In order
    //! to use the new functionality when available we use this module for
    //! detection.
    //!
    //! One option to use here is weak linkage, but that is unfortunately only
    //! really workable on Linux. Hence, use dlsym to get the symbol value at
    //! runtime. This is also done for compatibility with older versions of glibc,
    //! and to avoid creating dependencies on GLIBC_PRIVATE symbols. It assumes that
    //! we've been dynamically linked to the library the symbol comes from, but that
    //! is currently always the case for things like libpthread/libc.
    //!
    //! A long time ago this used weak linkage for the __pthread_get_minstack
    //! symbol, but that caused Debian to detect an unnecessarily strict versioned
    //! dependency on libc6 (#23628).

    use libc;

    use std::ffi::CString;
    use std::marker;
    use std::mem;
    use std::sync::atomic::{AtomicUsize, Ordering};

    pub struct Weak<F> {
        name: &'static str,
        addr: AtomicUsize,
        _marker: marker::PhantomData<F>,
    }

    impl<F> Weak<F> {
        pub fn new(name: &'static str) -> Weak<F> {
            Weak {
                name: name,
                addr: AtomicUsize::new(1),
                _marker: marker::PhantomData,
            }
        }

        pub fn get(&self) -> Option<&F> {
            assert_eq!(mem::size_of::<F>(), mem::size_of::<usize>());
            unsafe {
                if self.addr.load(Ordering::SeqCst) == 1 {
                    self.addr.store(fetch(self.name), Ordering::SeqCst);
                }
                if self.addr.load(Ordering::SeqCst) == 0 {
                    None
                } else {
                    mem::transmute::<&AtomicUsize, Option<&F>>(&self.addr)
                }
            }
        }
    }

    unsafe fn fetch(name: &str) -> usize {
        let name = match CString::new(name) {
            Ok(cstr) => cstr,
            Err(..) => return 0,
        };
        libc::dlsym(libc::RTLD_DEFAULT, name.as_ptr()) as usize
    }
}
