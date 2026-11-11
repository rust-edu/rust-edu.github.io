//! `llms.txt`: the site in brief for language models, in the Markdown that <https://llmstxt.org/>
//! proposes. Its `#` heading is the site's `title`, a `>` quote its `description`, and then come a
//! link to each page, a link to each post with its day and authors, and, as optional reading, the
//! feeds, which hold the whole text of every post.

use crate::build::BaseUrl;
use crate::check::{Rendered, RenderedKind};
use crate::feed::{Feed, Format};

/// `llms.txt` of the site with the `title` and `description` of `site.yaml`, the rendered `pages`
/// and the `feed`, if it has one, with its links at `base_url`. Without a `title`, the heading is
/// `base_url`. The pages are sorted by URL, as in the sitemap, and the posts, newest first, are
/// not among them; the redirects are left out, and so are the drafts, so that a preview has the
/// `llms.txt` of the live site, at its own address.
pub(crate) fn write(
    title: Option<&str>,
    description: Option<&str>,
    pages: &[Rendered],
    feed: Option<&Feed>,
    base_url: &BaseUrl,
) -> String {
    let heading = title.map_or_else(|| base_url.to_string(), escaped);
    let mut text = format!("# {heading}\n");
    if let Some(description) = description {
        text.push_str(&format!("\n> {}\n", escaped(description)));
    }
    let posts = feed.map_or(&[][..], Feed::posts);
    let mut listed: Vec<&Rendered> = pages
        .iter()
        .filter(|page| match page.kind {
            RenderedKind::Page => page.live && !posts.iter().any(|post| post.url == page.url),
            RenderedKind::Redirect => false,
        })
        .collect();
    listed.sort_by(|a, b| a.url.cmp(&b.url));
    if !listed.is_empty() {
        text.push_str("\n## Pages\n\n");
        for page in listed {
            // Every page but a redirect has a title, as `edu check` checks.
            let name = page.title.as_deref().unwrap_or(&page.url);
            text.push_str(&format!("- {}\n", link(name, base_url, &page.url)));
        }
    }
    if !posts.is_empty() {
        text.push_str("\n## Posts\n\n");
        for post in posts {
            let link = link(&post.title, base_url, &post.url);
            let mut note = post.date.text.clone();
            if !post.authors.is_empty() {
                note.push_str(&format!(", by {}", escaped(&post.authors.join(" and "))));
            }
            text.push_str(&format!("- {link}: {note}\n"));
        }
        text.push_str("\n## Optional\n\n");
        for format in Format::ALL {
            let name = format!("{} feed", format.name());
            let link = link(&name, base_url, &format!("/{}", format.file()));
            text.push_str(&format!(
                "- {link}: every post with its whole text, newest first\n"
            ));
        }
    }
    text
}

/// A Markdown link with the text `name` to `url`, a path of the site, at `base_url`. Neither the
/// base URL nor a path of the site has a space, a `(` or a `)`, which the link would need to
/// escape.
fn link(name: &str, base_url: &BaseUrl, url: &str) -> String {
    format!("[{}]({base_url}{url})", escaped(name))
}

/// `text` with a `\` before each character that Markdown would read as syntax in a heading, a
/// quote or a link's text: `\`, `` ` ``, `*`, `_`, `[`, `]` and `<`.
fn escaped(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for c in text.chars() {
        if "\\`*_[]<".contains(c) {
            escaped.push('\\');
        }
        escaped.push(c);
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_syntax_in_a_title_is_escaped_and_other_text_is_not() {
        let title = r"A [draft] of *Rust_Edu* <2027> \ `code` & “quotes”: done";
        let expected = r"A \[draft\] of \*Rust\_Edu\* \<2027> \\ \`code\` & “quotes”: done";
        assert_eq!(escaped(title), expected);
    }

    #[test]
    fn a_link_is_to_the_page_at_the_base_url() {
        let base_url = BaseUrl::parse("https://example.github.io/edu/").unwrap();
        let link = link("[Notes]", &base_url, "/news/notes/");
        assert_eq!(
            link,
            r"[\[Notes\]](https://example.github.io/edu/news/notes/)"
        );
    }
}
