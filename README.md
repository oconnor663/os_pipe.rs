# os_pipe.rs [![Actions Status](https://github.com/oconnor663/os_pipe.rs/workflows/tests/badge.svg)](https://github.com/oconnor663/os_pipe.rs/actions) [![crates.io](https://img.shields.io/crates/v/os_pipe.svg)](https://crates.io/crates/os_pipe) [![docs.rs](https://docs.rs/os_pipe/badge.svg)](https://docs.rs/os_pipe)

A cross-platform library for opening OS pipes, like those from the Unix
[`pipe`](https://man7.org/linux/man-pages/man2/pipe.2.html) system call. The Rust standard
library provides
[`Stdio::piped`](https://doc.rust-lang.org/std/process/struct.Stdio.html#method.piped) for
simple use cases involving child processes, but it doesn't support creating pipes directly.
This crate provides the `pipe` function to fill that gap.

- [Docs](https://docs.rs/os_pipe)
- [Crate](https://crates.io/crates/os_pipe)
- [Repo](https://github.com/oconnor663/os_pipe.rs)

## Common deadlocks related to pipes

When you work with pipes, you almost always end up debugging a
deadlock at some point. These can be very confusing if you don't
know why they happen. There are two crucial details you need to
know:

1. When a program reads from a pipe, it will block waiting for input
   as long as there's at least one writer that's still open. That
   means that if you **forget to close one of your writers**, your
   readers will block forever.
2. Pipes use an internal buffer with a fixed size. When you do a few
   small writes, your bytes will get copied into that buffer.
   However, if you write a lot (sometimes >64 KiB), and there aren't
   any readers consuming those bytes and clearing space in the pipe
   buffer, **eventually the pipe buffer will fill up**,
   and your writes will block waiting for space.

Deadlocks caused by a forgotten writer usually show up immediately
and reliably. That makes them relatively easy to fix, once you know
what to look for. However, deadlocks caused by full pipe buffers are
more complicated. These might only show up for larger inputs, and
they might be timing-dependent or platform-dependent. If you find
that writing to a pipe causes "weird" deadlocks only some of the
time, consider whether the pipe buffer might be filling up. For more
on this, see the [Gotchas
Doc](https://github.com/oconnor663/duct.py/blob/master/gotchas.md#using-io-threads-to-avoid-blocking-children)
from the [`duct`](https://github.com/oconnor663/duct.rs) crate. (And
consider whether [`duct`](https://github.com/oconnor663/duct.rs)
might be a good fit for your use case.)

## Examples

Here we write a single byte into a pipe and read it back out:

```rust
use std::io::prelude::*;

let (mut reader, mut writer) = os_pipe::pipe()?;
writer.write_all(b"x")?;
let mut output = [0];
reader.read_exact(&mut output)?;
assert_eq!(b"x", &output);
```

This is a minimal working example, but as discussed in the section
above, reading and writing on the same thread like this is
deadlock-prone. If we tried to write ~100 KB instead of just one
byte, this example would certainly deadlock.

For a more complex example, here we join the stdout and stderr of a
child process into a single pipe. To do that we open a pipe, clone
its writer, and set that pair of writers as the child's stdout and
stderr. This works because `PipeWriter` implements `Into<Stdio>`.
Then we can read interleaved output from the pipe reader. This
example is deadlock-free, but note the comment about closing the
writers.

```rust
// We're going to spawn a child process that prints "foo" to stdout
// and "bar" to stderr, and we'll combine these into a single pipe.
let mut command = std::process::Command::new("python");
command.args(&["-c", r#"
import sys
sys.stdout.write("foo")
sys.stdout.flush()
sys.stderr.write("bar")
sys.stderr.flush()
"#]);

// Here's the interesting part. Open a pipe, clone its writer, and
// set that pair of writers as the child's stdout and stderr.
let (mut reader, writer) = os_pipe::pipe()?;
let writer_clone = writer.try_clone()?;
command.stdout(writer);
command.stderr(writer_clone);

// Now start the child process running.
let mut handle = command.spawn()?;

// Avoid a deadlock! This parent process is still holding open pipe
// writers (inside the Command object), and we have to close those
// before we read. Here we do this by dropping the Command object.
drop(command);

// Finally we can read all the output and clean up the child.
let mut output = String::new();
reader.read_to_string(&mut output)?;
handle.wait()?;
assert_eq!(output, "foobar");
```

Note that the [`duct`](https://github.com/oconnor663/duct.rs) crate
can reproduce the example above in a single line of code, with no
risk of deadlocks or of leaking zombie children.
