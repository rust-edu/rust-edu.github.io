//! A page of `content/`: the values of its header, each checked, and its text as HTML.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use edu_yaml::{Entry, Pos};

use crate::check::{ContentError, PageError, ValueError};

/// The top of a page, where an error without a line of its own points.
pub(crate) const TOP: Pos = Pos { line: 1, col: 1 };

/// What a file of `content/` is: the home page `content/_index.md`, the page of a section (any
/// other `_index.md`), or a page of its own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Home,
    Section,
    Page,
}

impl Kind {
    fn of(path: &Path) -> Self {
        if path.file_name() != Some(OsStr::new("_index.md")) {
            Self::Page
        } else if path.parent() == Some(Path::new("content")) {
            Self::Home
        } else {
            Self::Section
        }
    }

    /// The layout of a page of this kind without a `template`.
    pub(crate) fn layout(self) -> &'static str {
        match self {
            Self::Home => "home.html",
            Self::Section => "section.html",
            Self::Page => "page.html",
        }
    }
}

/// The order of a section's pages: by URL, or with `sort_by: date` newest first.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Order {
    Url,
    Date,
}

/// A day of the calendar: as written, `2027-03-01`, and as the pages show it, `Mar 1, 2027`.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Date {
    pub(crate) text: String,
    pub(crate) shown: String,
}

/// The months as the pages show them.
const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

impl Date {
    /// `text` as a day, if it is `YYYY-MM-DD` and a day of the calendar.
    fn parse(text: &str) -> Option<Self> {
        let digits =
            |part: &str, len| part.len() == len && part.bytes().all(|b| b.is_ascii_digit());
        let mut parts = text.split('-');
        let (year, month, day) = (parts.next()?, parts.next()?, parts.next()?);
        if parts.next().is_some() || !digits(year, 4) || !digits(month, 2) || !digits(day, 2) {
            return None;
        }
        let (year, month, day): (u32, usize, u8) =
            (year.parse().ok()?, month.parse().ok()?, day.parse().ok()?);
        let name = MONTHS.get(month.checked_sub(1)?)?;
        let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
        let last = match month {
            2 if leap => 29,
            2 => 28,
            4 | 6 | 9 | 11 => 30,
            _ => 31,
        };
        let shown = format!("{name} {day}, {year}");
        (1..=last).contains(&day).then(|| Self {
            text: text.to_owned(),
            shown,
        })
    }
}

/// A page of `content/`: its path from the site root, the values of its header, and its text as
/// HTML.
#[derive(Debug)]
pub(crate) struct Page {
    pub(crate) path: PathBuf,
    pub(crate) kind: Kind,
    pub(crate) url: String,
    pub(crate) title: Option<String>,
    pub(crate) date: Option<Date>,
    pub(crate) authors: Vec<String>,
    /// The layout that `template` names, and where its name is.
    pub(crate) template: Option<(String, Pos)>,
    pub(crate) order: Order,
    pub(crate) redirect_to: Option<String>,
    pub(crate) draft: bool,
    pub(crate) html: String,
}

impl Page {
    /// Reads `text`, the page at `path`. The error is the first one: in the header, or else in
    /// the text, at a line counted from the top of the page.
    pub(crate) fn read(path: &Path, text: &str) -> Result<Self, PageError> {
        let url = url(path).map_err(|name| PageError::Value(TOP, ValueError::BadName(name)))?;
        let (header, body) = edu_yaml::parse_front_matter(text).map_err(PageError::Header)?;
        let mut page = Self {
            path: path.to_owned(),
            kind: Kind::of(path),
            url,
            title: None,
            date: None,
            authors: Vec::new(),
            template: None,
            order: Order::Url,
            redirect_to: None,
            draft: false,
            html: String::new(),
        };
        for entry in &header {
            page.set(entry)
                .map_err(|(pos, error)| PageError::Value(pos, error))?;
        }
        if page.title.is_none() && page.redirect_to.is_none() {
            return Err(PageError::Value(TOP, ValueError::NoTitle));
        }
        page.html = edu_markdown::to_html(body).map_err(|mut error| {
            // The text starts on the line after the header's closing `---`.
            let lines = text.get(..text.len() - body.len()).unwrap_or_default();
            error.pos.line += lines.matches('\n').count();
            PageError::Text(error)
        })?;
        Ok(page)
    }

