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

#![cfg(windows)]

//external crates
extern crate libc;

// public modules
pub mod watch;
pub mod notify;
pub mod errors;
pub mod types;

// private modules
mod ffi;

// reexports
pub use self::types::{FileNotifyChange, FileAction, NotifyStatus};
pub use self::watch::{WinWatch, FileNotifyInformation};
pub use self::notify::{WinNotify};

// uses
use std::path::Path;

pub fn watch_changes(directory: &Path, notify_changes: Box<Vec<FileNotifyChange>>, watch_subdirs: bool, buffer_size: u32) -> WinWatch {
    WinWatch::new(directory, notify_changes, watch_subdirs, buffer_size)
}

pub fn notify_changes(directory: &Path, filters: Box<Vec<FileNotifyChange>>, watch_subtree: bool) -> WinNotify {
    WinNotify::new(directory, filters, watch_subtree)
}