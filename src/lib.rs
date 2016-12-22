//! A cross-platform library for opening OS pipes.
//!
//! The standard library uses pipes to read output from child processes,
//! but it doesn't expose a way to create them directly. This crate
//! fills that gap with the `pipe` function. It also includes some
//! helpers for passing pipes to the `std::process::Command` API.
//!
//! `os_pipe` was originally built to support the higher-level
//! [`duct`](https://crates.io/crates/duct) library. If you need to do
//! fancy things with child processes, take a look at `duct` first. It
//! can run the following example in a single line.
//!
//! # Example
//!
//! Join the stdout and stderr of a child process into a single stream,
//! and read it. To do that we open a pipe, duplicate its write end, and
//! pass those writers as the child's stdout and stderr. Then we can
//! read combined output from the read end of the pipe. We have to be
//! careful to close the write ends first though, or reading will block
//! waiting for EOF.
//!
//! ```rust
//! use os_pipe::{pipe, Pair, stdio_from_file};
//! use std::io::prelude::*;
//! use std::process::Command;
//!
//! // This command prints "foo" to stdout and "bar" to stderr. It
//! // works on both Unix and Windows, though there are whitespace
//! // differences that we'll account for at the bottom.
//! let shell_command = "echo foo && echo bar >&2";
//!
//! // Ritual magic to run shell commands on different platforms.
//! let (shell, flag) = if cfg!(windows) { ("cmd.exe", "/C") } else { ("sh", "-c") };
//!
//! let mut child = Command::new(shell);
//! child.arg(flag);
//! child.arg(shell_command);
//!
//! // Here's the interesting part. Open a pipe, copy its write end, and
//! // give both copies to the child.
//! let Pair{mut read, write} = pipe().unwrap();
//! let write_copy = write.try_clone().unwrap();
//! child.stdout(stdio_from_file(write));
//! child.stderr(stdio_from_file(write_copy));
//!
//! // Now start the child running.
//! let mut handle = child.spawn().unwrap();
//!
//! // Very important when using pipes: This parent process is still
//! // holding its copies of the write ends, and we have to close them
//! // before we read, otherwise the read end will never report EOF. The
//! // Command object owns the writers now, and dropping it closes them.
//! drop(child);
//!
//! // Finally we can read all the output and clean up the child.
//! let mut output = String::new();
//! read.read_to_string(&mut output).unwrap();
//! handle.wait().unwrap();
//! assert!(output.split_whitespace().eq(vec!["foo", "bar"]));
//! ```

use std::fs::File;
use std::io;
use std::process::Stdio;

/// The read and write ends of the pipe created by `pipe`.
pub struct Pair {
    pub read: File,
    pub write: File,
}

/// Open a new pipe.
///
/// This corresponds to the `pipe2` library call on Posix and the
/// `CreatePipe` library call on Windows (though these implementation
/// details might change). Pipes are non-inheritable, so new child
/// processes won't receive a copy of them unless they're explicitly
/// passed as stdin/stdout/stderr.
pub fn pipe() -> io::Result<Pair> {
    sys::pipe()
}

/// Get a duplicated copy of the current process's standard input pipe.
///
/// This isn't intended for doing IO, rather it's in a form that can be
/// passed directly to the `std::process::Command` API.
pub fn parent_stdin() -> io::Result<Stdio> {
    sys::parent_stdin()
}

/// Get a duplicated copy of the current process's standard output pipe.
///
/// This isn't intended for doing IO, rather it's in a form that can be
/// passed directly to the `std::process::Command` API.
pub fn parent_stdout() -> io::Result<Stdio> {
    sys::parent_stdout()
}

/// Get a duplicated copy of the current process's standard error pipe.
///
/// This isn't intended for doing IO, rather it's in a form that can be
/// passed directly to the `std::process::Command` API.
pub fn parent_stderr() -> io::Result<Stdio> {
    sys::parent_stderr()
}

/// Safely convert a `std::fs::File` to a `std::process::Stdio`.
///
/// The standard library supports this conversion, but it requires
/// platform-specific traits and takes an `unsafe` call. This is a safe
/// wrapper for convenience. Currently there's not really such a thing
/// as a "closed file" in Rust, since closing requires dropping, but if
/// Rust ever introduces closed files in the future this function will
/// panic on them.
pub fn stdio_from_file(file: File) -> Stdio {
    sys::stdio_from_file(file)
}

#[cfg(not(windows))]
#[path = "unix.rs"]
mod sys;
#[cfg(windows)]
#[path = "windows.rs"]
mod sys;

#[cfg(test)]
mod tests {
    use std::io::prelude::*;
    use std::env::consts::EXE_EXTENSION;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::sync::{Once, ONCE_INIT};
    use std::thread;
    use ::Pair;

