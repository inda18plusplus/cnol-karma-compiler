
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

    /// A trailing skip in source code
    /// Wraps the line
    TrailingSkip(usize)
}

pub type Result<T> = std::result::Result<T, Error>;



#[inline]
pub fn try_or_exit<T>(result: Result<T>) -> T {
    match result {
        Ok(t) => t,
        Err(e) => {
            println!("Error: {:?}", e);
            std::process::exit(1);
        }
    }
}
