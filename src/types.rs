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

use std::fmt;

#[derive(Debug,Copy,Clone)]
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
    
    pub fn as_u32(filters: Box<Vec<FileNotifyChange>>) -> u32 {
        let mut result: u32 = 0;
        for f in filters.into_iter() {
            result |= f as u32;
        }
        result
    }
}

#[derive(Debug,Copy,Clone)]
pub enum FileAction {
    FileAdded = 0x00000001,
    FileRemoved = 0x00000002,
    FileModified = 0x00000003,
    FileRenamedOldName = 0x00000004,
    FileRenamedNewName = 0x00000005,
}

impl FileAction {

    pub fn from_u32(value: u32) -> FileAction {
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