
mod parse;
mod error;
mod load;


pub use parse::*;
pub use error::*;

use load::load_file_text;


use std::path::Path;

/// Load a file from a file and parse it's contents into an abstract syntax tree
pub fn parse_file<P: AsRef<Path>>(path: P) -> Result<Vec<parse::Sequence>> {
    let source = load_file_text(path)?;

    interpret(&source)
}


