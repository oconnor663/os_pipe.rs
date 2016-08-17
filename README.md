# os_pipe.rs [![Build Status](https://travis-ci.org/oconnor663/os_pipe.rs.svg?branch=master)](https://travis-ci.org/oconnor663/os_pipe.rs) [![Build status](https://ci.appveyor.com/api/projects/status/89o6o64nxfl80s78/branch/master?svg=true)](https://ci.appveyor.com/project/oconnor663/os-pipe-rs/branch/master)

A cross-platform Rust library for opening pipes, backed by libc's
`pipe()` on Unix and `CreatePipe` on Windows. Most of the code is
adapted from unexposed parts of the Rust standard library.

This library was created mainly for
[duct.rs](https://github.com/oconnor663/duct.rs), but I'm happy to get
feature requests. I'd especially appreciate corrections from people more
familiar with these OS-specific functions.

Current API:

- `pipe()` returns two `File` objects, the reading and writing ends of
  the new pipe, as the `read` and `write` members of a `Pair` struct.
- `dup_stdin()`, `dup_stdout()`, and `dup_stderr()` return duplicated
  copies of the stdin/stdout/stderr file handles. These aren't
  synchronized with the handles in `std::io`, so you shouldn't do actual
  IO with them. Their purpose is to let you do interesting things to a
  child process's pipes with `std::process::Command`. (TODO: Make these
  just std::process::Stdio objects, do discourage IO?)
- `stdio_from_file()` is a helper function to safely convert a `File` to
  a `std::process::Stdio` object, for passing to child processes. The
  standard library does provide this conversion, but it uses
  platform-specific traits and takes an `unsafe` call.
