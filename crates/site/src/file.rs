//! The files of `content/` and `static/` that are not pages, which the build copies as they are:
//! only images, PDF and text files, LaTeX files and stylesheets. A browser runs the scripts of
//! other types of file, such as HTML, as if they were pages of the site, and those of an SVG image
//! opened on its own too, so the site publishes an SVG image only if the scan of `svg.rs` lets it
//! through.

use std::fmt;
use std::fs;
use std::path::Path;

use edu_yaml::Pos;

use crate::Error;
use crate::check::{ContentError, PageError};
use crate::page::TOP;
use crate::svg::{self, SvgError};

/// The extensions, in lowercase, of the files that the site publishes besides its pages.
const PUBLISHED: [&str; 14] = [
    "avif", "bib", "cls", "css", "gif", "ico", "jpeg", "jpg", "pdf", "png", "svg", "tex", "txt",
    "webp",
];

/// The file that names the site's domain for GitHub Pages, the one file without an extension.
const CNAME: &str = "static/CNAME";

/// Why the site does not publish a file of `content/` or `static/` that is not a page.
#[derive(Debug, PartialEq, Eq)]
pub enum FileError {
    /// An HTML file, whose scripts a browser would run as a page of the site.
    Html,
    /// A file with another extension than those the site publishes, in lowercase, or with none.
    Type(Option<String>),
    /// An SVG image that the scan refuses.
    Svg(SvgError),
}

/// Checks the file at `path`, relative to `root`, which the build would copy: its type, and the
/// text of an SVG image. An error in the file goes to `errors`.
pub(crate) fn check(root: &Path, path: &Path, errors: &mut Vec<ContentError>) -> Result<(), Error> {
    if path == Path::new(CNAME) {
        return Ok(());
    }
    let extension = path
        .extension()
        .map(|extension| extension.to_string_lossy().to_ascii_lowercase());
    let (pos, error) = match extension.as_deref() {
        Some("svg") => match svg(root, path)? {
            Ok(()) => return Ok(()),
            Err((pos, error)) => (pos, FileError::Svg(error)),
        },
        Some(extension) if PUBLISHED.contains(&extension) => return Ok(()),
        Some("htm" | "html" | "xhtml") => (TOP, FileError::Html),
        _ => (TOP, FileError::Type(extension)),
    };
    let path = path.to_owned();
    let error = PageError::File(pos, error);
    errors.push(ContentError { path, error });
    Ok(())
}

/// The scan of the SVG image at `path`, relative to `root`.
fn svg(root: &Path, path: &Path) -> Result<Result<(), (Pos, SvgError)>, Error> {
    let read_error = |source| Error::Read {
        path: path.to_owned(),
        source,
    };
    let bytes = fs::read(root.join(path)).map_err(read_error)?;
    Ok(match String::from_utf8(bytes) {
        Ok(text) => svg::scan(&text),
        Err(_) => Err((TOP, SvgError::NotText)),
    })
}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Html => f.write_str(
                "the site publishes no HTML files; write the page in Markdown, as a `.md` file",
            ),
            Self::Type(extension) => {
                match extension {
                    Some(extension) => write!(f, "the site publishes no `.{extension}` files")?,
                    None => f.write_str("the site publishes no files without an extension")?,
                }
                f.write_str("; besides its pages, it publishes ")?;
                let last = PUBLISHED.len() - 1;
                for (at, extension) in PUBLISHED.iter().enumerate() {
                    let joint = match at {
                        0 => "",
                        _ if at == last => " and ",
                        _ => ", ",
                    };
                    write!(f, "{joint}`.{extension}`")?;
                }
                f.write_str(" files")
            }
            Self::Svg(error) => error.fmt(f),
        }
    }
}
