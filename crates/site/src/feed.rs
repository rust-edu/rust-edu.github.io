//! The site's Atom feed, `atom.xml`, as RFC 4287 describes it: the pages that each section with
//! `sort_by: date` lists, newest first, each with its whole text, for feed readers.

use std::path::{Path, PathBuf};

use crate::build::BaseUrl;
use crate::check::{ContentError, PageError, ValueError};
use crate::links::{self, Link};
use crate::page::{Order, Page, TOP};
use crate::render::{self, Site};

/// The feed's file, at the top of the built site.
pub(crate) const FEED: &str = "atom.xml";

/// A page that a section with `sort_by: date` lists: its file and URL, its `title`, its `date` as
/// written and its `authors`, and its text as HTML.
#[derive(Debug)]
pub(crate) struct Post {
    pub(crate) path: PathBuf,
    pub(crate) url: String,
    title: String,
    date: String,
    authors: Vec<String>,
    pub(crate) body: String,
}

/// The feed: the site's `title` and `description`, the address of `site.yaml`, which names the
/// feed and its posts, the day of the newest post, and the posts, newest first.
#[derive(Debug)]
pub(crate) struct Feed {
    title: String,
    subtitle: Option<String>,
    id_base: BaseUrl,
    updated: String,
    posts: Vec<Post>,
}

/// The posts of `pages`: the pages that each section with `sort_by: date` and no `redirect_to`
/// lists, newest first, if the live site has them, as `is_live` says of their path. A list leaves
/// out a page with `draft: true`, but a preview also lists the pages in the folder of a draft
/// `_index.md`; without them, a preview's feed lists what the live feed lists.
pub(crate) fn posts(
    pages: &[Page],
    is_live: impl Fn(&Path) -> bool,
) -> Result<Vec<Post>, ContentError> {
    let mut listed = Vec::new();
    for section in pages {
        // Only the header of an `_index.md` has `sort_by`.
        match section.order {
            Order::Date if section.redirect_to.is_none() => {
                listed.extend(render::listing(section, pages)?);
            }
            Order::Date | Order::Url => {}
        }
    }
    listed.retain(|page| is_live(&page.path));
    listed.sort_by(render::newest_first);
    let mut posts = Vec::new();
    for page in listed {
        // A page of its own has a title, and `listing` gives only pages with a date.
        let title = page.title.clone();
        let title = title.ok_or_else(|| page.error(TOP, ValueError::NoTitle))?;
        let date = page.date.as_ref();
        let date = date.ok_or_else(|| page.error(TOP, ValueError::NoDate))?;
        posts.push(Post {
            path: page.path.clone(),
            url: page.url.clone(),
            title,
            date: date.text.clone(),
            authors: page.authors.clone(),
            body: page.html.clone(),
        });
    }
    Ok(posts)
}

/// The feed of `posts` with the settings of `site`, or `None` without posts. A site with posts
/// needs a `title`, the name of its feed.
pub(crate) fn feed(site: &Site, posts: Vec<Post>) -> Result<Option<Feed>, ContentError> {
    let Some(updated) = posts.iter().map(|post| post.date.clone()).max() else {
        return Ok(None);
    };
    let Some(title) = site.title.clone() else {
        let path = PathBuf::from("site.yaml");
        let error = PageError::Value(TOP, ValueError::NoSiteTitle);
        return Err(ContentError { path, error });
    };
    let (subtitle, id_base) = (site.description.clone(), site.id_base.clone());
    Ok(Some(Feed {
        title,
        subtitle,
        id_base,
        updated,
        posts,
    }))
}

