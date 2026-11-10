//! The links of the site: each `href` and `src` in the HTML of a page, and each address of
//! `urls.txt`, must go to a page or file of the built site, and a `#` to an id of its page.

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

use edu_yaml::Pos;

use crate::Error;
use crate::check::{ContentError, PageError, Rendered, read};
use crate::page::TOP;
use crate::render::Layouts;

/// The addresses that other sites link to, one per line, which the site must keep.
const URLS: &str = "urls.txt";

/// Why a link would not work on the site.
#[derive(Debug, PartialEq, Eq)]
pub enum LinkError {
    /// A link that does not start with `/`, `#`, `https://`, `http://` or `mailto:`, such as a
    /// relative link.
    BadLink(String),
    /// A link to a page or file that the site does not have.
    NoTarget(String),
    /// A link to a page without the `/` at the end of its URL, written with it.
    NoSlash(String),
    /// A link to an id that its page does not have.
    NoAnchor(String),
    /// A link from a page of the live site to a draft, or to a file in the folder of one.
    ToDraft(String),
}

/// What a link goes to, by how it starts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Link {
    /// Another site or an email address: `https://`, `http://` or `mailto:`.
    Away,
    /// An id of the same page: `#`.
    Here,
    /// A page or file of the site: a single `/`.
    Top,
    /// Anything else, such as a relative link, which the check refuses.
    Other,
}

impl Link {
    /// What `link`, as written, goes to; another site's address may hold a `#` or a `/` of its own.
    pub(crate) fn of(link: &str) -> Self {
        if ["https://", "http://", "mailto:"]
            .iter()
            .any(|start| link.starts_with(start))
        {
            Self::Away
        } else if link.starts_with('#') {
            Self::Here
        } else if link.starts_with('/') && !link.starts_with("//") {
            Self::Top
        } else {
            Self::Other
        }
    }
}

/// A file of the built site, for the links to it: whether the live site has it, and for a page,
/// the ids of its elements.
pub(crate) struct Target<'a> {
    pub(crate) live: bool,
    pub(crate) ids: Option<Vec<&'a str>>,
}

/// Checks each link of `pages`, and each address of `urls.txt` in `root` if there is one, against
/// `targets`, the files of the built site by their path in it. An error goes to `errors` once, at
/// the first place where its link is written: the page, `site.yaml` or one of `layouts`.
pub(crate) fn check(
    root: &Path,
    pages: &[Rendered],
    targets: &BTreeMap<String, Target<'_>>,
    layouts: &Layouts,
    errors: &mut Vec<ContentError>,
) -> Result<(), Error> {
    for page in pages {
        let file = key(&page.url);
        for (_, link) in links(&page.html) {
            let link = unescape(link);
            let Err(kind) = resolve(&link, &file, page.live, targets) else {
                continue;
            };
            let (path, pos) = locate(root, &page.path, layouts, &link)?;
            let error = ContentError {
                path,
                error: PageError::Link(pos, kind),
            };
            if !errors
                .iter()
                .any(|seen| seen.to_string() == error.to_string())
            {
                errors.push(error);
            }
        }
    }
    let path = Path::new(URLS);
    if !root.join(path).is_file() {
        return Ok(());
    }
    let text = read(root, path)?;
    for (line, url) in text.lines().enumerate() {
        if url.is_empty() || url.starts_with('#') {
            continue;
        }
        if let Err(kind) = resolve(url, "", true, targets) {
            let pos = Pos {
                line: line + 1,
                col: 1,
            };
            let error = PageError::Link(pos, kind);
            errors.push(ContentError {
                path: path.to_owned(),
                error,
            });
        }
    }
    Ok(())
}

/// The file of the page at `url` in the built site: `index.html` for `/`, `news/index.html` for
/// `/news/`.
pub(crate) fn key(url: &str) -> String {
    format!("{}index.html", url.trim_start_matches('/'))
}

