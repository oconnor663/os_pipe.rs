extern crate os_pipe;

use std::io::prelude::*;

fn main() {
    let mut stdin = os_pipe::dup_stdin().unwrap();
    let mut stdout = os_pipe::dup_stdout().unwrap();
    let mut stderr = os_pipe::dup_stderr().unwrap();

    let mut input = Vec::new();
    stdin.read_to_end(&mut input).unwrap();

    stdout.write_all(&input).unwrap();
    stderr.write_all(&input).unwrap();
}
