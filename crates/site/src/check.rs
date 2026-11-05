//! `edu check`: every Markdown page under `content/` starts with a header in the YAML subset whose
//! values the site can use, its text is in the Markdown subset, and the layouts of `layouts/`
//! render it with the settings of `site.yaml`. `edu build` checks the same, and renders the pages
//! that it writes.

use std::ffi::OsStr;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use crate::Error;
use crate::build::{BaseUrl, Drafts, Options};
use crate::page::{Kind, Page};
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

/// Why the site cannot use a page or `site.yaml`: a value of its header, its path, or a layout it
/// needs.
#[derive(Debug, PartialEq, Eq)]
pub enum ValueError {
    /// A file or folder name in the page's path that Zola would change to make the URL.
    BadName(String),
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
    /// `draft: true` in `content/_index.md`, which every page is in.
    DraftHome,
    /// A `date` that is not a day of the calendar written as `2027-03-01`.
    BadDate,
    /// `authors` that is not a list of names.
    NotNames,
    /// A `sort_by` other than `date`.
    BadSortBy,
    /// A `redirect_to` that is not a URL starting with `https://` or `http://`.
    BadRedirect,
    /// A page without a `date` in a section with `sort_by: date`.
    NoDate,
    /// A `template`, or the layout a page needs without one, that is not a file of `layouts/`.
    NoLayout(String),
    /// A `site.yaml` without `base_url`.
    NoBaseUrl,
    /// A `base_url` that is not an address such as `https://rust-edu.org`.
    BadBaseUrl,
}

/// What `edu check` checked: how many pages and layouts; and for the build, the pages it rendered,
/// the files of `content/` that are not pages, and the site's address.
#[derive(Debug)]
pub(crate) struct Checked {
    pub(crate) pages: usize,
    pub(crate) layouts: usize,
    pub(crate) rendered: Vec<Rendered>,
    pub(crate) files: Vec<PathBuf>,
    pub(crate) base_url: BaseUrl,
}

/// A page as the build writes it: its path, its URL, its HTML and its `date`.
#[derive(Debug)]
pub(crate) struct Rendered {
    pub(crate) path: PathBuf,
    pub(crate) url: String,
    pub(crate) html: String,
    pub(crate) date: Option<String>,
}

/// Checks the header and the text of every Markdown page under `content/` in `root`, the settings
/// of `site.yaml` and the text of every layout of `layouts/`, and then renders with them each page
/// that has no error, as a redirect if it has `redirect_to`. Without drafts, it leaves out each
/// page with `draft: true` and everything in the folder of an `_index.md` with it.
pub(crate) fn pages(root: &Path, options: &Options) -> Result<Checked, Error> {
    let mut paths = Vec::new();
    walk(root, Path::new("content"), &mut paths)?;
    let is_page = |path: &PathBuf| path.extension() == Some(OsStr::new("md"));
    let (paths, files): (Vec<_>, Vec<_>) = paths.into_iter().partition(is_page);
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
    let site = render::site(root, options.base_url.as_ref(), &mut errors)?;
    let layouts = Layouts::read(root, &mut errors)?;
    // An error in `site.yaml` or in a layout would fail every page.
    let (Some(site), Some(layouts)) = (site, layouts) else {
        return Err(Error::Content(errors));
    };
    let drafts: Vec<PathBuf> = match options.drafts {
        Drafts::Include => Vec::new(),
        Drafts::Exclude => pages
            .iter()
            .filter(|page| page.draft)
            .map(left_out)
            .collect(),
    };
    let is_kept = |path: &Path| !drafts.iter().any(|draft| path.starts_with(draft));
    pages.retain(|page| is_kept(&page.path));
    let files = files.into_iter().filter(|file| is_kept(file)).collect();
    let mut rendered = Vec::new();
    for page in &pages {
        let html = match &page.redirect_to {
            Some(to) => Ok(render::redirect(to)),
            None => render::render(page, &pages, &site.value, &layouts),
        };
        match html {
            Ok(html) => {
                let (path, url) = (page.path.clone(), page.url.clone());
                let date = page.date.as_ref().map(|date| date.text.clone());
                rendered.push(Rendered {
                    path,
                    url,
                    html,
                    date,
                });
            }
            Err(error) => errors.push(error),
        }
    }
    if errors.is_empty() {
        let (pages, layouts, base_url) = (paths.len(), layouts.len(), site.base_url);
        Ok(Checked {
            pages,
            layouts,
            rendered,
            files,
            base_url,
        })
    } else {
        Err(Error::Content(errors))
    }
}