/// Whether `link`, on the page whose file in the built site is `page`, goes to a file of
/// `targets`; from a page of the live site, `live`, only to one that the live site has.
fn resolve(
    link: &str,
    page: &str,
    live: bool,
    targets: &BTreeMap<String, Target<'_>>,
) -> Result<(), LinkError> {
    let (path, id) = match link.split_once('#') {
        Some((path, id)) => (path, Some(id)),
        None => (link, None),
    };
    let file = match Link::of(link) {
        Link::Away => return Ok(()),
        Link::Here => page.to_owned(),
        Link::Top => {
            let rest = path.get(1..).unwrap_or_default();
            let rest = decode(rest).ok_or_else(|| LinkError::NoTarget(written(link)))?;
            if rest.is_empty() || rest.ends_with('/') {
                format!("{rest}index.html")
            } else {
                rest
            }
        }
        Link::Other => return Err(LinkError::BadLink(written(link))),
    };
    let Some(target) = targets.get(&file) else {
        if targets.contains_key(&format!("{file}/index.html")) {
            let id = id.map(|id| format!("#{id}")).unwrap_or_default();
            return Err(LinkError::NoSlash(written(&format!("{path}/{id}"))));
        }
        return Err(LinkError::NoTarget(written(link)));
    };
    if live && !target.live {
        return Err(LinkError::ToDraft(written(link)));
    }
    // An id keeps the letters of its heading, which the link writes outside ASCII as `%` escapes.
    let has_id = |ids: &Vec<&str>, id| decode(id).is_some_and(|id| ids.contains(&id.as_str()));
    match (id, &target.ids) {
        (Some(id), Some(ids)) if !has_id(ids, id) => Err(LinkError::NoAnchor(written(link))),
        _ => Ok(()),
    }
}

/// `link` as a page writes it: the HTML of the site escapes each character outside ASCII as `%`
/// and two hex digits, which this turns back, unless that makes a control character.
fn written(link: &str) -> String {
    let decoded = decode(link).filter(|text| !text.contains(char::is_control));
    decoded.unwrap_or_else(|| link.to_owned())
}

/// Where `link` is written first, with its characters outside ASCII or as their `%` escapes: in
/// the text of `page`, as `[text](link)` or `<link>`; or else in `site.yaml` or a layout; or
/// else at the top of `page`.
fn locate(
    root: &Path,
    page: &Path,
    layouts: &Layouts,
    link: &str,
) -> Result<(PathBuf, Pos), Error> {
    let text = read(root, page)?;
    let forms = [written(link), link.to_owned()];
    let found = forms.iter().find_map(|form| {
        let ways = [(format!("]({form})"), 2), (format!("<{form}>"), 1)];
        ways.iter()
            .find_map(|(way, skip)| text.find(way.as_str()).map(|at| at + skip))
    });
    if let Some(at) = found {
        return Ok((page.to_owned(), pos(&text, at)));
    }
    let layouts = layouts.names().map(|name| Path::new("layouts").join(name));
    for file in std::iter::once(PathBuf::from("site.yaml")).chain(layouts) {
        let text = read(root, &file)?;
        if let Some(at) = forms.iter().find_map(|form| text.find(form.as_str())) {
            return Ok((file, pos(&text, at)));
        }
    }
    Ok((page.to_owned(), TOP))
}

/// The line and column of the byte `at` of `text`.
fn pos(text: &str, at: usize) -> Pos {
    let before = text.get(..at).unwrap_or_default();
    let line = before.matches('\n').count() + 1;
    let col = before
        .rsplit('\n')
        .next()
        .unwrap_or_default()
        .chars()
        .count()
        + 1;
    Pos { line, col }
}

/// Each link of `html`, the value of an `href` or `src` as written, and where it starts.
pub(crate) fn links(html: &str) -> impl Iterator<Item = (usize, &str)> {
    let attributes = attributes(html).into_iter();
    let links = attributes.filter(|(name, ..)| matches!(*name, "href" | "src"));
    links.map(|(_, at, value)| (at, value))
}