/// The feed as Atom XML, with its links at `base_url` and its ids at the address of `site.yaml`,
/// which stay the same wherever the site is built for. A post's day is its time at midnight UTC.
pub(crate) fn atom(feed: &Feed, base_url: &BaseUrl) -> String {
    let (title, id_base, updated) = (&feed.title, &feed.id_base, &feed.updated);
    // In English, as the pages are.
    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <feed xmlns=\"http://www.w3.org/2005/Atom\" xml:lang=\"en\">\n",
    );
    xml.push_str(&format!("    <title>{}</title>\n", escaped(title)));
    if let Some(subtitle) = &feed.subtitle {
        xml.push_str(&format!("    <subtitle>{}</subtitle>\n", escaped(subtitle)));
    }
    // Neither an address nor a day has a character that XML escapes.
    let (atom, html) = ("type=\"application/atom+xml\"", "type=\"text/html\"");
    xml.push_str(&format!(
        "    <link rel=\"self\" {atom} href=\"{base_url}/{FEED}\"/>\n"
    ));
    xml.push_str(&format!(
        "    <link rel=\"alternate\" {html} href=\"{base_url}/\"/>\n"
    ));
    xml.push_str(&format!("    <id>{id_base}/{FEED}</id>\n"));
    xml.push_str(&format!("    <updated>{updated}T00:00:00Z</updated>\n"));
    // The author of each post without `authors`.
    xml.push_str(&format!(
        "    <author><name>{}</name></author>\n",
        escaped(title)
    ));
    for post in &feed.posts {
        let (url, date) = (&post.url, &post.date);
        xml.push_str("    <entry>\n");
        xml.push_str(&format!(
            "        <title>{}</title>\n",
            escaped(&post.title)
        ));
        xml.push_str(&format!(
            "        <link rel=\"alternate\" {html} href=\"{base_url}{url}\"/>\n"
        ));
        xml.push_str(&format!("        <id>{id_base}{url}</id>\n"));
        xml.push_str(&format!(
            "        <published>{date}T00:00:00Z</published>\n"
        ));
        xml.push_str(&format!("        <updated>{date}T00:00:00Z</updated>\n"));
        for author in &post.authors {
            xml.push_str(&format!(
                "        <author><name>{}</name></author>\n",
                escaped(author)
            ));
        }
        let content = escaped(&absolute(&post.body, base_url, url));
        xml.push_str(&format!(
            "        <content type=\"html\">{content}</content>\n"
        ));
        xml.push_str("    </entry>\n");
    }
    xml.push_str("</feed>\n");
    xml
}

/// `text` with `&`, `<`, `>`, `"` and `'` escaped, for XML.
fn escaped(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    edu_template::escape(&mut escaped, text);
    escaped
}

/// `html`, the text of the post at `url`, with each link to a page or file of the site, or to an
/// id of the post, made absolute at `base_url`, since a feed reader shows it away from the site.
fn absolute(html: &str, base_url: &BaseUrl, url: &str) -> String {
    let (top, here) = (base_url.to_string(), format!("{base_url}{url}"));
    links::prefix(html, |link| match Link::of(link) {
        Link::Top => Some(top.as_str()),
        Link::Here => Some(here.as_str()),
        Link::Away | Link::Other => None,
    })
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

    #[test]
    fn the_posts_are_the_live_pages_of_each_section_sorted_by_date_newest_first() {
        let pages = pages(&[
            ("_index.md", "---\ntitle: Home\n---\n"),
            ("dated.md", "---\ntitle: Dated\ndate: 2030-01-01\n---\n"),
            ("news/_index.md", "---\ntitle: News\nsort_by: date\n---\n"),
            ("news/a.md", "---\ntitle: A\ndate: 2022-07-11\n---\n"),
            ("news/new.md", "---\ntitle: New\ndate: 2026-04-26\n---\n"),
            ("news/z.md", "---\ntitle: Z\ndate: 2022-07-11\n---\n"),
            (
                "events/_index.md",
                "---\ntitle: Events\nsort_by: date\n---\n",
            ),
            (
                "events/talk.md",
                "---\ntitle: Talk\ndate: 2025-06-01\n---\n",
            ),
            (
                "old/_index.md",
                "---\nsort_by: date\nredirect_to: https://a.org/\n---\n",
            ),
            ("old/post.md", "---\ntitle: Old\ndate: 2029-01-01\n---\n"),
        ]);
        let is_live = |path: &Path| path != Path::new("content/news/new.md");
        let posts = posts(&pages, is_live).unwrap();
        let listed: Vec<_> = posts.iter().map(|post| post.url.as_str()).collect();
        assert_eq!(listed, ["/events/talk/", "/news/z/", "/news/a/"]);
    }

    #[test]
    fn each_link_of_a_post_to_the_site_is_absolute_at_the_base_url() {
        let html = "<a href=\"/news/\">/news/</a><img\nsrc=\"/a.png\" alt=\"/a\" />\
                    <a href=\"#part\">";
        for (base, top) in [
            ("https://a.org/edu", "https://a.org/edu"),
            ("https://a.org/", "https://a.org"),
        ] {
            let expected = format!(
                "<a href=\"{top}/news/\">/news/</a><img\nsrc=\"{top}/a.png\" alt=\"/a\" />\
                 <a href=\"{top}/news/p/#part\">"
            );
            let base_url = BaseUrl::parse(base).unwrap();
            assert_eq!(absolute(html, &base_url, "/news/p/"), expected, "{base}");
        }
        let others = "<a href=\"https://a.org/#x\"><a href=\"mailto:a@a.org\"><a data-href=\"/a\">\
                      <code>&lt;a href=\"/a\"&gt;</code>";
        let base_url = BaseUrl::parse("https://a.org/edu").unwrap();
        assert_eq!(absolute(others, &base_url, "/news/p/"), others);
    }
}
