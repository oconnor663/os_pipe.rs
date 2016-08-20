extern crate os_pipe;

use std::env::args_os;
use std::process::Command;

fn main() {
    let stdout = os_pipe::dup_stdout().unwrap();
    let stderr = os_pipe::dup_stderr().unwrap();
    let mut args = args_os();

    // Create a child process using all args to this one. The first arg is our
    // own executable, so we skip past that.
    let mut child = Command::new(args.by_ref().skip(1).next().unwrap());
    for arg in args {
        child.arg(arg);
    }

    // Swap the child's outputs, relative to this process.
    child.stdout(stderr);
    child.stderr(stdout);

    // Run it!
    let status = child.status().unwrap();
    assert!(status.success(), "child exited with error {}", status);
}
