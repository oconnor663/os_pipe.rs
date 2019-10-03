use std::fs::File;
use std::io;
use std::mem::ManuallyDrop;
use std::os::windows::prelude::*;
use std::ptr;

use winapi::shared::minwindef::DWORD;
use winapi::shared::ntdef::{HANDLE, PHANDLE};
use winapi::um::namedpipeapi;

use crate::PipeReader;
use crate::PipeWriter;

pub(crate) fn pipe() -> io::Result<(PipeReader, PipeWriter)> {
    let mut read_pipe: HANDLE = ptr::null_mut();
    let mut write_pipe: HANDLE = ptr::null_mut();

    let ret = unsafe {
        // NOTE: These pipes do not support IOCP. We might want to emulate
        // anonymous pipes with CreateNamedPipe, as Rust's stdlib does.
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

pub(crate) fn dup<T: AsRawHandle>(wrapper: T) -> io::Result<File> {
    let handle = wrapper.as_raw_handle();
    let temp_file = ManuallyDrop::new(unsafe { File::from_raw_handle(handle) });
    temp_file.try_clone()
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
