use errno::Errno;

use std::fmt;
use std::error::Error as StdError;
use std::result::Result as StdResult;
use std::convert::From;

#[derive(Debug)]
pub enum Error {
    Ioctl(Errno),
}

pub type Result<T> = StdResult<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Ioctl(ref err) => err.fmt(fmt),
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            _ => ""
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            _ => None
        }
    }
}

impl From<Errno> for Error {
    fn from(err: Errno) -> Error {
        Error::Ioctl(err)
    }
}
