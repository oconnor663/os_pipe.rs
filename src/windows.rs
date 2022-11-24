use crate::PipeReader;
use crate::PipeWriter;
use std::fs::File;
use std::io;
use std::os::windows::prelude::*;
use std::ptr;
use windows_sys::Win32::Foundation::{
    DuplicateHandle, BOOL, DUPLICATE_SAME_ACCESS, HANDLE, INVALID_HANDLE_VALUE,
};
use windows_sys::Win32::System::Pipes::CreatePipe;
use windows_sys::Win32::System::Threading::GetCurrentProcess;

pub(crate) fn pipe() -> io::Result<(PipeReader, PipeWriter)> {
    let mut read_pipe = INVALID_HANDLE_VALUE;
    let mut write_pipe = INVALID_HANDLE_VALUE;

    let ret = unsafe {
        // NOTE: These pipes do not support IOCP. We might want to emulate
        // anonymous pipes with CreateNamedPipe, as Rust's stdlib does.
        CreatePipe(&mut read_pipe, &mut write_pipe, ptr::null_mut(), 0)
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

pub(crate) fn dup<F: AsRawHandle>(wrapper: &F) -> io::Result<File> {
    // We rely on ("abuse") std::fs::File for a lot of descriptor/handle
    // operations. (For example, setting F_DUPFD_CLOEXEC on Unix is a
    // compatibility mess.) However, in the particular case of try_clone on
    // Windows, the standard library has a bug where duplicated handles end up
    // inheritable when they shouldn't be. See
    // https://github.com/rust-lang/rust/pull/65316. This leads to races where
    // child processes can inherit each other's handles, which tends to cause
    // deadlocks when the handle in question is a stdout pipe. To get that
    // right, we explicitly make the necessary system calls here, just like
    // libstd apart from that one flag.
    // TODO: The fix for this issue shipped in Rust 1.40 (December 2019). When
    // we bump the MSRV past that point, we can go ahead and delete this
    // workaround. Until then, no rush.
    let source_handle = wrapper.as_raw_handle() as HANDLE;
    let desired_access = 0; // Ignored because of DUPLICATE_SAME_ACCESS.
    let inherit_handle = false as BOOL; // <-- Libstd sets this to true!
    let options = DUPLICATE_SAME_ACCESS;
    let mut duplicated_handle = 0 as HANDLE;
    let ret = unsafe {
        let current_process = GetCurrentProcess();
        DuplicateHandle(
            current_process,
            source_handle,
            current_process,
            &mut duplicated_handle,
            desired_access,
            inherit_handle,
            options,
        )
    };
    if ret == 0 {
        Err(io::Error::last_os_error())
    } else {
        unsafe { Ok(File::from_raw_handle(duplicated_handle as RawHandle)) }
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

#[cfg(feature = "io_safety")]
impl From<PipeReader> for OwnedHandle {
    fn from(reader: PipeReader) -> Self {
        reader.0.into()
    }
}

#[cfg(feature = "io_safety")]
impl AsHandle for PipeReader {
    fn as_handle(&self) -> BorrowedHandle<'_> {
        self.0.as_handle()
    }
}

#[cfg(feature = "io_safety")]
impl From<OwnedHandle> for PipeReader {
    fn from(handle: OwnedHandle) -> Self {
        PipeReader(handle.into())
    }
}

#[cfg(feature = "io_safety")]
impl From<PipeWriter> for OwnedHandle {
    fn from(writer: PipeWriter) -> Self {
        writer.0.into()
    }
}

#[cfg(feature = "io_safety")]
impl AsHandle for PipeWriter {
    fn as_handle(&self) -> BorrowedHandle<'_> {
        self.0.as_handle()
    }
}

#[cfg(feature = "io_safety")]
impl From<OwnedHandle> for PipeWriter {
    fn from(handle: OwnedHandle) -> Self {
        PipeWriter(handle.into())
    }
}
