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

use std::error;
use std::fmt;
use std::convert::Into;

//
// Errors
//

#[derive(Debug)]
pub struct Error {
    description: String,
}

impl fmt::Display for Error {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description)
    }
}

impl error::Error for Error {

    fn description(&self) -> &str {
        &self.description
    }
}

impl Error {
    
    pub fn new<S>(description: S) -> Error where S: Into<String> {
        Error {
            description: description.into(),
        }
    }
}
