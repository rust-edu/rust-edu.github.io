//! The files of `content/` and `static/` that are not pages, which the build copies as they are:
//! only images, PDF and text files, LaTeX files and stylesheets. A browser runs the scripts of
//! other types of file, such as HTML, as if they were pages of the site.

use std::fmt;
use std::path::Path;

use crate::check::{ContentError, PageError};
use crate::page::TOP;

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
}

/// Checks the type of the file at `path`, relative to the site's root, which the build would
/// copy. An error goes to `errors`.
pub(crate) fn check(path: &Path, errors: &mut Vec<ContentError>) {
    if path == Path::new(CNAME) {
        return;
    }
    let extension = path
        .extension()
        .map(|extension| extension.to_string_lossy().to_ascii_lowercase());
    let error = match extension.as_deref() {
        Some(extension) if PUBLISHED.contains(&extension) => return,
        Some("htm" | "html" | "xhtml") => FileError::Html,
        _ => FileError::Type(extension),
    };
    let path = path.to_owned();
    let error = PageError::File(TOP, error);
    errors.push(ContentError { path, error });
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
        }
    }
}
