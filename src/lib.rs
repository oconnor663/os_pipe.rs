use std::fs::File;

pub struct PipePair {
    pub read: File,
    pub write: File,
}

pub use sys::pipe;

#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
mod sys {
    use std::fs::File;
    use std::io;
    use std::os::raw::{c_int, c_void, c_ulong};
    use std::os::windows::io::FromRawHandle;
    use std::ptr;

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

    pub fn pipe() -> io::Result<::PipePair> {
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
                Ok(::PipePair {
                    read: File::from_raw_handle(readPipe),
                    write: File::from_raw_handle(writePipe),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::pipe;
    use std::io::prelude::*;

    #[test]
    fn pipe_some_data() {
        let mut pair = pipe().unwrap();
        pair.write.write_all(b"some stuff").unwrap();
        drop(pair.write);
        let mut out = String::new();
        pair.read.read_to_string(&mut out).unwrap();
        assert_eq!(out, "some stuff");
    }
}
