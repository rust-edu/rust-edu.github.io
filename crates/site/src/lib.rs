//! The site generator's command line, `edu`. A command reads the site in the root directory that
//! `main` passes (the current directory) and writes its output to the writer that `main` passes
//! (stdout); `main` reports errors on stderr.

mod build;
mod check;
mod feed;
mod file;
mod links;
mod llms;
mod page;
mod render;
mod serve;
mod svg;
mod urls;

use std::error::Error as _;
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::io::{self, Write};
use std::iter;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::path::{Path, PathBuf};

pub use build::{BaseUrl, Drafts, Options};
pub use check::{ContentError, PageError, ValueError};
pub use file::FileError;
pub use links::LinkError;
pub use svg::SvgError;
pub use urls::UrlsError;

/// The folder that `edu build` writes, in the root of the site.
const OUT: &str = "public";

/// Where `edu serve` listens: on this computer only, at Zola's port.
const BIND: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 1111);

/// What `edu help` prints, and what follows a usage error.
pub const USAGE: &str = "\
Usage: edu <command>

Commands:
  build    Check the site, then write it into public/, without drafts
  check    Check every page in content/, and render it with layouts/ and site.yaml
  serve    Serve the site at http://127.0.0.1:1111/, built again after each change
  help     Print this help
  version  Print the version

Options of build:
  --drafts          Write the drafts too, as for a preview
  --base-url <url>  Build for this address instead of the base_url of site.yaml

Options of serve:
  --drafts          Serve the drafts too
  --bind <address>  Listen on this address and port instead, such as 0.0.0.0:1111 in a container
";

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Build(Options),
    Check,
    Serve { drafts: Drafts, bind: SocketAddr },
    Help,
    Version,
}

/// A command line that `edu` rejects; `main` exits with status 2.
#[derive(Debug, PartialEq, Eq)]
pub enum UsageError {
    MissingCommand,
    UnknownCommand(String),
    UnexpectedArgument(String),
    /// `--base-url` as the last argument.
    MissingBaseUrl,
    /// The value of `--base-url`, which is not an address.
    BadBaseUrl(String),
    /// `--bind` as the last argument.
    MissingBind,
    /// The value of `--bind`, which is not an IP address and a port.
    BadBind(String),
}