    /// Takes the value of `entry`: `title`, `template` and `draft` in every page, `date` and
    /// `authors` in a page of its own, and `sort_by` and `redirect_to` in an `_index.md`.
    fn set(&mut self, entry: &Entry) -> Result<(), (Pos, ValueError)> {
        let pos = entry.value.pos;
        match (entry.key.as_str(), self.kind) {
            ("title", _) => self.title = Some(text(entry)?.to_owned()),
            ("template", _) => self.template = Some((text(entry)?.to_owned(), pos)),
            ("draft", _) => {
                self.draft = match text(entry)? {
                    "true" => true,
                    "false" => false,
                    _ => return Err((pos, ValueError::BadDraft)),
                };
                if self.draft && self.kind == Kind::Home {
                    return Err((pos, ValueError::DraftHome));
                }
            }
            ("date", Kind::Page) => {
                let date = Date::parse(text(entry)?).ok_or((pos, ValueError::BadDate))?;
                self.date = Some(date);
            }
            ("authors", Kind::Page) => {
                self.authors = names(entry).ok_or((pos, ValueError::NotNames))?;
            }
            ("sort_by", Kind::Home | Kind::Section) => match text(entry)? {
                "date" => self.order = Order::Date,
                _ => return Err((pos, ValueError::BadSortBy)),
            },
            ("redirect_to", Kind::Home | Kind::Section) => {
                let to = text(entry)?;
                if !(to.starts_with("https://") || to.starts_with("http://")) {
                    return Err((pos, ValueError::BadRedirect));
                }
                self.redirect_to = Some(to.to_owned());
            }
            (key, Kind::Page) => return Err((entry.pos, ValueError::NotPageKey(key.to_owned()))),
            (key, Kind::Home | Kind::Section) => {
                return Err((entry.pos, ValueError::NotSectionKey(key.to_owned())));
            }
        }
        Ok(())
    }

    /// An error in this page's header, at `pos`.
    pub(crate) fn error(&self, pos: Pos, kind: ValueError) -> ContentError {
        let (path, error) = (self.path.clone(), PageError::Value(pos, kind));
        ContentError { path, error }
    }
}

/// `/` for `content/_index.md`, `/news/` for `content/news/_index.md`, and `/news/a/` for
/// `content/news/a.md`; or the first name in the path that Zola would change to make the URL.
fn url(path: &Path) -> Result<String, String> {
    let path = path.strip_prefix("content").unwrap_or(path);
    let mut url = String::from("/");
    for name in path.with_extension("").iter() {
        let name = name.to_string_lossy();
        if name == "_index" {
            continue;
        }
        if !is_slug(&name) {
            return Err(name.into_owned());
        }
        url.push_str(&name);
        url.push('/');
    }
    Ok(url)
}

/// Whether Zola keeps `name` as it is in a URL: lowercase ASCII letters and digits, with single
/// `-` between them.
fn is_slug(name: &str) -> bool {
    let is_part = |part: &str| {
        let is_letter = |b: u8| b.is_ascii_lowercase() || b.is_ascii_digit();
        !part.is_empty() && part.bytes().all(is_letter)
    };
    name.split('-').all(is_part)
}

/// The text of `entry`'s value, or an error at the value.
fn text(entry: &Entry) -> Result<&str, (Pos, ValueError)> {
    match &entry.value.value {
        edu_yaml::Value::Str(text) => Ok(text),
        edu_yaml::Value::List(_) | edu_yaml::Value::Map(_) => {
            Err((entry.value.pos, ValueError::NotText))
        }
    }
}

