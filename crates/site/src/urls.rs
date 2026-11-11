//! `urls.yaml`: the public addresses of the site that external links and feed readers may point
//! to (`keep`), and the addresses that redirect elsewhere (`redirects`).

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use edu_yaml::{Entry, Node, Pos, Value};

use crate::Error;
use crate::check::{self, ContentError, PageError};
use crate::links::{self, Link, Target};
use crate::page;

/// The file at the root of the site that lists its preserved addresses and redirects.
pub(crate) const FILE: &str = "urls.yaml";

/// A top-level section of `urls.yaml`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Section {
    Keep,
    Redirects,
}

impl Section {
    fn parse(key: &str) -> Option<Self> {
        match key {
            "keep" => Some(Self::Keep),
            "redirects" => Some(Self::Redirects),
            _ => None,
        }
    }
}

/// A key of one redirect entry under `redirects`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Field {
    From,
    To,
}

impl Field {
    fn parse(key: &str) -> Option<Self> {
        match key {
            "from" => Some(Self::From),
            "to" => Some(Self::To),
            _ => None,
        }
    }
}

/// Why `urls.yaml` cannot be used.
#[derive(Debug, PartialEq, Eq)]
pub enum UrlsError {
    /// A top-level key other than `keep` and `redirects`.
    UnknownKey(String),
    /// `keep` that is not a list of site addresses.
    NotKeepList,
    /// An item of `keep` that does not start with a single `/`.
    NotSiteUrl(String),
    /// An address listed twice in `keep`.
    DuplicateKeep { url: String, first_line: usize },
    /// An address in `keep` whose page path is already a `from` in `redirects`.
    KeepIsRedirect { url: String, redirect_line: usize },
    /// `redirects` that is not a list of `from` and `to` maps.
    NotRedirectList,
    /// A key in a redirect item other than `from` and `to`.
    UnknownRedirectKey(String),
    /// A list or a map where `from` or `to` needs text.
    NotText,
    /// A redirect item without `from`.
    MissingFrom,
    /// A redirect item without `to`.
    MissingTo,
    /// A `from` that is not `/name/` with lowercase kebab-case segments.
    BadFrom(String),
    /// Two redirects with the same `from`.
    DuplicateRedirect { url: String, first_line: usize },
    /// A `from` whose URL is already a page of `content/`.
    PageExists { url: String, page: PathBuf },
    /// A `to` that does not start with `https://`, `http://` or `/`.
    BadTo(String),
    /// A redirect whose `to` points back to its own `from`.
    SelfRedirect(String),
    /// A redirect whose `to` points to another redirect instead of the destination.
    RedirectChain(String),
}

/// The parsed `urls.yaml`: the addresses to check on the live site, and the redirects to build.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct Urls {
    pub(crate) keep: Vec<(Pos, String)>,
    pub(crate) redirects: Vec<Redirect>,
}

/// One redirect from `urls.yaml`, built as `<from>/index.html`.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Redirect {
    pub(crate) from_pos: Pos,
    pub(crate) from: String,
    pub(crate) to_pos: Pos,
    pub(crate) to: String,
}

impl Urls {
    /// Reads `urls.yaml` in `root`, if there is one, checking its structure against the pages of
    /// `content_pages` and any `_index.md` URLs in `section_redirects`. Structural errors go to
    /// `errors` in line order.
    pub(crate) fn read(
        root: &Path,
        content_pages: &[PathBuf],
        section_redirects: &[String],
        errors: &mut Vec<ContentError>,
    ) -> Result<Self, Error> {
        let path = Path::new(FILE);
        match fs::symlink_metadata(root.join(path)) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(Error::Symlink(path.to_owned()));
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(source) => {
                return Err(Error::Read {
                    path: path.to_owned(),
                    source,
                });
            }
        }
        let text = check::read(root, path)?;
        Ok(parse(&text, content_pages, section_redirects, errors))
    }

    /// Checks each internal redirect target `to` and each address of `keep` against `targets`,
    /// pushing any broken links to `errors` in file line order.
    pub(crate) fn check(
        &self,
        targets: &BTreeMap<String, Target<'_>>,
        errors: &mut Vec<ContentError>,
    ) {
        let mut found = Vec::new();
        for redirect in &self.redirects {
            if Link::of(&redirect.to) == Link::Top
                && let Err(kind) = links::resolve(&redirect.to, "", true, targets)
            {
                found.push((redirect.to_pos, PageError::Link(redirect.to_pos, kind)));
            }
        }
        for (pos, url) in &self.keep {
            if let Err(kind) = links::resolve(url, "", true, targets) {
                found.push((*pos, PageError::Link(*pos, kind)));
            }
        }
        found.sort_by_key(|(pos, _)| (pos.line, pos.col));
        for (_, error) in found {
            errors.push(ContentError {
                path: PathBuf::from(FILE),
                error,
            });
        }
    }
}

