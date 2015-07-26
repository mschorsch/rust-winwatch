// Copyright 2015 Matthias Schorsch
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//uses
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

// reexports
pub use ::libc::{c_void, LPDWORD, DWORD, LPCWSTR, BOOL, HANDLE, SECURITY_ATTRIBUTES, LPOVERLAPPED, TRUE, FALSE};

#[allow(non_camel_case_types)]
pub type LPOVERLAPPED_COMPLETION_ROUTINE = Option<unsafe extern "system" fn (
    dwErrorCode: DWORD, dwNumberOfBytesTransfered: DWORD, lpOverlapped: LPOVERLAPPED,
)>;

#[link(name = "kernel32")]
extern "system" {

    pub fn CreateFileW(lpFileName: LPCWSTR, dwDesiredAccess: DWORD, dwShareMode: DWORD,
                       lpSecurityAttributes: *mut SECURITY_ATTRIBUTES, dwCreationDisposition: DWORD,
                       dwFlagsAndAttributes: DWORD, hTemplateFile: HANDLE) -> HANDLE;    

    pub fn CloseHandle(hObject: HANDLE) -> BOOL;

    pub fn ReadDirectoryChangesW(hDirectory: HANDLE, lpBuffer: *mut c_void, nBufferLength: DWORD, bWatchSubtree: BOOL,
                                 dwNotifyFilter: DWORD, lpBytesReturned: LPDWORD, lpOverlapped: LPOVERLAPPED,
                                 lpCompletionRoutine: LPOVERLAPPED_COMPLETION_ROUTINE) -> BOOL;

    pub fn FindFirstChangeNotificationW(lpPathName: LPCWSTR, bWatchSubtree: BOOL, dwNotifyFilter: DWORD) -> HANDLE;

    pub fn FindNextChangeNotification(hChangeHandle: HANDLE) -> BOOL;

    pub fn FindCloseChangeNotification(hChangeHandle: HANDLE) -> BOOL;

    pub fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD;

    pub fn GetLastError() -> DWORD;
}

// LPCWSTR conversion
pub fn to_lpcwstr<S: AsRef<OsStr> + ?Sized>(s: &S) -> LPCWSTR {
    OsStr::new(s).encode_wide().collect::<Vec<u16>>().as_ptr()
}

// BOOL -> bool conversion
pub fn to_bool(b: BOOL) -> bool {
    match b {
        TRUE => true,
        FALSE => false,
        _ => unreachable!()
    }
}

// bool -> BOOL conversion
pub fn from_bool(b: bool) -> BOOL {
    match b {
        true => TRUE,
        false => FALSE,
    }
}

pub fn get_last_error() -> DWORD {
    unsafe {
        GetLastError()
    }
}