/// Why a command failed; `main` exits with status 1.
#[derive(Debug)]
pub enum Error {
    /// A file or directory of the site cannot be read, or a page is not UTF-8.
    Read { path: PathBuf, source: io::Error },
    /// A symbolic link among the files of the site, relative to the root, which could bring a file
    /// from outside the site into it.
    Symlink(PathBuf),
    /// Files of the site with errors: pages in path order, then the other files of `content/` and
    /// `static/`, then `site.yaml` and the layouts, and then the pages that cannot render, in path
    /// order.
    Content(Vec<ContentError>),
    /// The output cannot be written.
    Write(io::Error),
    /// A file or folder of the built site, relative to the root, cannot be written or deleted.
    Output { path: PathBuf, source: io::Error },
    /// A folder in the way of the built site that no earlier build made.
    Foreign(PathBuf),
    /// Two files of the site that would both be the file `path` of the built site.
    Clash {
        path: PathBuf,
        first: PathBuf,
        second: PathBuf,
    },
    /// A file of the site that would be `path`, a file that the build makes itself.
    Reserved { path: PathBuf, file: PathBuf },
    /// `edu serve` cannot listen on the address.
    Listen {
        address: SocketAddr,
        source: io::Error,
    },
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
            Some("build") => return build_options(args).map(Self::Build),
            Some("check") => Self::Check,
            Some("serve") => return serve_options(args),
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

/// The options of `edu build` in `args`, each at most once.
fn build_options(mut args: impl Iterator<Item = OsString>) -> Result<Options, UsageError> {
    let (mut drafts, mut base_url) = (Drafts::Exclude, None);
    while let Some(arg) = args.next() {
        match arg.to_str() {
            Some("--drafts") if drafts == Drafts::Exclude => drafts = Drafts::Include,
            Some("--base-url") if base_url.is_none() => {
                let url = lossy(&args.next().ok_or(UsageError::MissingBaseUrl)?);
                base_url = Some(BaseUrl::parse(&url).ok_or(UsageError::BadBaseUrl(url))?);
            }
            _ => return Err(UsageError::UnexpectedArgument(lossy(&arg))),
        }
    }
    Ok(Options { drafts, base_url })
}

/// The options of `edu serve` in `args`, each at most once.
fn serve_options(mut args: impl Iterator<Item = OsString>) -> Result<Command, UsageError> {
    let (mut drafts, mut bind) = (Drafts::Exclude, None);
    while let Some(arg) = args.next() {
        match arg.to_str() {
            Some("--drafts") if drafts == Drafts::Exclude => drafts = Drafts::Include,
            Some("--bind") if bind.is_none() => {
                let address = lossy(&args.next().ok_or(UsageError::MissingBind)?);
                bind = Some(address.parse().map_err(|_| UsageError::BadBind(address))?);
            }
            _ => return Err(UsageError::UnexpectedArgument(lossy(&arg))),
        }
    }
    let bind = bind.unwrap_or(BIND);
    Ok(Command::Serve { drafts, bind })
}

/// The argument as text, with invalid Unicode replaced by U+FFFD.
fn lossy(arg: &OsStr) -> String {
    arg.to_string_lossy().into_owned()
}

/// Runs one command on the site in `root`, writing its output to `out`.
pub fn run(command: &Command, root: &Path, out: &mut impl Write) -> Result<(), Error> {
    let written = match command {
        Command::Build(options) => {
            let built = build::site(root, Path::new(OUT), options)?;
            let (pages, files) = (count(built.pages, "page"), count(built.files, "file"));
            writeln!(out, "wrote {pages} and {files} to {OUT}/")
        }
        Command::Check => {
            let options = Options {
                drafts: Drafts::Include,
                base_url: None,
            };
            let checked = check::pages(root, &options)?;
            let (pages, layouts) = (
                count(checked.pages, "page"),
                count(checked.layouts, "layout"),
            );
            writeln!(out, "checked {pages} with {layouts}")
        }
        Command::Serve { drafts, bind } => {
            let listen_error = |source| Error::Listen {
                address: *bind,
                source,
            };
            let listener = TcpListener::bind(bind).map_err(listen_error)?;
            let address = listener.local_addr().map_err(listen_error)?;
            let started = writeln!(out, "serving http://{address}/ until Ctrl-C");
            started.and_then(|()| out.flush()).map_err(Error::Write)?;
            let drafts = *drafts;
            serve::serve(
                &listener,
                root,
                &Options {
                    drafts,
                    base_url: None,
                },
            );
            Ok(())
        }
        Command::Help => out.write_all(USAGE.as_bytes()),
        Command::Version => writeln!(out, "edu {}", env!("CARGO_PKG_VERSION")),
    };
    written.map_err(Error::Write)
}

/// The report of `error` as the command line prints it: each error of the content on its own line,
/// then the error and its causes.
pub fn report(error: &Error) -> String {
    let mut report = String::new();
    if let Error::Content(errors) = error {
        for error in errors {
            report.push_str(&format!("{error}\n"));
        }
    }
    report.push_str(&format!("edu: {error}"));
    for cause in iter::successors(error.source(), |&cause| cause.source()) {
        report.push_str(&format!(": {cause}"));
    }
    report.push('\n');
    report
}

/// `1 page`, `2 pages`.
pub(crate) fn count(n: usize, noun: &str) -> String {
    let plural = if n == 1 { "" } else { "s" };
    format!("{n} {noun}{plural}")
}

impl fmt::Display for UsageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingCommand => f.write_str("no command given"),
            Self::UnknownCommand(name) => write!(f, "unknown command `{name}`"),
            Self::UnexpectedArgument(arg) => write!(f, "unexpected argument `{arg}`"),
            Self::MissingBaseUrl => f.write_str("`--base-url` needs an address after it"),
            Self::BadBaseUrl(url) => write!(
                f,
                "`{url}` is not an address such as `https://example.github.io/rust-edu`"
            ),
            Self::MissingBind => f.write_str("`--bind` needs an address and a port after it"),
            Self::BadBind(address) => write!(
                f,
                "`{address}` is not an IP address and a port such as `0.0.0.0:1111`"
            ),
        }
    }
}

