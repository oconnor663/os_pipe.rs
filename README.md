# os_pipe.rs [![Travis build](https://travis-ci.org/oconnor663/os_pipe.rs.svg?branch=master)](https://travis-ci.org/oconnor663/os_pipe.rs) [![AppVeyor build](https://ci.appveyor.com/api/projects/status/89o6o64nxfl80s78/branch/master?svg=true)](https://ci.appveyor.com/project/oconnor663/os-pipe-rs/branch/master) [![crates.io](https://img.shields.io/crates/v/os_pipe.svg)](https://crates.io/crates/os_pipe) [![docs.rs](https://docs.rs/os_pipe/badge.svg)](https://docs.rs/os_pipe)

A cross-platform library for opening OS pipes.

The standard library uses pipes to read output from child processes,
but it doesn't expose a way to create them directly. This crate
fills that gap with the `pipe` function. It also includes some
helpers for passing pipes to the `std::process::Command` API.

- [Docs](https://docs.rs/os_pipe)
- [Crate](https://crates.io/crates/os_pipe)
- [Repo](https://github.com/oconnor663/os_pipe.rs)

Usage note: The main purpose of `os_pipe` is to support the
higher-level [`duct`](https://github.com/oconnor663/duct.rs)
library, which handles most of the same use cases with much less
code and no risk of deadlocks. `duct` can run the entire example
below in one line of code.

## Example

Join the stdout and stderr of a child process into a single stream,
and read it. To do that we open a pipe, duplicate its write end, and
pass those writers as the child's stdout and stderr. Then we can
read combined output from the read end of the pipe. We have to be
careful to close the write ends first though, or reading will block
waiting for EOF.

```rust
use os_pipe::pipe;
use std::io::prelude::*;
use std::process::{Command, Stdio};

// This command prints "foo" to stdout and "bar" to stderr. It
// works on both Unix and Windows, though there are whitespace
// differences that we'll account for at the bottom.
let shell_command = "echo foo && echo bar >&2";

// Ritual magic to run shell commands on different platforms.
let (shell, flag) = if cfg!(windows) { ("cmd.exe", "/C") } else { ("sh", "-c") };

let mut child = Command::new(shell);
child.arg(flag);
child.arg(shell_command);

// Here's the interesting part. Open a pipe, copy its write end, and
// give both copies to the child.
let (mut reader, writer) = pipe().unwrap();
let writer_clone = writer.try_clone().unwrap();
child.stdout(writer);
child.stderr(writer_clone);

// Now start the child running.
let mut handle = child.spawn().unwrap();

// Very important when using pipes: This parent process is still
// holding its copies of the write ends, and we have to close them
// before we read, otherwise the read end will never report EOF. The
// Command object owns the writers now, and dropping it closes them.
drop(child);

// Finally we can read all the output and clean up the child.
let mut output = String::new();
reader.read_to_string(&mut output).unwrap();
handle.wait().unwrap();
assert!(output.split_whitespace().eq(vec!["foo", "bar"]));
```