/// `html` with the text that `prefix` gives for a link, if any, put before that link.
pub(crate) fn prefix<'a>(html: &str, prefix: impl Fn(&str) -> Option<&'a str>) -> String {
    let mut prefixed = String::with_capacity(html.len());
    let mut last = 0;
    for (at, link) in links(html) {
        if let Some(before) = prefix(link) {
            prefixed.push_str(html.get(last..at).unwrap_or_default());
            prefixed.push_str(before);
            last = at;
        }
    }
    prefixed.push_str(html.get(last..).unwrap_or_default());
    prefixed
}

/// The ids of the elements of `html`.
pub(crate) fn ids(html: &str) -> Vec<&str> {
    let attributes = attributes(html).into_iter();
    let ids = attributes.filter(|(name, ..)| *name == "id");
    ids.map(|(_, _, id)| id).collect()
}

/// The attributes in the tags of `html`: each name, where its value starts, and the value as
/// written. The site's HTML writes every value in `"` and every `<` of its text as `&lt;`, so a
/// tag starts at each `<`, and a `href="` in the text of a page is not an attribute.
fn attributes(html: &str) -> Vec<(&str, usize, &str)> {
    let mut attributes = Vec::new();
    let mut from = 0;
    while let Some(start) = html.get(from..).and_then(|rest| rest.find('<')) {
        let start = from + start;
        let Some(end) = html.get(start..).and_then(|rest| rest.find('>')) else {
            break;
        };
        let tag = html.get(start..start + end).unwrap_or_default();
        let mut at = 0;
        while let Some(equals) = tag.get(at..).and_then(|rest| rest.find("=\"")) {
            let (equals, value_at) = (at + equals, at + equals + 2);
            let before = tag.get(at..equals).unwrap_or_default();
            let name = before.rsplit(|c: char| c.is_ascii_whitespace()).next();
            let Some(len) = tag.get(value_at..).and_then(|rest| rest.find('"')) else {
                break;
            };
            let value = tag.get(value_at..value_at + len).unwrap_or_default();
            attributes.push((name.unwrap_or_default(), start + value_at, value));
            at = value_at + len + 1;
        }
        from = start + end + 1;
    }
    attributes
}

/// `value`, the value of an attribute as written, with each character that the HTML of the site
/// escapes.
fn unescape(value: &str) -> String {
    let escapes = [
        ("&quot;", "\""),
        ("&#39;", "'"),
        ("&#x27;", "'"),
        ("&lt;", "<"),
        ("&gt;", ">"),
        ("&amp;", "&"),
    ];
    let unescape = |text: String, (escaped, c): &(&str, &str)| text.replace(escaped, c);
    escapes.iter().fold(value.to_owned(), unescape)
}

/// `path` with each `%` and two hex digits as the byte that they stand for, if the bytes are UTF-8.
pub(crate) fn decode(path: &str) -> Option<String> {
    let mut bytes = Vec::with_capacity(path.len());
    let mut rest = path.as_bytes();
    while let Some((&byte, tail)) = rest.split_first() {
        rest = tail;
        if byte == b'%' {
            let (hex, tail) = rest.split_at_checked(2)?;
            let hex = std::str::from_utf8(hex).ok()?;
            bytes.push(u8::from_str_radix(hex, 16).ok()?);
            rest = tail;
        } else {
            bytes.push(byte);
        }
    }
    String::from_utf8(bytes).ok()
}

