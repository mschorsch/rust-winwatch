//uses
use ::libc::{CreateFileW, CloseHandle, GetLastError};
use ::libc::types::os::arch::extra::{DWORD, WCHAR, LPCWSTR, BOOL, HANDLE, SECURITY_ATTRIBUTES};
use ::libc::consts::os::extra as winconsts;

use winapi::winnt;

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

use std::path::Path;
use std::ptr;

//
// LPCWSTR conversion
//
pub fn to_lpcwstr<S: AsRef<OsStr> + ?Sized>(s: &S) -> LPCWSTR {
    OsStr::new(s).encode_wide().collect::<Vec<WCHAR>>().as_ptr()
}

//
// BOOL -> bool conversion
//
pub fn to_bool(b: BOOL) -> bool {
    match b {
        winconsts::TRUE => true,
        winconsts::FALSE => false,
        _ => unreachable!()
    }
}

//
// bool -> BOOL conversion
//
pub fn from_bool(b: bool) -> BOOL {
    match b {
        true => winconsts::TRUE,
        false => winconsts::FALSE,
    }
}

pub fn to_filename(v: &[u16], offset: usize, file_name_length_in_bytes: usize) -> String {
    let result = &v[offset .. offset + (file_name_length_in_bytes / 2)];
    String::from_utf16(result).unwrap()
}

pub fn to_u32le(v: &[u16], offset: usize) -> u32 {
    (v[offset + 1] as u32) << 16 | (v[offset] as u32) // little endian
}

pub fn open_winhandle(directory: &Path) -> HANDLE {
    let lp_filename = to_lpcwstr(directory);
    let dw_desired_access = winnt::FILE_LIST_DIRECTORY;
    let dw_share_mode = winconsts::FILE_SHARE_WRITE | winconsts::FILE_SHARE_READ | winconsts::FILE_SHARE_DELETE;
    let lp_security_attributes: *mut SECURITY_ATTRIBUTES = ptr::null_mut();
    let dw_creation_disposition = winconsts::OPEN_EXISTING;
    let dw_flags_and_attributes = winconsts::FILE_FLAG_BACKUP_SEMANTICS;
    let h_template_file: HANDLE = ptr::null_mut();

    unsafe {
        CreateFileW(lp_filename, dw_desired_access, 
            dw_share_mode, lp_security_attributes, dw_creation_disposition, 
            dw_flags_and_attributes, h_template_file)        
    }
}

pub fn close_winhandle(handle: HANDLE) -> bool {
    let result = unsafe { CloseHandle(handle) };
    to_bool(result)
}

pub fn get_last_error() -> DWORD {
    unsafe {GetLastError()}
}