    fn path_to_exe(name: &str) -> PathBuf {
        // This project defines some associated binaries for testing, and we shell out to them in
        // these tests. `cargo test` doesn't automatically build associated binaries, so this
        // function takes care of building them explicitly, with the right debug/release flavor.
        static CARGO_BUILD_ONCE: Once = ONCE_INIT;
        CARGO_BUILD_ONCE.call_once(|| {
            let mut build_command = Command::new("cargo");
            build_command.args(&["build", "--quiet"]);
            if !cfg!(debug_assertions) {
                build_command.arg("--release");
            }
            let build_status = build_command.status().unwrap();
            assert!(build_status.success(),
                    "Cargo failed to build associated binaries.");
        });
        let flavor = if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        };
        Path::new("target").join(flavor).join(name).with_extension(EXE_EXTENSION)
    }

    #[test]
    fn test_pipe_some_data() {
        let mut pair = ::pipe().unwrap();
        // A small write won't fill the pipe buffer, so it won't block this thread.
        pair.write.write_all(b"some stuff").unwrap();
        drop(pair.write);
        let mut out = String::new();
        pair.read.read_to_string(&mut out).unwrap();
        assert_eq!(out, "some stuff");
    }

    #[test]
    fn test_pipe_no_data() {
        let mut pair = ::pipe().unwrap();
        drop(pair.write);
        let mut out = String::new();
        pair.read.read_to_string(&mut out).unwrap();
        assert_eq!(out, "");
    }

    #[test]
    fn test_pipe_a_megabyte_of_data_from_another_thread() {
        let data = vec![0xff; 1_000_000];
        let data_copy = data.clone();
        let Pair { mut read, mut write } = ::pipe().unwrap();
        let joiner = thread::spawn(move || {
            write.write_all(&data_copy).unwrap();
            // This drop happens automatically, so writing it out here is mostly
            // just for clarity. For what it's worth, it also guards against
            // accidentally forgetting to drop if we switch to scoped threads or
            // something like that and change this to a non-moving closure. The
            // explicit drop forces `write` to move.
            drop(write);
        });
        let mut out = Vec::new();
        read.read_to_end(&mut out).unwrap();
        joiner.join().unwrap();
        assert_eq!(out, data);
    }

    #[test]
    fn test_pipes_are_not_inheritable() {
        // Create pipes for a child process.
        let mut input_pipe = ::pipe().unwrap();
        let mut output_pipe = ::pipe().unwrap();
        let child_stdin = ::stdio_from_file(input_pipe.read);
        let child_stdout = ::stdio_from_file(output_pipe.write);

        // Spawn the child. Note that this temporary Command object takes ownership of our copies
        // of the child's stdin and stdout, and then closes them immediately when it drops. That
        // stops us from blocking our own read below. We use our own simple implementation of cat
        // for compatibility with Windows.
        let mut child = Command::new(path_to_exe("cat"))
            .stdin(child_stdin)
            .stdout(child_stdout)
            .spawn()
            .unwrap();

        // Write to the child's stdin. This is a small write, so it shouldn't block.
        input_pipe.write.write_all(b"hello").unwrap();
        drop(input_pipe.write);

        // Read from the child's stdout. If this child has accidentally inherited the write end of
        // its own stdin, then it will never exit, and this read will block forever. That's the
        // what this test is all about.
        let mut output = Vec::new();
        output_pipe.read.read_to_end(&mut output).unwrap();
        child.wait().unwrap();

        // Confirm that we got the right bytes.
        assert_eq!(b"hello", &*output);
    }

    #[test]
    fn test_parent_handles() {
        // This test invokes the `swap` test program, which uses parent_stdout() and
        // parent_stderr() to swap the outputs for another child that it spawns.

        // Create pipes for a child process.
        let mut input_pipe = ::pipe().unwrap();
        let child_stdin = ::stdio_from_file(input_pipe.read);

        // Write input. This shouldn't block because it's small. Then close the write end, or else
        // the child will hang.
        input_pipe.write.write_all(b"quack").unwrap();
        drop(input_pipe.write);

        // Use `swap` to run `cat`. `cat will read "quack" from stdin and write it to stdout. But
        // because we run it inside `swap`, that write should end up on stderr.
        let output = Command::new(path_to_exe("swap"))
            .arg(path_to_exe("cat"))
            .stdin(child_stdin)
            .output()
            .unwrap();

        // Check for a clean exit.
        assert!(output.status.success(),
                "child process returned {:#?}",
                output);

        // Confirm that we got the right bytes.
        assert_eq!(b"", &*output.stdout);
        assert_eq!(b"quack", &*output.stderr);
    }
}