/// Parses `text`, the contents of `urls.yaml`, recording any errors in `errors` sorted by line and
/// column.
fn parse(
    text: &str,
    content_pages: &[PathBuf],
    section_redirects: &[String],
    errors: &mut Vec<ContentError>,
) -> Urls {
    let entries = match edu_yaml::parse(text) {
        Ok(entries) => entries,
        Err(error) => {
            errors.push(ContentError {
                path: PathBuf::from(FILE),
                error: PageError::Header(error),
            });
            return Urls::default();
        }
    };
    let existing: BTreeMap<String, &Path> = content_pages
        .iter()
        .filter_map(|path| Some((page::url(path).ok()?, path.as_path())))
        .collect();
    let mut urls = Urls::default();
    let mut found: Vec<(Pos, UrlsError)> = Vec::new();
    for entry in &entries {
        match Section::parse(&entry.key) {
            Some(Section::Keep) => parse_keep(&entry.value, &mut urls.keep, &mut found),
            Some(Section::Redirects) => {
                parse_redirects(&entry.value, &existing, &mut urls.redirects, &mut found);
            }
            None => found.push((entry.pos, UrlsError::UnknownKey(entry.key.clone()))),
        }
    }
    for redirect in &urls.redirects {
        if Link::of(&redirect.to) == Link::Top {
            let to_path = without_anchor(&redirect.to);
            let chains_url = urls.redirects.iter().any(|other| other.from == to_path);
            let chains_section = section_redirects.iter().any(|url| url == to_path);
            if chains_url || chains_section {
                found.push((
                    redirect.to_pos,
                    UrlsError::RedirectChain(redirect.to.clone()),
                ));
            }
        }
    }
    urls.keep.retain(|(pos, url)| {
        let keep_path = without_anchor(url);
        if let Some(redirect) = urls.redirects.iter().find(|r| r.from == keep_path) {
            found.push((
                *pos,
                UrlsError::KeepIsRedirect {
                    url: url.clone(),
                    redirect_line: redirect.from_pos.line,
                },
            ));
            return false;
        }
        true
    });
    found.sort_by_key(|(pos, _)| (pos.line, pos.col));
    for (pos, kind) in found {
        errors.push(ContentError {
            path: PathBuf::from(FILE),
            error: PageError::Urls(pos, kind),
        });
    }
    urls
}

/// Parses `keep`, a list of root-relative site addresses.
fn parse_keep(node: &Node, keep: &mut Vec<(Pos, String)>, found: &mut Vec<(Pos, UrlsError)>) {
    let Value::List(items) = &node.value else {
        found.push((node.pos, UrlsError::NotKeepList));
        return;
    };
    for item in items {
        let Value::Str(url) = &item.value else {
            found.push((item.pos, UrlsError::NotKeepList));
            continue;
        };
        if Link::of(url) != Link::Top {
            found.push((item.pos, UrlsError::NotSiteUrl(url.clone())));
            continue;
        }
        if let Some((first, _)) = keep.iter().find(|(_, seen)| seen == url) {
            found.push((
                item.pos,
                UrlsError::DuplicateKeep {
                    url: url.clone(),
                    first_line: first.line,
                },
            ));
            continue;
        }
        keep.push((item.pos, url.clone()));
    }
}

