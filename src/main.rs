
use std::{
    env,
};

mod error;
use error::try_or_exit;

mod load;
use load::load_file_text;

mod parse;
use parse::interpret;

mod execution;
use execution::execute;

fn main() {
    let arguments: Vec<String> = env::args().collect();
    let path = &arguments[1];

    let source_code = try_or_exit(load_file_text(path));

    let sequences = try_or_exit(interpret(&source_code));
    execute(&sequences).unwrap();
}

