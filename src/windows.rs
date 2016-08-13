#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
use std::fs::File;
use std::io;
use std::os::raw::{c_int, c_void, c_ulong};
use std::os::windows::io::{FromRawHandle, IntoRawHandle};
use std::process::Stdio;
use std::ptr;

use ::Pair;

pub fn stdio_from_file(file: File) -> Stdio {
    unsafe { Stdio::from_raw_handle(file.into_raw_handle()) }
}

type DWORD = c_ulong;
type HANDLE = LPVOID;
type PHANDLE = *mut HANDLE;
type BOOL = c_int;
type WCHAR = u16;
type LPCWSTR = *const WCHAR;
type LPVOID = *mut c_void;
type LPSECURITY_ATTRIBUTES = *mut SECURITY_ATTRIBUTES;

#[repr(C)]
struct SECURITY_ATTRIBUTES {
    nLength: DWORD,
    lpSecurityDescriptor: LPVOID,
    bInheritHandle: BOOL,
}

extern "system" {
    fn CreatePipe(hReadPipe: PHANDLE,
                  hWritePipe: PHANDLE,
                  nSize: DWORD,
                  lpPipeAttributes: LPSECURITY_ATTRIBUTES)
                  -> BOOL;
}

pub fn pipe() -> io::Result<Pair> {
    let mut readPipe: HANDLE = ptr::null_mut();
    let mut writePipe: HANDLE = ptr::null_mut();

    let ret = unsafe {
        CreatePipe(&mut readPipe as PHANDLE,
                   &mut writePipe as PHANDLE,
                   0,
                   ptr::null_mut())
    };

    if ret == 0 {
        Err(io::Error::last_os_error())
    } else {
        unsafe {
            Ok(Pair {
                read: File::from_raw_handle(readPipe),
                write: File::from_raw_handle(writePipe),
            })
        }
    }
}