impl fmt::Display for LinkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadLink(link) => write!(
                f,
                "`{link}` cannot be checked: a link to a page or file of the site starts with \
                 `/`, as in `/news/`, and other links with `#`, `https://`, `http://` or `mailto:`"
            ),
            Self::NoTarget(link) => write!(f, "`{link}` is not a page or file of the site"),
            Self::NoSlash(link) => write!(f, "a link to a page ends with `/`: `{link}`"),
            Self::NoAnchor(link) => write!(f, "`{link}` goes to an id that its page does not have"),
            Self::ToDraft(link) => write!(
                f,
                "`{link}` goes to a draft, which the live site leaves out"
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_link_goes_to_a_file_of_the_site_and_its_id_or_is_an_error() {
        let target = |live, ids: Option<Vec<&'static str>>| Target { live, ids };
        let targets = BTreeMap::from([
            ("index.html".to_owned(), target(true, Some(vec!["main"]))),
            (
                "a/index.html".to_owned(),
                target(true, Some(vec!["part", "café"])),
            ),
            (
                "draft/index.html".to_owned(),
                target(false, Some(Vec::new())),
            ),
            ("a b.txt".to_owned(), target(true, None)),
        ]);
        let resolve = |link: &str| resolve(link, "index.html", true, &targets);
        let ok = "https://a.org/ http://a.org mailto:a@a.org #main / /a/ /a/#part /a%20b.txt \
                  /a%20b.txt#x /a/#caf%C3%A9";
        for link in ok.split(' ') {
            assert_eq!(resolve(link), Ok(()), "{link}");
        }
        let errors = [
            ("#part", LinkError::NoAnchor("#part".into())),
            ("/a/#x", LinkError::NoAnchor("/a/#x".into())),
            // An error shows a link with its characters outside ASCII, as the page writes it.
            ("/a/#caf%C3%A9s", LinkError::NoAnchor("/a/#cafés".into())),
            ("/caf%C3%A9/", LinkError::NoTarget("/café/".into())),
            ("/a", LinkError::NoSlash("/a/".into())),
            ("/a#part", LinkError::NoSlash("/a/#part".into())),
            ("/b/", LinkError::NoTarget("/b/".into())),
            ("/a%2", LinkError::NoTarget("/a%2".into())),
            ("/a%ff", LinkError::NoTarget("/a%ff".into())),
            ("/draft/", LinkError::ToDraft("/draft/".into())),
        ];
        for (link, error) in errors {
            assert_eq!(resolve(link), Err(error), "{link}");
        }
        let bad = [
            "",
            "a/",
            "../a/",
            "//a.org/",
            "email:a@a.org",
            "javascript:a",
            // Not shown with the line break that `%0A` stands for.
            "java%0Ascript:a",
        ];
        for link in bad {
            let error = LinkError::BadLink(link.to_owned());
            assert_eq!(resolve(link), Err(error), "{link}");
        }
        assert_eq!(
            super::resolve("/draft/", "a/index.html", false, &targets),
            Ok(())
        );
    }

    #[test]
    fn a_link_goes_away_to_an_id_here_to_the_top_or_elsewhere_by_how_it_starts() {
        let cases = [
            ("https://a.org/#b", Link::Away),
            ("http://a.org", Link::Away),
            ("mailto:a@a.org", Link::Away),
            ("#a", Link::Here),
            ("#/a", Link::Here),
            ("/", Link::Top),
            ("/a/#b", Link::Top),
            ("", Link::Other),
            ("a/#b", Link::Other),
            ("//a.org/", Link::Other),
            ("https:a", Link::Other),
        ];
        for (link, kind) in cases {
            assert_eq!(Link::of(link), kind, "{link}");
        }
    }

    #[test]
    fn an_attribute_is_in_a_tag_not_in_the_text() {
        let html = "<h2 id=\"a\">id=\"b\"</h2><p>&lt;i id=\"c\"&gt;</p>\
                    <main\nclass=\"x\"id=\"d\"><p data-id=\"e\">";
        assert_eq!(ids(html), ["a", "d"]);
        assert_eq!(
            unescape("/?a=&quot;1&quot;&amp;b=&#39;&#x27;&lt;&gt;&amp;lt;"),
            "/?a=\"1\"&b=''<>&lt;"
        );
    }
}
