# os_pipe.rs [![Build Status](https://travis-ci.org/oconnor663/os_pipe.rs.svg?branch=master)](https://travis-ci.org/oconnor663/os_pipe.rs) [![Build status](https://ci.appveyor.com/api/projects/status/89o6o64nxfl80s78/branch/master?svg=true)](https://ci.appveyor.com/project/oconnor663/os-pipe-rs/branch/master)

A cross-platform Rust library for opening pipes, similar to `os.pipe` in
Python. This is basically a copy of
`rust/src/libstd/sys/{unix,windows}/pipe.rs`, with changes to make it
build with stable Rust.
