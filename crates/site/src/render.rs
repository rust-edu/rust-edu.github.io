//! The settings of `site.yaml`, the layouts of `layouts/`, and what each page of `content/`
//! renders with: its layout and the values that the layout reads.

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use edu_template::{Template, Value};
use edu_yaml::{Entry, Node};

use crate::Error;
use crate::check::{ContentError, PageError, ValueError, read};
use crate::page::{Kind, Order, Page, TOP};

/// The layout that wraps every page's own layout, at `{{ content | safe }}`.
const LAYOUT: &str = "layout.html";

/// The settings of `site.yaml` in `root`, as the value `site`; an error in them goes to `errors`.
pub(crate) fn site(root: &Path, errors: &mut Vec<ContentError>) -> Result<Value, Error> {
    let path = PathBuf::from("site.yaml");
    match edu_yaml::parse(&read(root, &path)?) {
        Ok(entries) => Ok(map(&entries)),
        Err(error) => {
            let error = PageError::Header(error);
            errors.push(ContentError { path, error });
            Ok(Value::Map(Vec::new()))
        }
    }
}

fn map(entries: &[Entry]) -> Value {
    let values = entries
        .iter()
        .map(|entry| (entry.key.clone(), value(&entry.value)));
    Value::Map(values.collect())
}

fn value(node: &Node) -> Value {
    match &node.value {
        edu_yaml::Value::Str(text) => Value::Str(text.clone()),
        edu_yaml::Value::List(items) => Value::List(items.iter().map(value).collect()),
        edu_yaml::Value::Map(entries) => map(entries),
    }
}

/// The layouts of `layouts/`, parsed, by file name.
pub(crate) struct Layouts(Vec<(String, Template)>);

impl Layouts {
    /// Parses every `.html` file of `layouts/` in `root`, which must have `layout.html`, or gives
    /// `None` after an error in a layout, which goes to `errors`.
    pub(crate) fn read(root: &Path, errors: &mut Vec<ContentError>) -> Result<Option<Self>, Error> {
        let dir = Path::new("layouts");
        let read_error = |source| Error::Read {
            path: dir.to_owned(),
            source,
        };
        let mut names = Vec::new();
        for entry in fs::read_dir(root.join(dir)).map_err(read_error)? {
            let name = entry.map_err(read_error)?.file_name();
            if Path::new(&name).extension() == Some(OsStr::new("html")) {
                names.push(name.to_string_lossy().into_owned());
            }
        }
        names.sort();
        if !names.iter().any(|name| name == LAYOUT) {
            read(root, &dir.join(LAYOUT))?;
        }
        let count = names.len();
        let mut layouts = Vec::new();
        for name in names {
            let path = dir.join(&name);
            match Template::parse(&read(root, &path)?) {
                Ok(template) => layouts.push((name, template)),
                Err(error) => {
                    let error = PageError::Layout(error);
                    errors.push(ContentError { path, error });
                }
            }
        }
        Ok((layouts.len() == count).then_some(Self(layouts)))
    }

    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    fn get(&self, name: &str) -> Option<&Template> {
        self.0
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, layout)| layout)
    }
}

/// What `page` renders with: its layout, and its values, which the layout reads as `page`. The
/// layout is the page's `template`, or else `home.html`, `section.html` or `page.html` by its kind.
pub(crate) fn prepare<'a>(
    page: &Page,
    pages: &[Page],
    layouts: &'a Layouts,
) -> Result<(&'a Template, Value), ContentError> {
    let (name, pos) = match &page.template {
        Some((name, pos)) => (name.as_str(), *pos),
        None => (page.kind.layout(), TOP),
    };
    let Some(layout) = layouts.get(name) else {
        return Err(page.error(pos, ValueError::NoLayout(name.to_owned())));
    };
    Ok((layout, values(page, pages)?))
}

/// The values of `page` for the layouts, and for a section's page also `pages`, the pages in its
/// directory.
fn values(page: &Page, pages: &[Page]) -> Result<Value, ContentError> {
    let mut values = own_values(page);
    match page.kind {
        Kind::Home | Kind::Section => {
            let listed = listing(page, pages)?;
            let listed = listed.into_iter().map(|page| Value::Map(own_values(page)));
            values.push(("pages".to_owned(), Value::List(listed.collect())));
        }
        Kind::Page => {}
    }
    Ok(Value::Map(values))
}

/// The values of `page` for the layouts: `title`, `url` and `content`; `date` with `date_display`
/// and `authors` joined by `and`, if the header has them; and `home` for the home page.
fn own_values(page: &Page) -> Vec<(String, Value)> {
    let text = |text: &str| Value::Str(text.to_owned());
    let mut values = Vec::new();
    if let Some(title) = &page.title {
        values.push(("title", text(title)));
    }
    values.push(("url", text(&page.url)));
    values.push(("content", text(&page.html)));
    if let Some(date) = &page.date {
        values.push(("date", text(&date.text)));
        values.push(("date_display", text(&date.shown)));
    }
    if !page.authors.is_empty() {
        values.push(("authors", Value::Str(page.authors.join(" and "))));
    }
    match page.kind {
        Kind::Home => values.push(("home", Value::Bool(true))),
        Kind::Section | Kind::Page => {}
    }
    let values = values
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value));
    values.collect()
}