/// Parses `redirects`, a list of `{ from, to }` maps.
fn parse_redirects(
    node: &Node,
    existing: &BTreeMap<String, &Path>,
    redirects: &mut Vec<Redirect>,
    found: &mut Vec<(Pos, UrlsError)>,
) {
    let Value::List(items) = &node.value else {
        found.push((node.pos, UrlsError::NotRedirectList));
        return;
    };
    for item in items {
        let Value::Map(entries) = &item.value else {
            found.push((item.pos, UrlsError::NotRedirectList));
            continue;
        };
        if let Some(redirect) = parse_redirect(item.pos, entries, existing, redirects, found) {
            redirects.push(redirect);
        }
    }
}

/// Parses one map item under `redirects`.
fn parse_redirect(
    item_pos: Pos,
    entries: &[Entry],
    existing: &BTreeMap<String, &Path>,
    redirects: &[Redirect],
    found: &mut Vec<(Pos, UrlsError)>,
) -> Option<Redirect> {
    let (mut has_from, mut has_to) = (false, false);
    let (mut from, mut to) = (None, None);
    for entry in entries {
        match Field::parse(&entry.key) {
            Some(Field::From) => {
                has_from = true;
                match &entry.value.value {
                    Value::Str(text) => from = Some((entry.value.pos, text.as_str())),
                    Value::List(_) | Value::Map(_) => {
                        found.push((entry.value.pos, UrlsError::NotText));
                    }
                }
            }
            Some(Field::To) => {
                has_to = true;
                match &entry.value.value {
                    Value::Str(text) => to = Some((entry.value.pos, text.as_str())),
                    Value::List(_) | Value::Map(_) => {
                        found.push((entry.value.pos, UrlsError::NotText));
                    }
                }
            }
            None => found.push((entry.pos, UrlsError::UnknownRedirectKey(entry.key.clone()))),
        }
    }
    if !has_from {
        found.push((item_pos, UrlsError::MissingFrom));
    }
    if !has_to {
        found.push((item_pos, UrlsError::MissingTo));
    }
    let mut ok = true;
    if let Some((from_pos, from_url)) = from {
        if !is_redirect_from(from_url) {
            found.push((from_pos, UrlsError::BadFrom(from_url.to_owned())));
            ok = false;
        } else if let Some(first) = redirects.iter().find(|r| r.from == from_url) {
            found.push((
                from_pos,
                UrlsError::DuplicateRedirect {
                    url: from_url.to_owned(),
                    first_line: first.from_pos.line,
                },
            ));
            ok = false;
        } else if let Some(&page) = existing.get(from_url) {
            found.push((
                from_pos,
                UrlsError::PageExists {
                    url: from_url.to_owned(),
                    page: page.to_owned(),
                },
            ));
            ok = false;
        }
    } else {
        ok = false;
    }
    if let Some((to_pos, to_url)) = to {
        if !is_valid_to(to_url) {
            found.push((to_pos, UrlsError::BadTo(to_url.to_owned())));
            ok = false;
        } else if Link::of(to_url) == Link::Top {
            let to_path = without_anchor(to_url);
            if from.is_some_and(|(_, from_url)| from_url == to_path) {
                found.push((to_pos, UrlsError::SelfRedirect(to_path.to_owned())));
                ok = false;
            }
        }
    } else {
        ok = false;
    }
    let ((from_pos, from), (to_pos, to)) = (from?, to?);
    ok.then(|| Redirect {
        from_pos,
        from: from.to_owned(),
        to_pos,
        to: to.to_owned(),
    })
}

/// `url` before its `#` fragment, if any.
fn without_anchor(url: &str) -> &str {
    url.split('#').next().unwrap_or_default()
}

/// Whether `url` can be a redirect's `from`: `/name/` or `/a/b/`, with one or more lowercase
/// kebab-case segments.
fn is_redirect_from(url: &str) -> bool {
    let Some(rest) = url
        .strip_prefix('/')
        .and_then(|rest| rest.strip_suffix('/'))
    else {
        return false;
    };
    !rest.is_empty() && rest.split('/').all(page::is_slug)
}

