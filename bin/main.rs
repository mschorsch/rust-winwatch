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

#![allow(dead_code)]

extern crate winwatch;

use winwatch::*;

use std::path::Path;
use std::thread;
use std::sync::mpsc;

const DIRECTORY: &'static str = "d:/x";

fn main() {
    //Notify
    test_winnotify();

    // Watch
    let rx = test_winwatch(DIRECTORY.to_string());
    for r in rx {
        println!("{:?}", r);
    }

    // let directory = Path::new("d://x");
    // let filters = Box::new(vec![FileNotifyChange::FileName]);
    // let mut watcher = watch_changes(directory, filters, true, 1024);

    // loop {
    //     let results = watcher.watch().unwrap();
    //     for r in *results {
    //         println!("{:?}", r);
    //     }        
    // }
}

fn test_winnotify() {
    let notifier = notify_changes(Path::new(DIRECTORY), Box::new(vec![FileNotifyChange::FileName]), true);

    for _ in 0..2 {
        let status = notifier.notify();
        println!("{:?}", status);        
    }
}

fn test_winwatch(directory: String) -> mpsc::Receiver<FileNotifyInformation> {
    let (tx, rx) = mpsc::channel();
    
    thread::spawn(move || {
        let mut watcher = watch_changes(Path::new(&directory), Box::new(vec![FileNotifyChange::FileName]), true, 1024);
        loop {
            let results = watcher.watch().unwrap();
            for r in *results {
                tx.send(r).unwrap();
            }        
        }
    });

    rx
}