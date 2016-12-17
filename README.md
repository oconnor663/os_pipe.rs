# os_pipe.rs [![Travis build](https://travis-ci.org/oconnor663/os_pipe.rs.svg?branch=master)](https://travis-ci.org/oconnor663/os_pipe.rs) [![AppVeyor build](https://ci.appveyor.com/api/projects/status/89o6o64nxfl80s78/branch/master?svg=true)](https://ci.appveyor.com/project/oconnor663/os-pipe-rs/branch/master) [![crates.io](https://img.shields.io/crates/v/os_pipe.svg)](https://crates.io/crates/os_pipe) [![docs.rs](https://docs.rs/os_pipe/badge.svg)](https://docs.rs/os_pipe)

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
- `stdio_from_file()` is a helper function to safely convert a
  `std::fs::File` to a `std::process::Stdio` object, for passing to
  child processes. The standard library supports this conversion, but it
  requires platform-specific traits and takes an `unsafe` call.
  Currently there's not really such a thing as a "closed file" in Rust,
  since closing requires dropping, but if Rust ever introduces closed
  files in the future this function will panic on them.
