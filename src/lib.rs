//! A library for opening OS pipes, on both Windows and Posix. The
//! standard library uses pipes to read output from child processes, but
//! it doesn't expose a way to create them directly. This crate fills
//! that gap with the `pipe` function. It also includes some utilities
//! for passing pipes to `std::process::Command` API.

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
/// used for stdin/stdout/stderr.
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
