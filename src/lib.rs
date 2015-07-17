#![allow(dead_code)]

//external crates
extern crate kernel32;
extern crate winapi;
extern crate libc;

// mods
mod errors;
mod winutil;

// uses
use std::path::Path;
use std::fmt;
use std::ptr;
use std::ops::Drop;
use std::os::raw::{c_void};
use std::cell::RefCell;

use libc::types::os::arch::extra::{DWORD, HANDLE};

use winapi::minwinbase;

//
// FileNotifyChange
//

#[derive(Debug)]
pub enum FileNotifyChange {
    FileName = 0x00000001,
    DirName = 0x00000002,
    Attributes = 0x00000004,
    Size = 0x00000008,
    LastWrite = 0x00000010,
    LastAccess = 0x00000020,
    Creation = 0x00000040,
    Security = 0x00000100,    
}

impl FileNotifyChange {
    
    fn as_u32(filters: Box<Vec<FileNotifyChange>>) -> u32 {
        let mut result: u32 = 0;
        for f in filters.into_iter() {
            result |= f as u32;
        }
        result
    }
}

//
// FileAction
//

#[derive(Debug)]
pub enum FileAction {
    FileAdded = 0x00000001,
    FileRemoved = 0x00000002,
    FileModified = 0x00000003,
    FileRenamedOldName = 0x00000004,
    FileRenamedNewName = 0x00000005,
}

impl FileAction {

    fn from_u32(value: u32) -> FileAction {
        match value {
            1 => FileAction::FileAdded,
            2 => FileAction::FileRemoved,
            3 => FileAction::FileModified,
            4 => FileAction::FileRenamedOldName,
            5 => FileAction::FileRenamedNewName,
            _ => unreachable!(),
        }
    }
}

impl fmt::Display for FileAction {
    
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", *self)
    }
}

#[test]
fn test_display_fileaction() {
    assert_eq!(format!{"{}", FileAction::FileAdded}, "FileAdded");
}

#[test]
fn test_debug_fileaction() {
    assert_eq!(format!{"{:?}", FileAction::FileAdded}, "FileAdded");
}

//
// FileNotifyInformation
//

#[derive(Debug)]
pub struct FileNotifyInformation {
    action: FileAction,
    filename: String,
}

impl fmt::Display for FileNotifyInformation {
    
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} -> {}", self.action, self.filename)
    }
}

//
// DirWatch
//
pub fn sync(directory: &Path, notify_changes: Box<Vec<FileNotifyChange>>, watch_subdirs: bool, buffer_size: u32) -> DirWatch {
    DirWatch::new(directory, notify_changes, watch_subdirs, buffer_size)
}

pub struct DirWatch {

    h_directory: HANDLE,
    watch_subdirs: bool,
    buffer_size: u32,

    results_arr: RefCell<Vec<u16>>,
    dw_notify_filter: DWORD,
}

impl Drop for DirWatch {

    fn drop(&mut self) {
        winutil::close_winhandle(self.h_directory);
    }
}

impl DirWatch {
    
    fn new(directory: &Path, notify_changes: Box<Vec<FileNotifyChange>>, watch_subdirs: bool, buffer_size: u32) -> DirWatch {
        let h_directory = winutil::open_winhandle(directory); //TODO: check errors
        
        let mut results_arr: Vec<u16> = Vec::with_capacity(buffer_size as usize);
        unsafe {results_arr.set_len(buffer_size as usize)};

        DirWatch {
            h_directory: h_directory,
            watch_subdirs: watch_subdirs,
            buffer_size: buffer_size,
            results_arr: RefCell::new(results_arr),
            dw_notify_filter: FileNotifyChange::as_u32(notify_changes),
        }
    }

    pub fn watch(&self) -> Result<Box<Vec<FileNotifyInformation>>, errors::Error> {
        self.read_directory_changes()
    }

    fn read_directory_changes(&self) -> Result<Box<Vec<FileNotifyInformation>>, errors::Error> {
        //
        // prepare parameters
        let result_vec = &mut self.results_arr.borrow_mut();
        let lp_buffer = result_vec.as_mut_ptr() as *mut c_void;
        let n_buffer_length: DWORD = self.buffer_size * 2; //in bytes

        let b_watch_subtree = winutil::from_bool(self.watch_subdirs);
        let dw_notify_filter = self.dw_notify_filter;
        let mut lp_bytes_returned: DWORD = 0;

        //overlapped io + callback
        let lp_overlapped: minwinbase::LPOVERLAPPED = ptr::null_mut();
        let lp_completion_routine: minwinbase::LPOVERLAPPED_COMPLETION_ROUTINE = unsafe {
            std::mem::transmute(ptr::null_mut::<minwinbase::LPOVERLAPPED_COMPLETION_ROUTINE>())
        };

        //
        // watch
        let has_result: bool = winutil::to_bool(unsafe {
            kernel32::ReadDirectoryChangesW(self.h_directory as *mut c_void,
                                            lp_buffer, n_buffer_length, 
                                            b_watch_subtree, 
                                            dw_notify_filter, 
                                            &mut lp_bytes_returned, 
                                            lp_overlapped,
                                            lp_completion_routine)
        });

        //
        // results
        if has_result {
            Result::Ok(self.from_u16_slice(result_vec))
        } else {
            let error_desc = format!("Failure detected with system error code {}", winutil::get_last_error());
            Result::Err(errors::Error::new(error_desc))
        }
    }

    fn from_u16_slice(&self, v: &[u16]) -> Box<Vec<FileNotifyInformation>> {
        let mut result:Vec<FileNotifyInformation> = Vec::new();
        let mut offset: usize = 0;
        loop {
            let (next_entry_offset, fni) = self.to_file_notify_information(v, offset);
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

    fn to_file_notify_information(&self, v: &[u16], offset: usize) -> (u32, FileNotifyInformation) {
        let next_entry_offset_in_u16 = winutil::to_u32le(v, offset) / 2;
        let action = winutil::to_u32le(v, offset + 2);
        let file_name_length_in_bytes = winutil::to_u32le(v, offset + 4) as usize;
        let filename = winutil::to_filename(v, offset + 6, file_name_length_in_bytes);

        let fni = FileNotifyInformation {
            action: FileAction::from_u32(action),
            filename: filename,
        };

        (next_entry_offset_in_u16, fni)
    }
}

// #[test]
// fn test_sync() {
//     let watcher = sync(Path::new("d://x"), Box::new(vec![FileNotifyChange::FileName]), true, 1024);

//     loop {
//         let results = watcher.watch().unwrap();
//         for r in *results {
//             println!("{:?}", r);
//         }        
//     }
// }