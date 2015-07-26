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

// import modules
use ffi::*;
use types::{FileAction, FileNotifyChange};
use errors;

use ::libc::{FILE_SHARE_WRITE, FILE_SHARE_READ, FILE_SHARE_DELETE, OPEN_EXISTING, FILE_FLAG_BACKUP_SEMANTICS};

// std uses
use std::path::Path;
use std::fmt;
use std::ptr;

// consts
const FILE_LIST_DIRECTORY: DWORD = 0x0001;

#[derive(Debug,Clone)]
pub struct FileNotifyInformation {
    action: FileAction,
    filename: String,
}

impl fmt::Display for FileNotifyInformation {
    
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} -> {}", self.action, self.filename)
    }
}

#[derive(Debug)]
pub struct WinWatch {

    h_directory: HANDLE,
    watch_subdirs: bool,
    buffer_size: u32,

    results_arr: Box<Vec<u16>>,
    dw_notify_filter: DWORD,
}

impl Drop for WinWatch {

    fn drop(&mut self) {
        close_dir_handle(self.h_directory);
    }
}

impl WinWatch {
    
    pub fn new(directory: &Path, notify_changes: Box<Vec<FileNotifyChange>>, watch_subdirs: bool, buffer_size: u32) -> WinWatch {
        let h_directory = open_dir_handle(directory); //TODO: check errors
        
        let mut results_arr: Vec<u16> = Vec::with_capacity(buffer_size as usize);
        unsafe {results_arr.set_len(buffer_size as usize)};

        WinWatch {
            h_directory: h_directory,
            watch_subdirs: watch_subdirs,
            buffer_size: buffer_size,
            results_arr: Box::new(results_arr),
            dw_notify_filter: FileNotifyChange::as_u32(notify_changes),
        }
    }

    pub fn watch(&mut self) -> Result<Box<Vec<FileNotifyInformation>>, errors::Error> {
        read_directory_changes(self.h_directory, &mut *self.results_arr, self.buffer_size, self.dw_notify_filter, self.watch_subdirs)
    }
}

fn open_dir_handle(directory: &Path) -> HANDLE {
    let lp_filename = to_lpcwstr(directory);
    let dw_desired_access = FILE_LIST_DIRECTORY;
    let dw_share_mode = FILE_SHARE_WRITE | FILE_SHARE_READ | FILE_SHARE_DELETE;
    let lp_security_attributes: *mut SECURITY_ATTRIBUTES = ptr::null_mut();
    let dw_creation_disposition = OPEN_EXISTING;
    let dw_flags_and_attributes = FILE_FLAG_BACKUP_SEMANTICS;
    let h_template_file: HANDLE = ptr::null_mut();

    unsafe {
        CreateFileW(lp_filename, dw_desired_access, 
            dw_share_mode, lp_security_attributes, dw_creation_disposition, 
            dw_flags_and_attributes, h_template_file)        
    }
}

fn close_dir_handle(handle: HANDLE) -> bool {
    to_bool(unsafe {
        CloseHandle(handle)
    })
}

fn read_directory_changes(h_directory: HANDLE, result_vec: &mut [u16], buffer_size: DWORD,
                          dw_notify_filter: DWORD, watch_subdirs: bool) -> Result<Box<Vec<FileNotifyInformation>>, errors::Error> {
    //
    // prepare parameters
    let handle = h_directory as *mut c_void;
    let lp_buffer = result_vec.as_mut_ptr() as *mut c_void;
    let n_buffer_length: DWORD = buffer_size * 2; // in bytes

    let b_watch_subtree = from_bool(watch_subdirs);
    let mut lp_bytes_returned: DWORD = 0;

    //overlapped io + callback
    let lp_overlapped: LPOVERLAPPED = ptr::null_mut(); // c_null
    let lp_completion_routine: LPOVERLAPPED_COMPLETION_ROUTINE = Option::None; /*unsafe {
        std::mem::transmute(ptr::null_mut::<LPOVERLAPPED_COMPLETION_ROUTINE>())
    };*/

    //
    // watch
    let has_result: bool = to_bool(unsafe {
        ReadDirectoryChangesW(handle,
                              lp_buffer,
                              n_buffer_length, 
                              b_watch_subtree, 
                              dw_notify_filter, 
                              &mut lp_bytes_returned, 
                              lp_overlapped,
                              lp_completion_routine)
    });

    //
    // results
    if has_result {
        Result::Ok(from_u16_slice(result_vec))
    } else {
        let error_desc = format!("Failure detected with system error code {}", get_last_error());
        Result::Err(errors::Error::new(error_desc))
    }
}

fn from_u16_slice(v: &[u16]) -> Box<Vec<FileNotifyInformation>> {
    let mut result:Vec<FileNotifyInformation> = Vec::new();
    let mut offset: usize = 0;
    loop {
        let (next_entry_offset, fni) = to_file_notify_information(v, offset);
        result.push(fni);

        // check for 0.
        // 0 indicates that this is the last record
        if next_entry_offset == 0 {
            break;
        }
        offset += next_entry_offset as usize
    }
    Box::new(result)
}

fn to_file_notify_information(v: &[u16], offset: usize) -> (u32, FileNotifyInformation) {
    let next_entry_offset_in_u16 = to_u32le(v, offset) / 2;
    let action = to_u32le(v, offset + 2);
    let file_name_length_in_bytes = to_u32le(v, offset + 4) as usize;
    let filename = to_filename(v, offset + 6, file_name_length_in_bytes);

    let fni = FileNotifyInformation {
        action: FileAction::from_u32(action),
        filename: filename,
    };

    (next_entry_offset_in_u16, fni)
}

fn to_filename(v: &[u16], offset: usize, file_name_length_in_bytes: usize) -> String {
    let result = &v[offset .. offset + (file_name_length_in_bytes / 2)];
    String::from_utf16(result).unwrap()
}

fn to_u32le(v: &[u16], offset: usize) -> u32 {
    (v[offset + 1] as u32) << 16 | (v[offset] as u32) // little endian
}