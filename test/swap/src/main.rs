#![deny(warnings)]

extern crate os_pipe;

use std::env::args_os;
use std::ffi::OsString;
use std::process::Command;

use os_pipe::ParentHandle;

fn main() {
    let stdin = ParentHandle::Stdin.to_stdio().unwrap();
    let stdout = ParentHandle::Stdout.to_stdio().unwrap();
    let stderr = ParentHandle::Stderr.to_stdio().unwrap();

    let args: Vec<OsString> = args_os().collect();
    let mut child = Command::new(&args[1]);
    child.args(&args[2..]);

    // Swap stdout and stderr in the child. Set stdin too, just for testing,
    // though this should be the same as the default behavior.
    child.stdin(stdin);
    child.stdout(stderr);
    child.stderr(stdout);

    // Run the child. This method is kind of confusingly named...
    child.status().unwrap();
}
