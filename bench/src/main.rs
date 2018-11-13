
use std::{
    env,
    time,
    process
};

fn main() {
    let args = env::args().skip(1);

    
    let mut command = if cfg!(target_os = "windows") {
        let mut command = process::Command::new("cmd");
        command.arg("/C");
        command
    } else {
        let mut command = process::Command::new("sh");
        command.arg("-c");
        command
    };

    let command_string = args.fold(String::new(), |acc, arg| format!("{} {}", acc, arg));
    command.arg(command_string);

    let start = time::Instant::now();

    command.status().unwrap();

    let seconds = seconds_since(start);
    println!("\nseconds: {}", seconds);
}


fn seconds_since(instant: time::Instant) -> f64 {
    let now = time::Instant::now();

    let duration = now - instant;

    duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1e-9
}
