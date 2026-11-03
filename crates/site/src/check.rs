//! `edu check`: every Markdown page under `content/` starts with a header in the YAML subset.

use std::ffi::OsStr;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use crate::Error;

/// A page whose header is not in the YAML subset.
#[derive(Debug)]
pub struct ContentError {
    /// The page, relative to the site root.
    pub path: PathBuf,
    pub error: edu_yaml::Error,
}

/// Parses the header of every Markdown page under `content/` in `root`, and returns how many
/// pages there are.
pub(crate) fn headers(root: &Path) -> Result<usize, Error> {
    let mut pages = Vec::new();
    markdown_files(root, Path::new("content"), &mut pages)?;
    let mut errors = Vec::new();
    for path in &pages {
        let read_error = |source| Error::Read {
            path: path.clone(),
            source,
        };
        let text = fs::read_to_string(root.join(path)).map_err(read_error)?;
        if let Err(error) = edu_yaml::parse_front_matter(&text) {
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
        let Self { path, error } = self;
        write!(f, "{}:{}: error: {}", path.display(), error.pos, error.kind)
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
        assert_eq!(headers(&site("valid")).unwrap(), 2);
    }

    #[test]
    fn each_bad_header_is_an_error_with_its_page_line_and_column_in_path_order() {
        let Err(Error::Content(errors)) = headers(&site("broken")) else {
            panic!("expected content errors");
        };
        let errors: Vec<String> = errors.iter().map(ToString::to_string).collect();
        let expected = [
            "content/a/no-header.md:1:1: error: expected a line `---` that opens the header",
            "content/b.md:2:8: error: a value with `: ` in it or `:` at its end needs quotes",
        ];
        assert_eq!(errors, expected);
    }

    #[test]
    fn a_site_without_content_is_a_read_error() {
        let result = headers(&site("missing"));
        let is_content = |path: &PathBuf| path == Path::new("content");
        assert!(
            matches!(&result, Err(Error::Read { path, .. }) if is_content(path)),
            "{result:?}"
        );
    }
}
