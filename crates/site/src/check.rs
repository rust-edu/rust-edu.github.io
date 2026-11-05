//! `edu check`: every Markdown page under `content/` starts with a header in the YAML subset whose
//! values the site can use, its text is in the Markdown subset, and the layouts of `layouts/`
//! render it with the settings of `site.yaml`.

use std::ffi::OsStr;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use crate::Error;
use crate::page::Page;
use crate::render::{self, Layouts};

/// A file of the site with an error: a page, `site.yaml` or a layout.
#[derive(Debug)]
pub struct ContentError {
    /// The file, relative to the site root.
    pub path: PathBuf,
    pub error: PageError,
}

/// What is wrong with a file; in a page, positions count lines from the top of the page.
#[derive(Debug)]
pub enum PageError {
    /// An error in the YAML of a page's header or of `site.yaml`.
    Header(edu_yaml::Error),
    /// A value of a page's header that the site cannot use, and where it is.
    Value(edu_yaml::Pos, ValueError),
    Text(edu_markdown::Error),
    /// An error in a layout's text.
    Layout(edu_template::Error),
    /// An error in a layout while it renders `page`.
    Render {
        page: PathBuf,
        error: edu_template::Error,
    },
}

/// Why the site cannot use a value of a page's header.
#[derive(Debug, PartialEq, Eq)]
pub enum ValueError {
    /// A page without a `title` that does not redirect.
    NoTitle,
    /// A key that the header of a page of its own does not have.
    NotPageKey(String),
    /// A key that the header of an `_index.md` does not have.
    NotSectionKey(String),
    /// A list or a map where the header needs a text.
    NotText,
    /// A `draft` other than `true` and `false`.
    BadDraft,
    /// A `date` that is not a day of the calendar written as `2027-03-01`.
    BadDate,
    /// `authors` that is not a list of names.
    NotNames,
    /// A `sort_by` other than `date`.
    BadSortBy,
    /// A page without a `date` in a section with `sort_by: date`.
    NoDate,
    /// A `template`, or the layout a page needs without one, that is not a file of `layouts/`.
    NoLayout(String),
}

/// How many pages and layouts `edu check` checked.
#[derive(Debug)]
pub(crate) struct Checked {
    pub(crate) pages: usize,
    pub(crate) layouts: usize,
}

/// Checks the header and the text of every Markdown page under `content/` in `root`, the settings
/// of `site.yaml` and the text of every layout of `layouts/`, and then renders with them each page
/// that has no error and does not redirect.
pub(crate) fn pages(root: &Path) -> Result<Checked, Error> {
    let mut paths = Vec::new();
    markdown_files(root, Path::new("content"), &mut paths)?;
    let (mut pages, mut errors) = (Vec::new(), Vec::new());
    for path in &paths {
        match Page::read(path, &read(root, path)?) {
            Ok(page) => pages.push(page),
            Err(error) => {
                let path = path.clone();
                errors.push(ContentError { path, error });
            }
        }
    }
    let site = render::site(root, &mut errors)?;
    let layouts = Layouts::read(root, &mut errors)?;
    // An error in `site.yaml` or in a layout would fail every page.
    let (Some(site), Some(layouts)) = (site, layouts) else {
        return Err(Error::Content(errors));
    };
    for page in pages.iter().filter(|page| page.redirect_to.is_none()) {
        if let Err(error) = render::render(page, &pages, &site, &layouts) {
            errors.push(error);
        }
    }
    if errors.is_empty() {
        let (pages, layouts) = (paths.len(), layouts.len());
        Ok(Checked { pages, layouts })
    } else {
        Err(Error::Content(errors))
    }
}

/// The text of the file at `path`, relative to `root`.
pub(crate) fn read(root: &Path, path: &Path) -> Result<String, Error> {
    let read_error = |source| Error::Read {
        path: path.to_owned(),
        source,
    };
    fs::read_to_string(root.join(path)).map_err(read_error)
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
            PageError::Value(pos, kind) => write!(f, "{path}:{pos}: error: {kind}"),
            PageError::Text(error) => write!(f, "{path}:{}: error: {}", error.pos, error.kind),
            PageError::Layout(error) => write!(f, "{path}:{}: error: {}", error.pos, error.kind),
            PageError::Render { page, error } => {
                let page = page.display();
                write!(
                    f,
                    "{path}:{}: error: {} while rendering {page}",
                    error.pos, error.kind
                )
            }
        }
    }
}