/// The pages in the directory of `section`, without sections: by URL, or with `sort_by: date`
/// newest first and on the same day by URL from Z to A, as Zola lists them.
fn listing<'a>(section: &Page, pages: &'a [Page]) -> Result<Vec<&'a Page>, ContentError> {
    let dir = section.path.parent();
    let beside = |page: &&Page| page.kind == Kind::Page && page.path.parent() == dir;
    let mut listed: Vec<&Page> = pages.iter().filter(beside).collect();
    match section.order {
        Order::Url => listed.sort_by(|a, b| a.url.cmp(&b.url)),
        Order::Date => {
            if let Some(page) = listed.iter().find(|page| page.date.is_none()) {
                return Err(page.error(TOP, ValueError::NoDate));
            }
            listed.sort_by(|a, b| (&b.date, &b.url).cmp(&(&a.date, &a.url)));
        }
    }
    Ok(listed)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Pages of `texts`: each a path under `content/` and a page's text.
    fn pages(texts: &[(&str, &str)]) -> Vec<Page> {
        let page =
            |&(path, text): &(&str, &str)| Page::read(&Path::new("content").join(path), text);
        texts.iter().map(page).collect::<Result<_, _>>().unwrap()
    }

    /// Layouts of `texts`: each a file name and a layout's text.
    fn parse(texts: &[(&str, &str)]) -> Layouts {
        let layout =
            |&(name, text): &(&str, &str)| (name.to_owned(), Template::parse(text).unwrap());
        Layouts(texts.iter().map(layout).collect())
    }

    /// What the layout of the page at `path` shows of the page's values, or the error that keeps
    /// the page from rendering.
    fn show(pages: &[Page], path: &str, layouts: &Layouts) -> String {
        let page = pages
            .iter()
            .find(|page| page.path == Path::new("content").join(path));
        let shown = prepare(page.unwrap(), pages, layouts).map(|(layout, values)| {
            let context = Value::Map(vec![("page".into(), values)]);
            let shown = layout.render(&context);
            shown.unwrap_or_else(|error| format!("error {}", error.kind))
        });
        shown.unwrap_or_else(|error| format!("error {error}"))
    }

    #[test]
    fn a_page_shows_its_values_in_its_layout() {
        let home = "home{% if page.home %} {{ page.url }}{% endif %}";
        let page = "{{ page.authors }} on {{ page.date_display }} ({{ page.date }})";
        let list = "{{ page.url }}{% if page.home %} home{% endif %} {{ page.content | safe }}";
        let layouts = parse(&[
            ("home.html", home),
            ("page.html", page),
            ("list.html", list),
        ]);
        let a = "---\ntitle: A\ndate: 2025-06-01\nauthors:\n  - B\n  - C & D\n---\n";
        let pages = pages(&[
            ("_index.md", "---\ntitle: Home\n---\n"),
            ("news/a.md", a),
            ("b.md", "---\ntitle: B\ntemplate: list.html\n---\n*text*\n"),
            ("c.md", "---\ntitle: C\n---\n"),
        ]);
        let cases = [
            ("_index.md", "home /"),
            ("news/a.md", "B and C &amp; D on Jun 1, 2025 (2025-06-01)"),
            ("b.md", "/b/ <p><em>text</em></p>\n"),
            ("c.md", "error unknown variable `page.authors`"),
        ];
        for (path, expected) in cases {
            assert_eq!(show(&pages, path, &layouts), expected, "{path}");
        }
        let expected = "error content/_index.md:1:1: error: `layouts/home.html` does not exist";
        assert_eq!(show(&pages, "_index.md", &parse(&[])), expected);
    }

    #[test]
    fn a_section_lists_its_own_pages_by_url_or_newest_first() {
        let titles = "{% for post in page.pages %}{{ post.title }} {% endfor %}";
        let urls = "{% for post in page.pages %}{{ post.url }} {% endfor %}";
        let layouts = parse(&[("home.html", titles), ("section.html", urls)]);
        let pages = pages(&[
            ("_index.md", "---\ntitle: Home\n---\n"),
            ("b.md", "---\ntitle: B\ndate: 2027-01-01\n---\n"),
            ("a.md", "---\ntitle: A\ndate: 2020-01-01\n---\n"),
            ("news/_index.md", "---\ntitle: News\nsort_by: date\n---\n"),
            ("news/a.md", "---\ntitle: A\ndate: 2022-07-11\n---\n"),
            ("news/old.md", "---\ntitle: Old\ndate: 2021-12-31\n---\n"),
            ("news/z.md", "---\ntitle: Z\ndate: 2022-07-11\n---\n"),
            ("news/new.md", "---\ntitle: New\ndate: 2026-04-26\n---\n"),
            ("news/2027/a.md", "---\ntitle: A\ndate: 2027-01-01\n---\n"),
        ]);
        assert_eq!(show(&pages, "_index.md", &layouts), "A B ");
        let expected = "/news/new/ /news/z/ /news/a/ /news/old/ ";
        assert_eq!(show(&pages, "news/_index.md", &layouts), expected);
    }
}
