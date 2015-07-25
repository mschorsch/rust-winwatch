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

//external crates
extern crate kernel32;

use util;
use errors;
use types::{FileAction, FileNotifyChange};

// uses
use std::path::Path;
use std::fmt;
use std::ptr;
use std::ops::Drop;
use std::os::raw::{c_void};

use ::libc::types::os::arch::extra::{DWORD, HANDLE};
use ::winapi::minwinbase::{LPOVERLAPPED, LPOVERLAPPED_COMPLETION_ROUTINE};

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
        util::close_winhandle(self.h_directory);
    }
}

impl WinWatch {
    
    pub fn new(directory: &Path, notify_changes: Box<Vec<FileNotifyChange>>, watch_subdirs: bool, buffer_size: u32) -> WinWatch {
        let h_directory = util::open_winhandle(directory); //TODO: check errors
        
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

fn read_directory_changes(h_directory: HANDLE, result_vec: &mut [u16], buffer_size: DWORD,
                          dw_notify_filter: DWORD, watch_subdirs: bool) -> Result<Box<Vec<FileNotifyInformation>>, errors::Error> {
    //
    // prepare parameters
    let handle = h_directory as *mut c_void;
    let lp_buffer = result_vec.as_mut_ptr() as *mut c_void;
    let n_buffer_length: DWORD = buffer_size * 2; //in bytes

    let b_watch_subtree = util::from_bool(watch_subdirs);
    let mut lp_bytes_returned: DWORD = 0;

    //overlapped io + callback
    let lp_overlapped: LPOVERLAPPED = ptr::null_mut();
    let lp_completion_routine: LPOVERLAPPED_COMPLETION_ROUTINE = Option::None; /*unsafe {
        std::mem::transmute(ptr::null_mut::<LPOVERLAPPED_COMPLETION_ROUTINE>())
    };*/

    //
    // watch
    let has_result: bool = util::to_bool(unsafe {
        kernel32::ReadDirectoryChangesW(handle,
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
        let error_desc = format!("Failure detected with system error code {}", util::get_last_error());
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
    let next_entry_offset_in_u16 = util::to_u32le(v, offset) / 2;
    let action = util::to_u32le(v, offset + 2);
    let file_name_length_in_bytes = util::to_u32le(v, offset + 4) as usize;
    let filename = util::to_filename(v, offset + 6, file_name_length_in_bytes);

    let fni = FileNotifyInformation {
        action: FileAction::from_u32(action),
        filename: filename,
    };

    (next_entry_offset_in_u16, fni)
}