impl std::error::Error for UsageError {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { path, .. } => write!(f, "cannot read {}", path.display()),
            Self::Symlink(path) => write!(
                f,
                "{} is a symbolic link, which could bring a file from outside the site into it; \
                 put the file or folder itself there",
                path.display()
            ),
            Self::Content(errors) => write!(f, "{} in the site", count(errors.len(), "error")),
            Self::Write(_) => f.write_str("cannot write the output"),
            Self::Output { path, .. } => write!(f, "cannot write {}", path.display()),
            Self::Foreign(path) => write!(
                f,
                "{} is in the way: no earlier `edu build` made it, so move it away",
                path.display()
            ),
            Self::Clash {
                path,
                first,
                second,
            } => {
                let (path, first, second) = (path.display(), first.display(), second.display());
                write!(f, "{first} and {second} would both be /{path}")
            }
            Self::Listen { address, .. } => write!(f, "cannot listen on {address}"),
            Self::Reserved { path, file } => {
                let (path, file) = (path.display(), file.display());
                write!(
                    f,
                    "{file} would be /{path}, which `edu build` writes itself"
                )
            }
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Read { source, .. }
            | Self::Write(source)
            | Self::Output { source, .. }
            | Self::Listen { source, .. } => Some(source),
            Self::Symlink(_)
            | Self::Content(_)
            | Self::Foreign(_)
            | Self::Clash { .. }
            | Self::Reserved { .. } => None,
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
        let live = Options {
            drafts: Drafts::Exclude,
            base_url: None,
        };
        assert_eq!(parse(&["build"]), Ok(Command::Build(live)));
        assert_eq!(parse(&["check"]), Ok(Command::Check));
        for arg in ["help", "--help", "-h"] {
            assert_eq!(parse(&[arg]), Ok(Command::Help), "{arg}");
        }
        for arg in ["version", "--version", "-V"] {
            assert_eq!(parse(&[arg]), Ok(Command::Version), "{arg}");
        }
    }

    #[test]
    fn build_takes_drafts_and_a_base_url_once_each() {
        let base_url = BaseUrl::parse("https://a.org/b");
        let preview = Options {
            drafts: Drafts::Include,
            base_url,
        };
        let args = ["build", "--base-url", "https://a.org/b/", "--drafts"];
        assert_eq!(parse(&args), Ok(Command::Build(preview)));
        let unexpected = |arg: &str| Err(UsageError::UnexpectedArgument(arg.to_owned()));
        let cases = [
            (
                &["build", "--drafts", "--drafts"][..],
                unexpected("--drafts"),
            ),
            (
                &["build", "--base-url", "http://a", "--base-url"],
                unexpected("--base-url"),
            ),
            (&["build", "--draft"], unexpected("--draft")),
            (&["build", "--base-url"], Err(UsageError::MissingBaseUrl)),
            (
                &["build", "--base-url", "a.org"],
                Err(UsageError::BadBaseUrl("a.org".into())),
            ),
        ];
        for (args, expected) in cases {
            assert_eq!(parse(args), expected, "{args:?}");
        }
        let messages = [
            UsageError::MissingBaseUrl.to_string(),
            UsageError::BadBaseUrl("a".into()).to_string(),
        ];
        let expected = [
            "`--base-url` needs an address after it",
            "`a` is not an address such as `https://example.github.io/rust-edu`",
        ];
        assert_eq!(messages, expected);
    }

    #[test]
    fn serve_takes_drafts_and_an_address_to_listen_on_once_each() {
        let serve = |drafts, bind: [u8; 4]| {
            Ok(Command::Serve {
                drafts,
                bind: (bind, 1111).into(),
            })
        };
        assert_eq!(parse(&["serve"]), serve(Drafts::Exclude, [127, 0, 0, 1]));
        let args = ["serve", "--bind", "0.0.0.0:1111", "--drafts"];
        assert_eq!(parse(&args), serve(Drafts::Include, [0, 0, 0, 0]));
        let unexpected = |arg: &str| Err(UsageError::UnexpectedArgument(arg.to_owned()));
        let bad = Err(UsageError::BadBind(":1".into()));
        let cases = [
            (
                &["serve", "--drafts", "--drafts"][..],
                unexpected("--drafts"),
            ),
            (
                &["serve", "--bind", "127.0.0.1:1", "--bind", "a"],
                unexpected("--bind"),
            ),
            (
                &["serve", "--base-url", "https://a.org"],
                unexpected("--base-url"),
            ),
            (&["serve", "--bind", ":1"], bad),
            (&["serve", "--bind"], Err(UsageError::MissingBind)),
        ];
        for (args, expected) in cases {
            assert_eq!(parse(args), expected, "{args:?}");
        }
        let messages = [UsageError::MissingBind, UsageError::BadBind("a".into())];
        let expected = [
            "`--bind` needs an address and a port after it",
            "`a` is not an IP address and a port such as `0.0.0.0:1111`",
        ];
        assert_eq!(messages.map(|message| message.to_string()), expected);
    }

    #[test]
    fn serve_says_when_it_cannot_listen() {
        let taken = TcpListener::bind("127.0.0.1:0").unwrap();
        let bind = taken.local_addr().unwrap();
        let drafts = Drafts::Exclude;
        let site = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/sites/valid");
        let error = run(&Command::Serve { drafts, bind }, &site, &mut Vec::new()).unwrap_err();
        assert_eq!(error.to_string(), format!("cannot listen on {bind}"));
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
        assert_eq!(output(&Command::Check), "checked 8 pages with 4 layouts\n");
    }

    #[test]
    fn check_renders_the_drafts_too() {
        // One of the errors of `unrenderable` is in a draft.
        let site = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/sites/unrenderable");
        let error = run(&Command::Check, &site, &mut Vec::new()).unwrap_err();
        assert_eq!(error.to_string(), "22 errors in the site");
    }

    #[test]
    fn counts_are_singular_for_one() {
        assert_eq!(count(1, "error"), "1 error");
        assert_eq!(count(2, "error"), "2 errors");
    }
}
