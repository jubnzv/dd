use std::fmt;

/// An error that occurrs in `dd` crate.
#[derive(Debug)]
pub enum Error {
    /// The given input did not cause a failure.
    NoChange,
    /// Other kinds of errors propagated from other Result errors.
    Error(String),
}

impl Error {
    pub fn new<S>(msg: S) -> Error
    where
        S: Into<String>,
    {
        Error::Error(msg.into())
    }
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::NoChange => write!(f, "input did not cause a failure"),
            Error::Error(s) => write!(f, "{}", s),
        }
    }
}

impl From<String> for Error {
    fn from(msg: String) -> Self {
        Error::Error(msg)
    }
}
