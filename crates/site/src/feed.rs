//! The posts of the site's feed: the pages that each section with `sort_by: date` lists, newest
//! first, each with its whole text, for feed readers.

use std::path::{Path, PathBuf};

use crate::check::ContentError;
use crate::page::{Order, Page};
use crate::render;

/// A page that a section with `sort_by: date` lists: its file and URL, and its text as HTML.
#[derive(Debug)]
pub(crate) struct Post {
    pub(crate) path: PathBuf,
    pub(crate) url: String,
    pub(crate) body: String,
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
        posts.push(Post {
            path: page.path.clone(),
            url: page.url.clone(),
            body: page.html.clone(),
        });
    }
    Ok(posts)
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
}
