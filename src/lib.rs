// TODO: Figure out why including lazy_static breaks the Windows build.
#[cfg(not(windows))]
#[macro_use]
extern crate lazy_static;

// TODO: Figure out why Windows things we're depending on the unstable libc.
#[cfg(not(windows))]
extern crate libc;

#[cfg(not(windows))]
#[path = "unix.rs"]
mod sys;
#[cfg(windows)]
#[path = "windows.rs"]
mod sys;

use std::fs::File;

pub use sys::pipe;
pub use sys::stdio_from_file;
pub use sys::dup_stdin;
pub use sys::dup_stdout;
pub use sys::dup_stderr;

pub struct Pair {
    pub read: File,
    pub write: File,
}

#[cfg(test)]
mod tests {
    use std::io::prelude::*;
    use std::path::Path;
    use std::process;
    use std::thread;
    use ::Pair;

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
        let cat_dir = Path::new(".").join("test").join("cat");
        let mut child = process::Command::new("cargo")
            .arg("run")
            .arg("-q")
            .current_dir(&cat_dir)
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
    fn test_duped_handles() {
        // Create pipes for a child process.
        let mut input_pipe = ::pipe().unwrap();
        let child_stdin = ::stdio_from_file(input_pipe.read);

        // Write input. This shouldn't block because it's small. Then close the write end, or else
        // the child will hang.
        input_pipe.write.write_all(b"quack").unwrap();
        drop(input_pipe.write);

        // Spawn the child and read its output. This is a tee program that copies its input to both
        // stdout and stderr. It depends on os_pipe itself, and uses the dup_* handles for all of
        // its IO.
        let tee_dir = Path::new(".").join("test").join("tee");
        let output = process::Command::new("cargo")
            .arg("run")
            .arg("-q")
            .current_dir(&tee_dir)
            .stdin(child_stdin)
            .output()
            .unwrap();

        // Check for a clean exit.
        assert!(output.status.success(),
                "child process returned {:#?}",
                output);

        // Confirm that we got the right bytes.
        assert_eq!(b"quack", &*output.stdout);
        assert_eq!(b"quack", &*output.stderr);
    }
}