/// What a draft leaves out of the build: the page, or for an `_index.md` its folder.
fn left_out(page: &Page) -> PathBuf {
    match page.kind {
        Kind::Home | Kind::Section => page.path.parent().unwrap_or(&page.path).to_owned(),
        Kind::Page => page.path.clone(),
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

/// Adds the files under `dir`, a directory relative to `root`, to `files` in path order.
pub(crate) fn walk(root: &Path, dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), Error> {
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
            walk(root, &path, files)?;
        } else {
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
            Self::BadName(name) => write!(
                f,
                "`{name}` cannot be part of a URL; name files and folders with lowercase letters and \
                 digits joined by `-`, such as `rust-week-2027`"
            ),
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
            Self::DraftHome => f.write_str("the home page cannot be a draft"),
            Self::BadDate => f.write_str("a date is a day written as `2027-03-01`"),
            Self::NotNames => f.write_str("`authors` is a list of names, one `- name` per line"),
            Self::BadSortBy => f.write_str("the only `sort_by` is `date`"),
            Self::BadRedirect => f.write_str(
                "`redirect_to` is a full address, starting with `https://` or `http://`",
            ),
            Self::NoDate => f.write_str("a page in a section with `sort_by: date` needs a `date`"),
            Self::NoLayout(name) => write!(f, "`layouts/{name}` does not exist"),
            Self::NoBaseUrl => f.write_str("the settings need `base_url`, the site's address"),
            Self::BadBaseUrl => f.write_str(
                "`base_url` is an address such as `https://rust-edu.org` or \
                 `https://example.github.io/rust-edu`",
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A site under `crates/site/tests/sites/`, and how `edu check` checks it.
    fn site(name: &str) -> (PathBuf, Options) {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/sites")
            .join(name);
        let options = Options {
            drafts: Drafts::Include,
            base_url: None,
        };
        (path, options)
    }

    /// The URLs of the pages of `checked` and the files of its `content/`.
    fn built(checked: &Checked) -> (Vec<&str>, Vec<String>) {
        let urls = checked.rendered.iter().map(|page| page.url.as_str());
        let files = checked.files.iter().map(|file| file.display().to_string());
        (urls.collect(), files.collect())
    }

    #[test]
    fn every_markdown_file_under_content_is_a_page_and_every_html_file_a_layout() {
        // `valid` holds seven pages, one of them a redirect and three of them drafts or in the
        // folder of one, three other files, one of them in that folder, and three layouts and
        // `notes.txt`.
        let (root, mut options) = site("valid");
        let checked = pages(&root, &options).unwrap();
        assert_eq!((checked.pages, checked.layouts), (7, 3));
        let (urls, files) = built(&checked);
        let all = "/2027/ / /chat/ /news/draft/ /news/post/ /wip/ /wip/page/";
        assert_eq!((urls.join(" ").as_str(), files.len()), (all, 3));
        options.drafts = Drafts::Exclude;
        let checked = pages(&root, &options).unwrap();
        assert_eq!(checked.pages, 7);
        let (urls, files) = built(&checked);
        assert_eq!(urls.join(" "), "/2027/ / /chat/ /news/post/");
        assert_eq!(files, ["content/news/notes.txt", "content/news/sample.bib"]);
    }

    #[test]
    fn each_bad_header_or_text_is_an_error_with_its_page_line_and_column_in_path_order() {
        let (root, options) = site("broken");
        let Err(Error::Content(errors)) = pages(&root, &options) else {
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
        let (root, options) = site("unrenderable");
        let Err(Error::Content(errors)) = pages(&root, &options) else {
            panic!("expected content errors");
        };
        let errors: Vec<String> = errors.iter().map(ToString::to_string).collect();
        let expected = [
            "content/_index.md:3:8: error: the home page cannot be a draft",
            "content/author-map.md:4:3: error: `authors` is a list of names, one `- name` per line",
            "content/bad-date.md:3:7: error: a date is a day written as `2027-03-01`",
            "content/bad-draft.md:3:8: error: `draft` is `true` or `false`",
            "content/bad-redirect/_index.md:2:14: error: `redirect_to` is a full address, \
             starting with `https://` or `http://`",
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
            "content/snake_case.md:1:1: error: `snake_case` cannot be part of a URL; name files and \
             folders with lowercase letters and digits joined by `-`, such as `rust-week-2027`",
            "content/typo.md:2:1: error: a page's header has `title`, `date`, `authors`, \
             `template` and `draft`, not `titel`",
            "content/a.md:3:11: error: `layouts/missing.html` does not exist",
            "layouts/page.html:1:7: error: unknown variable `page.authors` while rendering \
             content/b.md",
            "layouts/page.html:1:7: error: unknown variable `page.authors` while rendering \
             content/draft.md",
            "content/news/undated.md:1:1: error: a page in a section with `sort_by: date` needs a \
             `date`",
        ];
        assert_eq!(errors, expected);
    }

    #[test]
    fn a_site_without_layout_html_is_a_read_error() {
        // `no-layout` has a page, `site.yaml` and `layouts/page.html`.
        let (root, options) = site("no-layout");
        let result = pages(&root, &options);
        let is_layout = |path: &PathBuf| path == Path::new("layouts/layout.html");
        assert!(
            matches!(&result, Err(Error::Read { path, .. }) if is_layout(path)),
            "{result:?}"
        );
    }

    #[test]
    fn a_site_without_content_is_a_read_error() {
        let (root, options) = site("missing");
        let result = pages(&root, &options);
        let is_content = |path: &PathBuf| path == Path::new("content");
        assert!(
            matches!(&result, Err(Error::Read { path, .. }) if is_content(path)),
            "{result:?}"
        );
    }
}
