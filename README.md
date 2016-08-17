# os_pipe.rs [![Build Status](https://travis-ci.org/oconnor663/os_pipe.rs.svg?branch=master)](https://travis-ci.org/oconnor663/os_pipe.rs) [![Build status](https://ci.appveyor.com/api/projects/status/89o6o64nxfl80s78/branch/master?svg=true)](https://ci.appveyor.com/project/oconnor663/os-pipe-rs/branch/master)

A cross-platform Rust library for opening pipes, backed by libc's
`pipe()` on Unix and `CreatePipe` on Windows. Most of the code is
adapted from unexposed parts of the Rust standard library.

This library was created mainly for
[duct.rs](https://github.com/oconnor663/duct.rs), but I'm happy to get
feature requests. I'd especially appreciate corrections from people more
familiar with these OS-specific functions.
