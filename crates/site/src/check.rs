//! `edu check`: every Markdown page under `content/` starts with a header in the YAML subset, and
//! the text after it is in the Markdown subset.

use std::ffi::OsStr;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use crate::Error;

/// A page whose header or text is not in the subsets.
#[derive(Debug)]
pub struct ContentError {
    /// The page, relative to the site root.
    pub path: PathBuf,
    pub error: PageError,
}

/// What is wrong with a page; positions count lines from the top of the page.
#[derive(Debug)]
pub enum PageError {
    Header(edu_yaml::Error),
    Text(edu_markdown::Error),
}

/// Checks the header and the text of every Markdown page under `content/` in `root`, and returns
/// how many pages there are.
pub(crate) fn pages(root: &Path) -> Result<usize, Error> {
    let mut pages = Vec::new();
    markdown_files(root, Path::new("content"), &mut pages)?;
    let mut errors = Vec::new();
    for path in &pages {
        let read_error = |source| Error::Read {
            path: path.clone(),
            source,
        };
        let text = fs::read_to_string(root.join(path)).map_err(read_error)?;
        if let Some(error) = page_error(&text) {
            let path = path.clone();
            errors.push(ContentError { path, error });
        }
    }
    if errors.is_empty() {
        Ok(pages.len())
    } else {
        Err(Error::Content(errors))
    }
}

/// The first error in `text`, a page: in its header, or else in the text after it.
fn page_error(text: &str) -> Option<PageError> {
    let body = match edu_yaml::parse_front_matter(text) {
        Ok((_, body)) => body,
        Err(error) => return Some(PageError::Header(error)),
    };
    let mut error = edu_markdown::to_html(body).err()?;
    // The text starts on the line after the header's closing `---`.
    let header = text.get(..text.len() - body.len()).unwrap_or_default();
    error.pos.line += header.matches('\n').count();
    Some(PageError::Text(error))
}

/// Adds the Markdown files under `dir`, a directory relative to `root`, to `files` in path order.
fn markdown_files(root: &Path, dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), Error> {
    let read_error = |source| Error::Read {
        path: dir.to_owned(),
        source,
    };
    let mut entries = Vec::new();
    for entry in fs::read_dir(root.join(dir)).map_err(read_error)? {
        let entry = entry.map_err(read_error)?;
        let is_dir = entry.file_type().map_err(read_error)?.is_dir();
        entries.push((entry.file_name(), is_dir));
    }
    entries.sort();
    for (name, is_dir) in entries {
        let path = dir.join(name);
        if is_dir {
            markdown_files(root, &path, files)?;
        } else if path.extension() == Some(OsStr::new("md")) {
            files.push(path);
        }
    }
    Ok(())
}

impl fmt::Display for ContentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let path = self.path.display();
        match &self.error {
            PageError::Header(error) => write!(f, "{path}:{}: error: {}", error.pos, error.kind),
            PageError::Text(error) => write!(f, "{path}:{}: error: {}", error.pos, error.kind),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A site under `crates/site/tests/sites/`.
    fn site(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/sites")
            .join(name)
    }

    #[test]
    fn every_markdown_file_under_content_is_a_page() {
        // `valid` holds two pages and `news/notes.txt`.
        assert_eq!(pages(&site("valid")).unwrap(), 2);
    }

    #[test]
    fn each_bad_header_or_text_is_an_error_with_its_page_line_and_column_in_path_order() {
        let Err(Error::Content(errors)) = pages(&site("broken")) else {
            panic!("expected content errors");
        };
        let errors: Vec<String> = errors.iter().map(ToString::to_string).collect();
        let expected = [
            "content/a/no-header.md:1:1: error: expected a line `---` that opens the header",
            "content/b.md:2:8: error: a value with `: ` in it or `:` at its end needs quotes",
            "content/d.md:5:11: error: HTML is not supported; write `\\<` for a `<`, or put code \
             in backticks",
        ];
        assert_eq!(errors, expected);
    }

    #[test]
    fn a_site_without_content_is_a_read_error() {
        let result = pages(&site("missing"));
        let is_content = |path: &PathBuf| path == Path::new("content");
        assert!(
            matches!(&result, Err(Error::Read { path, .. }) if is_content(path)),
            "{result:?}"
        );
    }
}
