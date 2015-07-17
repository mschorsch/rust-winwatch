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
