use std::io;
use std::process::ExitCode;

use edu_site::{Command, USAGE};

fn main() -> ExitCode {
    let command = match Command::parse(std::env::args_os().skip(1)) {
        Ok(command) => command,
        Err(error) => {
            eprint!("edu: {error}\n\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    match edu_site::run(&command, &mut io::stdout().lock()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("edu: {error}");
            ExitCode::FAILURE
        }
    }
}
