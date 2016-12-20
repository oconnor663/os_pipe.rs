# os_pipe.rs [![Travis build](https://travis-ci.org/oconnor663/os_pipe.rs.svg?branch=master)](https://travis-ci.org/oconnor663/os_pipe.rs) [![AppVeyor build](https://ci.appveyor.com/api/projects/status/89o6o64nxfl80s78/branch/master?svg=true)](https://ci.appveyor.com/project/oconnor663/os-pipe-rs/branch/master) [![crates.io](https://img.shields.io/crates/v/os_pipe.svg)](https://crates.io/crates/os_pipe) [![docs.rs](https://docs.rs/os_pipe/badge.svg)](https://docs.rs/os_pipe)

A cross-platform library for opening OS pipes.

The standard library uses pipes to read output from child processes,
but it doesn't expose a way to create them directly. This crate
fills that gap with the `pipe` function. It also includes some
utilities for passing pipes to `std::process::Command` API.

The main motivation for this crate is to provide pipes for the
higher level [`duct`](https://crates.io/crates/duct) crate. If your
main use case for pipes is to talk to child processes, `duct` can
handle all of the details for you.

# Example

```rust
// Join the stdout and stderr of a child process into a single
// stream, and read it. We do this by opening a pipe, duping its
// write end, using passing those write ends as the stdout and
// stderr of the child. We then read from the read end of the pipe,
// though we have to be careful to close the write ends by dropping
// the Command object that's holding them, or else `read_to_end`
// will block forever.

use os_pipe::{pipe, Pair, stdio_from_file};
use std::io::prelude::*;
use std::process::Command;

let Pair{mut read, write} = pipe().unwrap();
let write_copy = write.try_clone().unwrap();
let mut child = Command::new("echo");
child.arg("foo");
child.stdout(stdio_from_file(write));
child.stderr(stdio_from_file(write_copy));
let mut handle = child.spawn().unwrap();
drop(child);
let mut stdout_and_stderr = Vec::new();
read.read_to_end(&mut stdout_and_stderr).unwrap();
handle.wait().unwrap();
```
