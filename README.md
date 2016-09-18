# os_pipe.rs [![Build Status](https://travis-ci.org/oconnor663/os_pipe.rs.svg?branch=master)](https://travis-ci.org/oconnor663/os_pipe.rs) [![Build status](https://ci.appveyor.com/api/projects/status/89o6o64nxfl80s78/branch/master?svg=true)](https://ci.appveyor.com/project/oconnor663/os-pipe-rs/branch/master)

A cross-platform Rust library for opening anonymous pipes, backed by
[`nix`](https://github.com/nix-rust/nix) on Unix and
[`winapi`](https://github.com/retep998/winapi-rs) on Windows. If anyone
needs it, we could also add support for named pipes and IOCP (using
random names) on Windows, or creating filesystem FIFO's on Unix.

Current API:

- `pipe()` returns two `std::fs::File` objects, the reading and writing
  ends of the new pipe, as the `read` and `write` members of a `Pair`
  struct.
- `parent_stdin()`, `parent_stdout()`, and `parent_stderr()` return
  duplicated copies of the stdin/stdout/stderr file handles as
  `std::process::Stdio` objects that can be passed to child processes.
  This is useful for e.g. swapping stdout and stderr.
- The `IntoStdio` trait makes it easier to build `std::process::Stdio`
  objects from e.g. `std::fs::File`. The standard library supports this
  conversion, but it requires platform-specific traits and takes an
  `unsafe` call.
