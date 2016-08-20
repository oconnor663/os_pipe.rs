// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::io;
use libc::{self, c_int};
use std::mem;
use sys::cvt::cvt;

pub struct FileDesc {
    fd: c_int,
}

impl FileDesc {
    pub fn new(fd: c_int) -> FileDesc {
        FileDesc { fd: fd }
    }

    pub fn raw(&self) -> c_int {
        self.fd
    }

    /// Extracts the actual filedescriptor without closing it.
    pub fn into_raw(self) -> c_int {
        let fd = self.fd;
        mem::forget(self);
        fd
    }

    #[cfg(not(any(target_env = "newlib", target_os = "solaris", target_os = "emscripten")))]
    pub fn set_cloexec(&self) -> io::Result<()> {
        unsafe {
            try!(cvt(libc::ioctl(self.fd, libc::FIOCLEX)));
            Ok(())
        }
    }
    #[cfg(any(target_env = "newlib", target_os = "solaris", target_os = "emscripten"))]
    pub fn set_cloexec(&self) -> io::Result<()> {
        unsafe {
            let previous = try!(cvt(libc::fcntl(self.fd, libc::F_GETFD)));
            try!(cvt(libc::fcntl(self.fd, libc::F_SETFD, previous | libc::FD_CLOEXEC)));
            Ok(())
        }
    }
}

impl Drop for FileDesc {
    fn drop(&mut self) {
        // Note that errors are ignored when closing a file descriptor. The
        // reason for this is that if an error occurs we don't actually know if
        // the file descriptor was closed or not, and if we retried (for
        // something like EINTR), we might close another valid file descriptor
        // (opened after we closed ours.
        let _ = unsafe { libc::close(self.fd) };
    }
}
