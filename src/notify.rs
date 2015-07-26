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

use std::path::Path;

use types::{FileNotifyChange, NotifyStatus};
use ffi::*;

use ::libc::{WAIT_OBJECT_0, WAIT_ABANDONED, WAIT_TIMEOUT, WAIT_FAILED, INFINITE};

fn find_first_change_notification(directory: &Path, watch_subtree: bool, filters: Box<Vec<FileNotifyChange>>) -> HANDLE {
    let lp_path_name = to_lpcwstr(directory);
    let dw_notify_filter = FileNotifyChange::as_u32(filters);
    let b_watch_subtree = from_bool(watch_subtree);

    unsafe {
        // TODO check INVALID_HANDLE_VALUE for error
        FindFirstChangeNotificationW(lp_path_name, b_watch_subtree, dw_notify_filter)
    }
}

fn find_next_change_notification(handle: HANDLE) -> bool {
    to_bool(unsafe {
        FindNextChangeNotification(handle)
    })
}

fn find_close_change_notification(handle: HANDLE) -> bool {
    to_bool(unsafe {
        FindCloseChangeNotification(handle)
    })    
}

fn wait_for_single_object(handle: HANDLE) -> u32 {
    unsafe {
        WaitForSingleObject(handle, INFINITE)
    }
}

#[derive(Debug)]
pub struct WinNotify {

    handle: HANDLE,
}

impl Drop for WinNotify {
    
    fn drop(&mut self) {
        find_close_change_notification(self.handle);
    }
}

impl WinNotify {
    
    pub fn new(directory: &Path, filters: Box<Vec<FileNotifyChange>>, watch_subtree: bool) -> WinNotify {
        let handle = find_first_change_notification(directory, watch_subtree, filters); //TODO check errors

        WinNotify {
            handle: handle,
        }
    }

    pub fn notify(&self) -> NotifyStatus {
        let dw_wait_status = wait_for_single_object(self.handle);

        match dw_wait_status {
            WAIT_OBJECT_0 => {
                find_next_change_notification(self.handle); //FIXME check return type
                //let has_change = find_next_change_notification(self.handle);
                // if has_change {
                //     break;
                // }
                NotifyStatus::Change
            },
            WAIT_ABANDONED => NotifyStatus::Abandonned,
            WAIT_TIMEOUT => NotifyStatus::Timout,
            WAIT_FAILED => NotifyStatus::Faild(format!("Failure detected with system error code {}", get_last_error())),
            _ => unreachable!()
        }
    }
}