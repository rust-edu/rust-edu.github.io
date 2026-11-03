use std::error::Error as _;
use std::io;
use std::iter;
use std::path::Path;
use std::process::ExitCode;

use edu_site::{Command, Error, USAGE};

fn main() -> ExitCode {
    let command = match Command::parse(std::env::args_os().skip(1)) {
        Ok(command) => command,
        Err(error) => {
            eprint!("edu: {error}\n\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    match edu_site::run(&command, Path::new("."), &mut io::stdout().lock()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            report(&error);
            ExitCode::FAILURE
        }
    }
}

/// Prints each content error on its own line, then the error and its causes.
fn report(error: &Error) {
    if let Error::Content(errors) = error {
        for error in errors {
            eprintln!("{error}");
        }
    }
    eprint!("edu: {error}");
    for cause in iter::successors(error.source(), |&cause| cause.source()) {
        eprint!(": {cause}");
    }
    eprintln!();
}