impl fmt::Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoTitle => f.write_str("a page needs a `title`"),
            Self::NotPageKey(key) => write!(
                f,
                "a page's header has `title`, `date`, `authors`, `template` and `draft`, not `{key}`"
            ),
            Self::NotSectionKey(key) => write!(
                f,
                "the header of an `_index.md` has `title`, `sort_by`, `template`, `redirect_to` \
                 and `draft`, not `{key}`"
            ),
            Self::NotText => f.write_str("this value must be text"),
            Self::BadDraft => f.write_str("`draft` is `true` or `false`"),
            Self::BadDate => f.write_str("a date is a day written as `2027-03-01`"),
            Self::NotNames => f.write_str("`authors` is a list of names, one `- name` per line"),
            Self::BadSortBy => f.write_str("the only `sort_by` is `date`"),
            Self::NoDate => f.write_str("a page in a section with `sort_by: date` needs a `date`"),
            Self::NoLayout(name) => write!(f, "`layouts/{name}` does not exist"),
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
    fn every_markdown_file_under_content_is_a_page_and_every_html_file_a_layout() {
        // `valid` holds three pages, one of them a redirect, and `news/notes.txt`, and three
        // layouts and `notes.txt`.
        let checked = pages(&site("valid")).unwrap();
        assert_eq!((checked.pages, checked.layouts), (3, 3));
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
            "site.yaml:1:8: error: a value with `: ` in it or `:` at its end needs quotes",
            "layouts/layout.html:1:11: error: unknown filter `upper`; the only filter is `safe`",
        ];
        assert_eq!(errors, expected);
    }

    #[test]
    fn each_value_or_layout_that_a_page_cannot_render_with_is_an_error() {
        let Err(Error::Content(errors)) = pages(&site("unrenderable")) else {
            panic!("expected content errors");
        };
        let errors: Vec<String> = errors.iter().map(ToString::to_string).collect();
        let expected = [
            "content/author-map.md:4:3: error: `authors` is a list of names, one `- name` per line",
            "content/bad-date.md:3:7: error: a date is a day written as `2027-03-01`",
            "content/bad-draft.md:3:8: error: `draft` is `true` or `false`",
            "content/list-title.md:3:3: error: this value must be text",
            "content/no-title.md:1:1: error: a page needs a `title`",
            "content/one-author.md:3:10: error: `authors` is a list of names, one `- name` per line",
            "content/page-key.md:3:1: error: a page's header has `title`, `date`, `authors`, \
             `template` and `draft`, not `sort_by`",
            "content/page-redirect.md:3:1: error: a page's header has `title`, `date`, `authors`, \
             `template` and `draft`, not `redirect_to`",
            "content/sections/_index.md:3:10: error: the only `sort_by` is `date`",
            "content/sections/authors/_index.md:3:1: error: the header of an `_index.md` has \
             `title`, `sort_by`, `template`, `redirect_to` and `draft`, not `authors`",
            "content/sections/date/_index.md:3:1: error: the header of an `_index.md` has `title`, \
             `sort_by`, `template`, `redirect_to` and `draft`, not `date`",
            "content/typo.md:2:1: error: a page's header has `title`, `date`, `authors`, \
             `template` and `draft`, not `titel`",
            "content/a.md:3:11: error: `layouts/missing.html` does not exist",
            "layouts/page.html:1:7: error: unknown variable `page.authors` while rendering \
             content/b.md",
            "content/news/undated.md:1:1: error: a page in a section with `sort_by: date` needs a \
             `date`",
        ];
        assert_eq!(errors, expected);
    }

    #[test]
    fn a_site_without_layout_html_is_a_read_error() {
        // `no-layout` has a page, `site.yaml` and `layouts/page.html`.
        let result = pages(&site("no-layout"));
        let is_layout = |path: &PathBuf| path == Path::new("layouts/layout.html");
        assert!(
            matches!(&result, Err(Error::Read { path, .. }) if is_layout(path)),
            "{result:?}"
        );
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
