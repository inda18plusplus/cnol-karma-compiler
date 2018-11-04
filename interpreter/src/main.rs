
extern crate karma_parser;


use std::{
    env,
    process::exit,
    fmt::Debug
};

#[cfg(feature = "benchmark")]
use std::time;


use karma_parser::parse_file;

mod execution;
use execution::execute;

fn main() {
    #[cfg(feature = "benchmark")]
    let start_time = {
        time::Instant::now()
    };

    let path = try_or_exit(get_path_argument());
    let sequences = try_or_exit(parse_file(path));
    execute(&sequences);

    #[cfg(feature = "benchmark")]
    {
        let end_time = time::Instant::now();
        let duration = end_time - start_time;
        let seconds = duration.as_secs() as f64 + 1e-9 * duration.subsec_nanos() as f64;
        println!("Execution took: {:.4} seconds", seconds);
    };
}


fn get_path_argument() -> Result<String, String> {
    let mut arguments: Vec<String> = env::args().collect();

    if arguments.len() < 2 {
        Err("No source file in arguments".to_owned())
    } else {
        Ok(arguments.remove(1))
    }
}


fn try_or_exit<T, E: Debug>(result: Result<T, E>) -> T {
    match result {
        Ok(t) => t,
        Err(e) => {
            println!("Error: {:?}", e);
            exit(1);
        }
    }
}
