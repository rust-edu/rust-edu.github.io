//! The site generator's command line, `edu`. A command reads the site in the root directory that
//! `main` passes (the current directory) and writes its output to the writer that `main` passes
//! (stdout); `main` reports errors on stderr.

mod check;
mod page;
mod render;

use std::ffi::{OsStr, OsString};
use std::fmt;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

pub use check::{ContentError, PageError, ValueError};

/// What `edu help` prints, and what follows a usage error.
pub const USAGE: &str = "\
Usage: edu <command>

Commands:
  check    Check every page in content/, site.yaml and every layout in layouts/
  help     Print this help
  version  Print the version
";

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Check,
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

/// Why a command failed; `main` exits with status 1.
#[derive(Debug)]
pub enum Error {
    /// A file or directory of the site cannot be read, or a page is not UTF-8.
    Read { path: PathBuf, source: io::Error },
    /// Files of the site with errors: pages in path order, then `site.yaml` and the layouts, and
    /// then the pages that cannot render, in path order.
    Content(Vec<ContentError>),
    /// The output cannot be written.
    Write(io::Error),
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
            Some("check") => Self::Check,
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

/// Runs one command on the site in `root`, writing its output to `out`.
pub fn run(command: &Command, root: &Path, out: &mut impl Write) -> Result<(), Error> {
    let written = match command {
        Command::Check => {
            let checked = check::pages(root)?;
            let (pages, layouts) = (
                count(checked.pages, "page"),
                count(checked.layouts, "layout"),
            );
            writeln!(out, "checked {pages} with {layouts}")
        }
        Command::Help => out.write_all(USAGE.as_bytes()),
        Command::Version => writeln!(out, "edu {}", env!("CARGO_PKG_VERSION")),
    };
    written.map_err(Error::Write)
}

/// `1 page`, `2 pages`.
fn count(n: usize, noun: &str) -> String {
    let plural = if n == 1 { "" } else { "s" };
    format!("{n} {noun}{plural}")
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

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { path, .. } => write!(f, "cannot read {}", path.display()),
            Self::Content(errors) => write!(f, "{} in the site", count(errors.len(), "error")),
            Self::Write(_) => f.write_str("cannot write the output"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Read { source, .. } | Self::Write(source) => Some(source),
            Self::Content(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Command, UsageError> {
        Command::parse(args.iter().map(OsString::from))
    }

    /// Runs a command on the site `crates/site/tests/sites/valid`.
    fn output(command: &Command) -> String {
        let site = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/sites/valid");
        let mut out = Vec::new();
        run(command, &site, &mut out).unwrap();
        String::from_utf8(out).unwrap()
    }

    #[test]
    fn check_help_and_version_are_commands() {
        assert_eq!(parse(&["check"]), Ok(Command::Check));
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
        assert_eq!(output(&Command::Help), USAGE);
    }

    #[test]
    fn version_writes_the_package_version() {
        let expected = format!("edu {}\n", env!("CARGO_PKG_VERSION"));
        assert_eq!(output(&Command::Version), expected);
    }

    #[test]
    fn check_writes_how_many_pages_it_checked() {
        assert_eq!(output(&Command::Check), "checked 3 pages with 3 layouts\n");
    }

    #[test]
    fn counts_are_singular_for_one() {
        assert_eq!(count(1, "error"), "1 error");
        assert_eq!(count(2, "error"), "2 errors");
    }
}
