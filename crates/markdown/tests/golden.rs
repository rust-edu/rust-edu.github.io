//! The site's pages, byte for byte: `golden/<page>.md` is the text of a page under `content/` as
//! of November 2026, after its header, and `golden/<page>.html` the HTML that Zola 0.22.1 made of
//! it on the live site, with the `#` links that Zola makes absolute
//! (`https://rust-edu.org/workshop/cfp-2022/#schedule`) as written in the page (`#schedule`).
//! `UPDATE_GOLDEN=1 cargo test -p edu-markdown --test golden` writes the HTML files anew.

use std::env;
use std::fs;
use std::path::Path;

#[test]
fn each_page_renders_as_zola_rendered_it() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden");
    let mut pages: Vec<_> = fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "md"))
        .collect();
    pages.sort();
    // The 11 pages of the site, without the 2 that are only a header.
    assert_eq!(pages.len(), 9, "{pages:?}");
    let update = env::var_os("UPDATE_GOLDEN").is_some();
    let mut different = Vec::new();
    for page in &pages {
        let html = edu_markdown::to_html(&fs::read_to_string(page).unwrap()).unwrap();
        let golden = page.with_extension("html");
        if update {
            fs::write(&golden, &html).unwrap();
        } else if fs::read_to_string(&golden).unwrap() != html {
            different.push(golden);
        }
    }
    assert!(different.is_empty(), "different HTML: {different:?}");
}
