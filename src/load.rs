
use std::{
    path::Path,
    fs::File,
    io::{
        Read,
    },
    str::from_utf8,
};

use error::{Error, Result};


pub fn load_file_text<P: AsRef<Path>>(path: P) -> Result<String> {
    let bytes = load_file_bytes(path)?;

    match from_utf8(&bytes) {
        Err(e) => Err(Error::IncompatibleEncoding(e)),
        Ok(text) => Ok(text.to_owned())
    }
}

fn load_file_bytes<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
    match File::open(path) {
        Err(e) => Err(Error::IoError(e)),
        Ok(mut file) => {
            let mut data = Vec::new();
            match file.read_to_end(&mut data) {
                Err(e) => Err(Error::IoError(e)),
                Ok(_) => Ok(data)
            }
        }
    }
}