/// The names of `authors`, a list of texts.
fn names(entry: &Entry) -> Option<Vec<String>> {
    let edu_yaml::Value::List(items) = &entry.value.value else {
        return None;
    };
    let names = items.iter().map(|item| match &item.value {
        edu_yaml::Value::Str(name) => Some(name.clone()),
        edu_yaml::Value::List(_) | edu_yaml::Value::Map(_) => None,
    });
    names.collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The page at `path` under `content/` with the header `header`. The test site `unrenderable`
    /// has the errors.
    fn page(path: &str, header: &str) -> Page {
        let path = Path::new("content").join(path);
        Page::read(&path, &format!("---\n{header}---\n")).unwrap()
    }

    #[test]
    fn a_page_is_the_home_page_a_section_or_a_page_of_its_own() {
        let cases = [
            ("_index.md", Kind::Home, "/"),
            ("news/_index.md", Kind::Section, "/news/"),
            ("news/a.md", Kind::Page, "/news/a/"),
            ("index.md", Kind::Page, "/index/"),
        ];
        for (path, kind, url) in cases {
            let page = page(path, "title: T\n");
            assert_eq!((page.kind, page.url.as_str()), (kind, url), "{path}");
        }
    }

    #[test]
    fn a_url_is_the_names_of_the_path_as_zola_keeps_them() {
        for name in ["a", "rust-week-2027", "2027", "cfp-2022"] {
            assert!(is_slug(name), "{name}");
        }
        for name in [
            "", "A", "Rust", "a_b", "a--b", "-a", "a-", "a b", "é", "a.b",
        ] {
            assert!(!is_slug(name), "{name}");
        }
        let path = |path: &str| url(&Path::new("content").join(path));
        assert_eq!(path("news/Rust Week.md"), Err("Rust Week".to_owned()));
        assert_eq!(path("Old_News/a.md"), Err("Old_News".to_owned()));
    }

    #[test]
    fn a_header_holds_the_values_of_its_kind_of_page() {
        let header = "title: A\ndate: 2022-07-11\nauthors:\n  - B\n  - C\ntemplate: list.html\n";
        let a = page("news/a.md", &format!("{header}draft: true\n"));
        assert_eq!((a.title.as_deref(), a.draft), (Some("A"), true));
        let date = a.date.unwrap();
        assert_eq!(
            (date.text.as_str(), date.shown.as_str()),
            ("2022-07-11", "Jul 11, 2022")
        );
        assert_eq!(a.authors, ["B", "C"]);
        let template = a.template.unwrap();
        let at = Pos { line: 7, col: 11 };
        assert_eq!((template.0.as_str(), template.1), ("list.html", at));
        let section = "title: News\nsort_by: date\ndraft: false\n";
        let news = page("news/_index.md", section);
        assert_eq!(
            (news.order, news.redirect_to, news.draft),
            (Order::Date, None, false)
        );
        let chat = page("chat/_index.md", "redirect_to: https://example.org/\n");
        assert_eq!(chat.redirect_to.as_deref(), Some("https://example.org/"));
        assert_eq!((chat.order, chat.title), (Order::Url, None));
        let home = page(
            "_index.md",
            "sort_by: date\nredirect_to: http://a.org/\ndraft: false\n",
        );
        assert_eq!(
            (home.order, home.redirect_to.as_deref()),
            (Order::Date, Some("http://a.org/"))
        );
    }

    #[test]
    fn a_day_of_the_calendar_is_a_date() {
        let days = [
            ("2022-07-11", "Jul 11, 2022"),
            ("2025-06-01", "Jun 1, 2025"),
            ("2024-02-29", "Feb 29, 2024"),
            ("2000-02-29", "Feb 29, 2000"),
            ("1999-12-31", "Dec 31, 1999"),
            ("2024-12-31", "Dec 31, 2024"),
        ];
        for (text, shown) in days {
            let date = Date::parse(text).unwrap();
            assert_eq!((date.text.as_str(), date.shown.as_str()), (text, shown));
        }
        let not_days = "2023-02-29 1900-02-29 2022-04-31 2022-13-01 2022-00-10 2022-01-00 \
                        2022-7-11 2022-07-1 22-07-11 2022-07-11-01 2022-07 +202-07-11";
        for text in not_days.split(' ') {
            assert_eq!(Date::parse(text), None, "{text}");
        }
    }
}
