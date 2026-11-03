//! The site generator's command line, `edu`. A command writes its output to the writer that
//! `main` passes (stdout); `main` reports errors on stderr.

use std::ffi::{OsStr, OsString};
use std::fmt;
use std::io::{self, Write};

/// What `edu help` prints, and what follows a usage error.
pub const USAGE: &str = "\
Usage: edu <command>

Commands:
  help     Print this help
  version  Print the version
";

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Help,
    Version,
}

/// A command line that `edu` rejects; `main` exits with status 2.
#[derive(Debug, PartialEq, Eq)]
pub enum UsageError {
    MissingCommand,
    UnknownCommand(String),
    UnexpectedArgument(String),
}

impl Command {
    /// Reads the arguments that follow the program name. An argument that is not valid Unicode
    /// is reported with its invalid bytes replaced, where `std::env::args` would panic.
    pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Self, UsageError> {
        let mut args = args.into_iter();
        let Some(name) = args.next() else {
            return Err(UsageError::MissingCommand);
        };
        let command = match name.to_str() {
            Some("help" | "--help" | "-h") => Self::Help,
            Some("version" | "--version" | "-V") => Self::Version,
            _ => return Err(UsageError::UnknownCommand(lossy(&name))),
        };
        match args.next() {
            None => Ok(command),
            Some(extra) => Err(UsageError::UnexpectedArgument(lossy(&extra))),
        }
    }
}

/// The argument as text, with invalid Unicode replaced by U+FFFD.
fn lossy(arg: &OsStr) -> String {
    arg.to_string_lossy().into_owned()
}

/// Runs one command, writing its output to `out`.
pub fn run(command: &Command, out: &mut impl Write) -> io::Result<()> {
    match command {
        Command::Help => out.write_all(USAGE.as_bytes()),
        Command::Version => writeln!(out, "edu {}", env!("CARGO_PKG_VERSION")),
    }
}

impl fmt::Display for UsageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingCommand => f.write_str("no command given"),
            Self::UnknownCommand(name) => write!(f, "unknown command `{name}`"),
            Self::UnexpectedArgument(arg) => write!(f, "unexpected argument `{arg}`"),
        }
    }
}

impl std::error::Error for UsageError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Command, UsageError> {
        Command::parse(args.iter().map(OsString::from))
    }

    #[test]
    fn help_and_version_are_commands_and_flags() {
        for arg in ["help", "--help", "-h"] {
            assert_eq!(parse(&[arg]), Ok(Command::Help), "{arg}");
        }
        for arg in ["version", "--version", "-V"] {
            assert_eq!(parse(&[arg]), Ok(Command::Version), "{arg}");
        }
    }

    #[test]
    fn a_command_is_required() {
        assert_eq!(parse(&[]), Err(UsageError::MissingCommand));
    }

    #[test]
    fn an_unknown_command_is_named_in_the_error() {
        let error = UsageError::UnknownCommand("biuld".to_owned());
        assert_eq!(parse(&["biuld"]), Err(error));
    }

    #[test]
    fn a_command_takes_no_further_arguments() {
        let error = UsageError::UnexpectedArgument("--verbose".to_owned());
        assert_eq!(parse(&["version", "--verbose"]), Err(error));
    }

    #[cfg(unix)]
    #[test]
    fn an_argument_that_is_not_unicode_is_an_error_not_a_panic() {
        use std::os::unix::ffi::OsStringExt;

        let arg = OsString::from_vec(b"b\xffild".to_vec());
        let error = UsageError::UnknownCommand("b\u{fffd}ild".to_owned());
        assert_eq!(Command::parse([arg]), Err(error));
    }

    #[test]
    fn help_writes_the_usage_to_the_given_output() {
        let mut out = Vec::new();
        run(&Command::Help, &mut out).unwrap();
        assert_eq!(String::from_utf8(out).unwrap(), USAGE);
    }

    #[test]
    fn version_writes_the_package_version() {
        let mut out = Vec::new();
        run(&Command::Version, &mut out).unwrap();
        let expected = format!("edu {}\n", env!("CARGO_PKG_VERSION"));
        assert_eq!(String::from_utf8(out).unwrap(), expected);
    }
}
