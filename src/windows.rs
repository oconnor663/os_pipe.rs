extern crate winapi;

use std::fs::File;
use std::io;
use std::os::windows::prelude::*;
use std::process::Stdio;
use std::ptr;

use self::winapi::shared::minwindef::DWORD;
use self::winapi::shared::ntdef::{HANDLE, PHANDLE};
use self::winapi::um::{handleapi, namedpipeapi, processenv, winbase};

use PipeReader;
use PipeWriter;
use IntoStdio;

pub fn pipe() -> io::Result<(PipeReader, PipeWriter)> {
    let mut read_pipe: HANDLE = ptr::null_mut();
    let mut write_pipe: HANDLE = ptr::null_mut();

    let ret = unsafe {
        // TODO: These pipes do not support IOCP. We might want to emulate anonymous pipes with
        // CreateNamedPipe, as Rust's stdlib does.
        namedpipeapi::CreatePipe(
            &mut read_pipe as PHANDLE,
            &mut write_pipe as PHANDLE,
            ptr::null_mut(),
            0,
        )
    };

    if ret == 0 {
        Err(io::Error::last_os_error())
    } else {
        unsafe {
            Ok((
                PipeReader::from_raw_handle(read_pipe as _),
                PipeWriter::from_raw_handle(write_pipe as _),
            ))
        }
    }
}

pub fn parent_stdin() -> io::Result<Stdio> {
    dup_std_handle(winbase::STD_INPUT_HANDLE)
}

pub fn parent_stdout() -> io::Result<Stdio> {
    dup_std_handle(winbase::STD_OUTPUT_HANDLE)
}

pub fn parent_stderr() -> io::Result<Stdio> {
    dup_std_handle(winbase::STD_ERROR_HANDLE)
}

// adapted from src/libstd/sys/windows/stdio.rs
fn dup_std_handle(which: DWORD) -> io::Result<Stdio> {
    let handle = unsafe { processenv::GetStdHandle(which) };
    if handle == handleapi::INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    if handle.is_null() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "no stdio handle available for this process",
        ));
    }
    // This handle is *not* a dup. It's just a copy of the global stdin/stdout/stderr handle, and
    // we need to dup it ourselves. The simplest way to do that is File::try_clone(), but we need
    // to make sure that the file is never dropped.
    let temp_file = unsafe { File::from_raw_handle(handle as _) };
    let dup_result = temp_file.try_clone(); // No short-circuit here!
    temp_file.into_raw_handle(); // Prevent closing handle on drop().
    dup_result.map(File::into_stdio)
}

impl<T: IntoRawHandle> IntoStdio for T {
    fn into_stdio(self) -> Stdio {
        let handle = self.into_raw_handle();
        unsafe { Stdio::from_raw_handle(handle) }
    }
}

impl IntoRawHandle for PipeReader {
    fn into_raw_handle(self) -> RawHandle {
        self.0.into_raw_handle()
    }
}

impl AsRawHandle for PipeReader {
    fn as_raw_handle(&self) -> RawHandle {
        self.0.as_raw_handle()
    }
}

impl FromRawHandle for PipeReader {
    unsafe fn from_raw_handle(handle: RawHandle) -> PipeReader {
        PipeReader(File::from_raw_handle(handle))
    }
}

impl IntoRawHandle for PipeWriter {
    fn into_raw_handle(self) -> RawHandle {
        self.0.into_raw_handle()
    }
}

impl AsRawHandle for PipeWriter {
    fn as_raw_handle(&self) -> RawHandle {
        self.0.as_raw_handle()
    }
}

impl FromRawHandle for PipeWriter {
    unsafe fn from_raw_handle(handle: RawHandle) -> PipeWriter {
        PipeWriter(File::from_raw_handle(handle))
    }
}
