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
use sys::cvt::cvt_r;
use sys::fd::FileDesc;

/// /////////////////////////////////////////////////////////////////////////////
/// Anonymous pipes
/// /////////////////////////////////////////////////////////////////////////////

pub struct AnonPipe(FileDesc);

pub fn anon_pipe() -> io::Result<(AnonPipe, AnonPipe)> {
    let mut fds = [0; 2];

    // Unfortunately the only known way right now to create atomically set the
    // CLOEXEC flag is to use the `pipe2` syscall on Linux. This was added in
    // 2.6.27, however, and because we support 2.6.18 we must detect this
    // support dynamically.
    if cfg!(target_os = "linux") {
        weak! { fn PIPE2(*mut c_int, c_int) -> c_int }
        if let Some(pipe) = PIPE2.get() {
            match cvt_r(|| unsafe { pipe(fds.as_mut_ptr(), libc::O_CLOEXEC) }) {
                Ok(_) => {
                    return Ok((AnonPipe(FileDesc::new(fds[0])), AnonPipe(FileDesc::new(fds[1]))))
                }
                Err(ref e) if e.raw_os_error() == Some(libc::ENOSYS) => {}
                Err(e) => return Err(e),
            }
        }
    }
    if unsafe { libc::pipe(fds.as_mut_ptr()) == 0 } {
        let fd0 = FileDesc::new(fds[0]);
        let fd1 = FileDesc::new(fds[1]);
        Ok((try!(AnonPipe::from_fd(fd0)), try!(AnonPipe::from_fd(fd1))))
    } else {
        Err(io::Error::last_os_error())
    }
}

impl AnonPipe {
    pub fn from_fd(fd: FileDesc) -> io::Result<AnonPipe> {
        try!(fd.set_cloexec());
        Ok(AnonPipe(fd))
    }

    pub fn into_fd(self) -> FileDesc {
        self.0
    }
}
