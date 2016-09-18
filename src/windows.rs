extern crate winapi;
extern crate kernel32;

use std::fs::File;
use std::io;
use std::os::windows::prelude::*;
use std::process::Stdio;
use std::mem;
use std::ptr;

use Pair;
use ParentHandle;

pub fn stdio_from_file(file: File) -> Stdio {
    unsafe { Stdio::from_raw_handle(file.into_raw_handle()) }
}

pub fn pipe() -> io::Result<Pair> {
    let mut read_pipe: winapi::HANDLE = ptr::null_mut();
    let mut write_pipe: winapi::HANDLE = ptr::null_mut();

    let ret = unsafe {
        // TODO: These pipes do not support IOCP. We might want to emulate anonymous pipes with
        // CreateNamedPipe, as Rust's stdlib does.
        kernel32::CreatePipe(&mut read_pipe as winapi::PHANDLE,
                             &mut write_pipe as winapi::PHANDLE,
                             ptr::null_mut(),
                             0)
    };

    if ret == 0 {
        Err(io::Error::last_os_error())
    } else {
        unsafe {
            Ok(Pair {
                read: File::from_raw_handle(read_pipe),
                write: File::from_raw_handle(write_pipe),
            })
        }
    }
}

pub fn parent_handle_to_stdio(parent_handle: ParentHandle) -> io::Result<Stdio> {
    let windows_handle = match parent_handle {
        ParentHandle::Stdin => winapi::STD_INPUT_HANDLE,
        ParentHandle::Stdout => winapi::STD_OUTPUT_HANDLE,
        ParentHandle::Stderr => winapi::STD_ERROR_HANDLE,
    };
    dup_std_handle(windows_handle)
}

// adapted from src/libstd/sys/windows/stdio.rs
fn dup_std_handle(which: winapi::DWORD) -> io::Result<Stdio> {
    let handle = unsafe { kernel32::GetStdHandle(which) };
    if handle == winapi::INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    if handle.is_null() {
        return Err(io::Error::new(io::ErrorKind::Other,
                                  "no stdio handle available for this process"));
    }
    // This handle is *not* a dup. It's just a copy of the global stdin/stdout/stderr handle, and
    // we need to dup it ourselves. The simplest way to do that is File::try_clone(), but we need
    // to make sure that the file is never dropped.
    let temp_file = unsafe { File::from_raw_handle(handle) };
    let dup_result = temp_file.try_clone();  // No short-circuit here!
    mem::forget(temp_file);  // Avoid closing the global handle.
    dup_result.map(stdio_from_file)
}