/// Whether `to` can be a redirect's target: an `https://` or `http://` URL with a host, or a
/// root-relative site path starting with `/`, with no spaces or control characters.
fn is_valid_to(to: &str) -> bool {
    if to.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return false;
    }
    match Link::of(to) {
        Link::Top => true,
        Link::Away => to
            .strip_prefix("https://")
            .or_else(|| to.strip_prefix("http://"))
            .is_some_and(|rest| !rest.is_empty() && !rest.starts_with('/')),
        Link::Here | Link::Other => false,
    }
}

impl fmt::Display for UrlsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownKey(key) => {
                write!(f, "`urls.yaml` has `keep` and `redirects`, not `{key}`")
            }
            Self::NotKeepList => {
                f.write_str("`keep` is a list of addresses on this site, one `- /path/` per line")
            }
            Self::NotSiteUrl(url) => write!(
                f,
                "`{url}` is not an address on this site; start with `/`, as in `/news/`"
            ),
            Self::DuplicateKeep { url, first_line } => {
                write!(f, "duplicate address `{url}`, first on line {first_line}")
            }
            Self::KeepIsRedirect { url, redirect_line } => write!(
                f,
                "`{url}` is already in `redirects` on line {redirect_line}; list an address in \
                 `keep` or in `redirects`, not both"
            ),
            Self::NotRedirectList => f.write_str("`redirects` is a list of `from` and `to` pairs"),
            Self::UnknownRedirectKey(key) => {
                write!(f, "a redirect has `from` and `to`, not `{key}`")
            }
            Self::NotText => f.write_str("this value must be text"),
            Self::MissingFrom => {
                f.write_str("a redirect needs `from`, the old address such as `/zulip/`")
            }
            Self::MissingTo => f.write_str("a redirect needs `to`, the address it forwards to"),
            Self::BadFrom(url) => write!(
                f,
                "`{url}` is not a page address; write `/name/` with lowercase letters and digits \
                 joined by `-`, such as `/zulip/`"
            ),
            Self::DuplicateRedirect { url, first_line } => {
                write!(
                    f,
                    "duplicate redirect for `{url}`, first on line {first_line}"
                )
            }
            Self::PageExists { url, page } => write!(
                f,
                "`{url}` is already the page {}; delete that page or choose another `from`",
                page.display()
            ),
            Self::BadTo(url) => write!(
                f,
                "`{url}` cannot be a redirect target; start with `https://`, `http://`, or `/` \
                 for an address on this site"
            ),
            Self::SelfRedirect(url) => write!(f, "`{url}` redirects to itself"),
            Self::RedirectChain(url) => {
                write!(
                    f,
                    "`{url}` is itself a redirect; point `to` straight at the destination"
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_urls(text: &str) -> (Urls, Vec<ContentError>) {
        let pages = [
            PathBuf::from("content/_index.md"),
            PathBuf::from("content/news/_index.md"),
        ];
        let section_redirects = ["/_chat/".to_owned()];
        let mut errors = Vec::new();
        let urls = parse(text, &pages, &section_redirects, &mut errors);
        (urls, errors)
    }

    fn one_error(text: &str) -> (usize, usize, UrlsError) {
        let (_, mut errors) = parse_urls(text);
        assert_eq!(errors.len(), 1, "{text}\n{errors:?}");
        let ContentError { path, error } = errors.pop().unwrap();
        assert_eq!(path, Path::new("urls.yaml"));
        let PageError::Urls(pos, kind) = error else {
            panic!("expected Urls error, got {error:?}");
        };
        (pos.line, pos.col, kind)
    }

    #[test]
    fn keep_and_redirects_parse_with_comments_or_alone() {
        let text = "\
# Preserved URLs and redirects.
keep:
  - /
  - /atom.xml # feed
  - /news/#latest

redirects:
  - from: /zulip/
    to: https://rust-edu.zulip.cs.pdx.edu
  - from: /news/old-post/
    to: /news/#latest
";
        let (urls, errors) = parse_urls(text);
        assert!(errors.is_empty(), "{errors:?}");
        let kept: Vec<&str> = urls.keep.iter().map(|(_, url)| url.as_str()).collect();
        assert_eq!(kept, ["/", "/atom.xml", "/news/#latest"]);
        assert_eq!(urls.redirects.len(), 2);
        assert_eq!(
            (&urls.redirects[0].from, &urls.redirects[0].to),
            (
                &"/zulip/".to_owned(),
                &"https://rust-edu.zulip.cs.pdx.edu".to_owned()
            )
        );
        assert_eq!(
            (&urls.redirects[1].from, &urls.redirects[1].to),
            (&"/news/old-post/".to_owned(), &"/news/#latest".to_owned())
        );
        let (empty, empty_errors) = parse_urls("# Empty.\n");
        assert!(empty_errors.is_empty());
        assert_eq!(empty, Urls::default());
    }

    #[test]
    fn unknown_keys_and_wrong_shapes_are_errors_at_their_position() {
        assert_eq!(
            one_error("pages:\n  - /\n"),
            (1, 1, UrlsError::UnknownKey("pages".into()))
        );
        assert_eq!(one_error("keep: /\n"), (1, 7, UrlsError::NotKeepList));
        assert_eq!(
            one_error("keep:\n  - url: /\n"),
            (2, 5, UrlsError::NotKeepList)
        );
        assert_eq!(
            one_error("redirects: /zulip/\n"),
            (1, 12, UrlsError::NotRedirectList)
        );
        assert_eq!(
            one_error("redirects:\n  - /zulip/\n"),
            (2, 5, UrlsError::NotRedirectList)
        );
        assert_eq!(
            one_error("redirects:\n  - from: /zulip/\n    to: https://a.org\n    code: 301\n"),
            (4, 5, UrlsError::UnknownRedirectKey("code".into()))
        );
        // Non-text `from` or `to` reports `NotText` without a second `MissingFrom`/`MissingTo`.
        assert_eq!(
            one_error("redirects:\n  - from:\n      - /zulip/\n    to: https://a.org\n"),
            (3, 7, UrlsError::NotText)
        );
        assert_eq!(
            one_error("redirects:\n  - from: /zulip/\n    to:\n      - https://a.org\n"),
            (4, 7, UrlsError::NotText)
        );
        assert_eq!(
            one_error("redirects:\n  - to: https://a.org\n"),
            (2, 5, UrlsError::MissingFrom)
        );
        assert_eq!(
            one_error("redirects:\n  - from: /zulip/\n"),
            (2, 5, UrlsError::MissingTo)
        );
    }

    #[test]
    fn keep_requires_unique_site_paths_not_in_redirects() {
        for bad in ["https://a.org/", "\"#part\"", "news/", "//a.org/", "\"\""] {
            let text = format!("keep:\n  - {bad}\n");
            let unquoted = bad.trim_matches('"');
            assert_eq!(
                one_error(&text),
                (2, 5, UrlsError::NotSiteUrl(unquoted.into())),
                "{bad}"
            );
        }
        assert_eq!(
            one_error("keep:\n  - /news/\n  - /news/\n"),
            (
                3,
                5,
                UrlsError::DuplicateKeep {
                    url: "/news/".into(),
                    first_line: 2,
                }
            )
        );
        let both = "keep:\n  - /zulip/#top\nredirects:\n  - from: /zulip/\n    to: https://a.org\n";
        assert_eq!(
            one_error(both),
            (
                2,
                5,
                UrlsError::KeepIsRedirect {
                    url: "/zulip/#top".into(),
                    redirect_line: 4,
                }
            )
        );
    }

    #[test]
    fn redirects_reject_bad_addresses_clashes_loops_and_chains() {
        let bad_froms = [
            "/",
            "/zulip",
            "zulip/",
            "/Bad/",
            "/../a/",
            "/a.pdf/",
            "//a/",
            "/zulip/#top",
        ];
        for bad in bad_froms {
            let text = format!("redirects:\n  - from: \"{bad}\"\n    to: https://a.org\n");
            assert_eq!(
                one_error(&text),
                (2, 11, UrlsError::BadFrom(bad.into())),
                "{bad}"
            );
        }
        let duplicate = concat!(
            "redirects:\n",
            "  - from: /zulip/\n    to: https://a.org\n",
            "  - from: /zulip/\n    to: https://b.org\n",
        );
        assert_eq!(
            one_error(duplicate),
            (
                4,
                11,
                UrlsError::DuplicateRedirect {
                    url: "/zulip/".into(),
                    first_line: 2,
                }
            )
        );
        assert_eq!(
            one_error("redirects:\n  - from: /news/\n    to: https://a.org\n"),
            (
                2,
                11,
                UrlsError::PageExists {
                    url: "/news/".into(),
                    page: PathBuf::from("content/news/_index.md"),
                }
            )
        );
        let bad_tos = [
            "",
            "rust-edu.org",
            "mailto:a@b.org",
            "#part",
            "https://",
            "https:///a",
            "https://a.org/ b",
            "//a.org/",
        ];
        for bad in bad_tos {
            let text = format!("redirects:\n  - from: /zulip/\n    to: \"{bad}\"\n");
            assert_eq!(
                one_error(&text),
                (3, 9, UrlsError::BadTo(bad.into())),
                "{bad}"
            );
        }
        assert_eq!(
            one_error("redirects:\n  - from: /zulip/\n    to: /zulip/#top\n"),
            (3, 9, UrlsError::SelfRedirect("/zulip/".into()))
        );
        let chain = concat!(
            "redirects:\n",
            "  - from: /first/\n    to: /second/#part\n",
            "  - from: /second/\n    to: https://a.org\n",
        );
        assert_eq!(
            one_error(chain),
            (3, 9, UrlsError::RedirectChain("/second/#part".into()))
        );
        let section_chain = "redirects:\n  - from: /first/\n    to: /_chat/\n";
        let mut errors = Vec::new();
        let section_redirects = ["/chat/".to_owned()];
        let text = section_chain.replace("/_chat/", "/chat/");
        parse(&text, &[], &section_redirects, &mut errors);
        assert_eq!(errors.len(), 1);
        assert!(
            matches!(
                &errors[0].error,
                PageError::Urls(_, UrlsError::RedirectChain(url)) if url == "/chat/"
            ),
            "{errors:?}"
        );
    }

    #[test]
    fn broken_targets_in_keep_and_redirects_report_in_line_order() {
        let text = concat!(
            "redirects:\n",
            "  - from: /first/\n    to: /missing-first/\n",
            "keep:\n  - /missing-middle/\n",
        );
        let (mut urls, mut errors) = parse_urls(text);
        assert!(errors.is_empty(), "{errors:?}");
        urls.redirects.push(Redirect {
            from_pos: Pos { line: 7, col: 11 },
            from: "/last/".into(),
            to_pos: Pos { line: 8, col: 9 },
            to: "/missing-last/".into(),
        });
        let targets = BTreeMap::new();
        urls.check(&targets, &mut errors);
        let formatted: Vec<String> = errors.iter().map(ToString::to_string).collect();
        let expected = [
            "urls.yaml:3:9: error: `/missing-first/` is not a page or file of the site",
            "urls.yaml:5:5: error: `/missing-middle/` is not a page or file of the site",
            "urls.yaml:8:9: error: `/missing-last/` is not a page or file of the site",
        ];
        assert_eq!(formatted, expected);
    }

    #[test]
    fn a_damaged_urls_file_is_never_a_panic() {
        let sample: Vec<char> = "\
keep:
  - /
  - /news/#part
redirects:
  - from: /zulip/
    to: https://example.org/chat
  - from: /old/
    to: /news/
"
        .chars()
        .collect();
        let damage = [
            ':', '-', '#', '"', '\'', '/', '[', ']', '{', '}', '\\', '\n', ' ', 'é',
        ];
        let pages = [PathBuf::from("content/_index.md")];
        let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
        for _ in 0..5_000 {
            let mut text = sample.clone();
            for _ in 0..2 {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                let [low, high, how, what, ..] = state.to_le_bytes();
                let at = usize::from(u16::from_le_bytes([low, high])) % text.len();
                let c = damage[usize::from(what) % damage.len()];
                match how % 3 {
                    0 => text[at] = c,
                    1 => text.insert(at, c),
                    _ => _ = text.remove(at),
                }
            }
            let text: String = text.into_iter().collect();
            let mut errors = Vec::new();
            let _ = parse(&text, &pages, &[], &mut errors);
        }
    }
}
