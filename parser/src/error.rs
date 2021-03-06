
use std::{
    self,
    io,
    str::Utf8Error
};

#[derive(Debug)]
pub enum Error {
    /// Any IO error
    IoError(io::Error),

    /// Incompatible encoding in source
    IncompatibleEncoding(Utf8Error),

    /// An invalid command was found in the source code
    InvalidCommand(char),

    /// A trailing skip on a specific line in source code
    TrailingSkip(usize)
}

pub type Result<T> = std::result::Result<T, Error>;

