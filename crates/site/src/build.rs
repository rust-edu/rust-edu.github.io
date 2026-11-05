//! `edu build`: the site as files. Each page becomes `index.html` in the folder of its URL, the
//! files of `static/` are copied to the top, and the other files of `content/` beside the pages of
//! their folder; `404.html`, `robots.txt` and `sitemap.xml` go at the top. A preview has the
//! drafts too, and works at another address.

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::Error;
use crate::check::{self, Rendered};
use crate::links;

/// The file that marks a folder as made by `edu build`, which may then delete it.
const MARKER: &str = ".edu-output";

/// The files that the build makes at the top of the site, besides the pages.
pub(crate) const MADE: [&str; 3] = ["404.html", "robots.txt", "sitemap.xml"];

/// The page for an address without a file, as Zola writes it.
const NOT_FOUND: &str = "<!doctype html>\n<title>404 Not Found</title>\n<h1>404 Not Found</h1>\n";

/// How `edu build` builds: with or without drafts, and for the address of `site.yaml` or another.
#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    pub drafts: Drafts,
    pub base_url: Option<BaseUrl>,
}

/// Whether the build has the pages with `draft: true` and everything in the folder of an
/// `_index.md` with it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Drafts {
    /// Left out, as on the live site.
    Exclude,
    /// Like the other pages, as `edu check` checks them and `edu build --drafts` writes them.
    Include,
}

/// The address of the built site, such as `https://rust-edu.org`, without a `/` at its end.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BaseUrl(String);

impl BaseUrl {
    /// `text` as an address: `https://` or `http://`, then a host and maybe a path, as names of
    /// ASCII letters, digits, `-`, `.`, `_`, `~` and `:` between single `/`, and maybe a `/` at
    /// the end, which the address leaves out.
    pub fn parse(text: &str) -> Option<Self> {
        let text = text.strip_suffix('/').unwrap_or(text);
        let rest = text
            .strip_prefix("https://")
            .or_else(|| text.strip_prefix("http://"))?;
        let is_name = |name: &str| {
            let is_allowed = |c: char| c.is_ascii_alphanumeric() || "-._~:".contains(c);
            !name.is_empty() && name.chars().all(is_allowed)
        };
        rest.split('/').all(is_name).then(|| Self(text.to_owned()))
    }

    /// The path after the host, such as `/rust-edu` in `https://example.github.io/rust-edu`, or
    /// nothing.
    fn path(&self) -> &str {
        let rest = self.0.split_once("://").map_or("", |(_, rest)| rest);
        rest.find('/').and_then(|at| rest.get(at..)).unwrap_or("")
    }
}

impl fmt::Display for BaseUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// How many pages `edu build` wrote, and how many other files.
#[derive(Debug)]
pub(crate) struct Built {
    pub(crate) pages: usize,
    pub(crate) files: usize,
}

/// What a file of the output holds: the HTML of a page, or a copy of a file of the site.
enum Output<'a> {
    Page(&'a str),
    Copy(&'a Path),
}

/// Builds the site in `root` with `options` into `out`, relative to `root`. The files go to `out`
/// with `.tmp` added first, which then takes the place of `out`; a folder in the way is deleted
/// only if an earlier build made it.
pub(crate) fn site(root: &Path, out: &Path, options: &Options) -> Result<Built, Error> {
    let mut checked = check::pages(root, options)?;
    let base_url = &checked.base_url;
    for page in &mut checked.rendered {
        page.html = rebase(&page.html, base_url.path());
    }
    // The texts of the files of `MADE`, in its order.
    let texts = [
        NOT_FOUND.to_owned(),
        robots(base_url),
        sitemap(base_url, &checked.rendered),
    ];
    let made: Vec<(&str, String)> = MADE.into_iter().zip(texts).collect();
    // Each file of the output, with the file of the site that it comes from.
    let mut outputs = BTreeMap::new();
    for page in &checked.rendered {
        let path = Path::new(page.url.trim_start_matches('/')).join("index.html");
        add(&mut outputs, path, &page.path, Output::Page(&page.html))?;
    }
    for (dir, files) in [("content", &checked.files), ("static", &checked.statics)] {
        for file in files {
            let path = file.strip_prefix(dir).unwrap_or(file).to_owned();
            add(&mut outputs, path, file, Output::Copy(file))?;
        }
    }
    for (name, _) in &made {
        if let Some((file, _)) = outputs.get(Path::new(name)) {
            let (path, file) = (PathBuf::from(name), file.to_path_buf());
            return Err(Error::Reserved { path, file });
        }
    }
    let tmp = out.with_extension("tmp");
    clear(root, &tmp)?;
    let written = write(root, &tmp, &outputs, &made);
    if written.is_err() {
        // A folder with the marker is deleted by the next build anyway.
        let _ = fs::remove_dir_all(root.join(&tmp));
    }
    written?;
    clear(root, out)?;
    fs::rename(root.join(&tmp), root.join(out)).map_err(output_error(out))?;
    let pages = checked.rendered.len();
    let files = checked.files.len() + checked.statics.len() + made.len();
    Ok(Built { pages, files })
}

/// `robots.txt`, as Zola writes it: every page may be crawled, and the sitemap is at `base_url`.
fn robots(base_url: &BaseUrl) -> String {
    format!("User-agent: *\nDisallow:\nAllow: /\nSitemap: {base_url}/sitemap.xml\n")
}

/// `sitemap.xml`, as Zola writes it: the address of each page at `base_url`, by URL, with the
/// `date` of a page that has one.
fn sitemap(base_url: &BaseUrl, pages: &[Rendered]) -> String {
    let mut pages: Vec<&Rendered> = pages.iter().collect();
    pages.sort_by(|a, b| a.url.cmp(&b.url));
    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
    );
    for page in pages {
        // Neither the base URL nor a page's URL has a character that XML escapes.
        let url = &page.url;
        xml.push_str(&format!("    <url>\n        <loc>{base_url}{url}</loc>\n"));
        if let Some(date) = &page.date {
            xml.push_str(&format!("        <lastmod>{date}</lastmod>\n"));
        }
        xml.push_str("    </url>\n");
    }
    xml.push_str("</urlset>\n");
    xml
}

/// `html` with `path`, the path of the base URL, before each link that starts with one `/`, so
/// that the links between the pages and files of the site work under that path too.
fn rebase(html: &str, path: &str) -> String {
    let mut rebased = String::with_capacity(html.len());
    let mut last = 0;
    for (at, link) in links::links(html) {
        if link.starts_with('/') && !link.starts_with("//") {
            rebased.push_str(html.get(last..at).unwrap_or_default());
            rebased.push_str(path);
            last = at;
        }
    }
    rebased.push_str(html.get(last..).unwrap_or_default());
    rebased
}

/// Adds the file `path` of the output, from `source`, unless another file of the site is `path`.
fn add<'a>(
    outputs: &mut BTreeMap<PathBuf, (&'a Path, Output<'a>)>,
    path: PathBuf,
    source: &'a Path,
    output: Output<'a>,
) -> Result<(), Error> {
    if let Some((first, _)) = outputs.get(&path) {
        let (first, second) = (first.to_path_buf(), source.to_owned());
        return Err(Error::Clash {
            path,
            first,
            second,
        });
    }
    outputs.insert(path, (source, output));
    Ok(())
}

/// Deletes the folder `dir`, relative to `root`, if it is there and an earlier build made it.
fn clear(root: &Path, dir: &Path) -> Result<(), Error> {
    let full = root.join(dir);
    if !full.try_exists().map_err(output_error(dir))? {
        return Ok(());
    }
    if !full.join(MARKER).is_file() {
        return Err(Error::Foreign(dir.to_owned()));
    }
    fs::remove_dir_all(full).map_err(output_error(dir))
}

/// Writes the marker, every file of `outputs` and the files that the build `made` into the folder
/// `dir`, relative to `root`.
fn write(
    root: &Path,
    dir: &Path,
    outputs: &BTreeMap<PathBuf, (&Path, Output<'_>)>,
    made: &[(&str, String)],
) -> Result<(), Error> {
    let marker =
        "This folder is the output of `edu build`, which deletes it when it builds again.\n";
    fs::create_dir_all(root.join(dir)).map_err(output_error(dir))?;
    fs::write(root.join(dir).join(MARKER), marker).map_err(output_error(dir))?;
    for (path, (_, output)) in outputs {
        let to = dir.join(path);
        let full = root.join(&to);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).map_err(output_error(&to))?;
        }
        let written = match output {
            Output::Page(html) => fs::write(&full, html),
            Output::Copy(from) => fs::copy(root.join(from), &full).map(|_| ()),
        };
        written.map_err(output_error(&to))?;
    }
    for (name, text) in made {
        let to = dir.join(name);
        fs::write(root.join(&to), text).map_err(output_error(&to))?;
    }
    Ok(())
}

fn output_error(path: &Path) -> impl Fn(io::Error) -> Error + '_ {
    |source| Error::Output {
        path: path.to_owned(),
        source,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The options of the live site: no drafts, at the address of `site.yaml`.
    const LIVE: Options = Options {
        drafts: Drafts::Exclude,
        base_url: None,
    };

    /// A site under `crates/site/tests/sites/`.
    fn test_site(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/sites")
            .join(name)
    }

    /// A folder for the output of one test, not there yet, with no `.tmp` folder beside it.
    fn out(test: &str) -> PathBuf {
        let name = format!("edu-build-{}-{test}", std::process::id());
        let out = std::env::temp_dir().join(name);
        let _ = fs::remove_dir_all(&out);
        let _ = fs::remove_dir_all(out.with_extension("tmp"));
        out
    }

    /// The files under `dir`, as paths relative to it.
    fn files(dir: &Path) -> Vec<String> {
        let mut files = Vec::new();
        check::walk(dir, Path::new(""), &mut files).unwrap();
        files
            .iter()
            .map(|path| path.display().to_string())
            .collect()
    }

    #[test]
    fn each_page_is_index_html_at_its_url_beside_the_files_of_static_and_content() {
        let out = out("valid");
        let built = site(&test_site("valid"), &out, &LIVE).unwrap();
        assert_eq!((built.pages, built.files), (4, 7));
        let expected = [
            ".edu-output",
            "2027/index.html",
            "404.html",
            "chat/index.html",
            "images/logo.svg",
            "index.html",
            "news/notes.txt",
            "news/post/index.html",
            "news/sample.bib",
            "robots.txt",
            "sitemap.xml",
            "style.css",
        ];
        assert_eq!(files(&out), expected);
        let home = fs::read_to_string(out.join("index.html")).unwrap();
        assert!(home.starts_with("<title>Home - Test</title>\n"), "{home}");
        let style = test_site("valid").join("static/style.css");
        assert_eq!(
            fs::read(out.join("style.css")).unwrap(),
            fs::read(style).unwrap()
        );
        assert!(!out.with_extension("tmp").exists());
        fs::remove_dir_all(out).unwrap();
    }

    #[test]
    fn the_build_makes_404_html_robots_txt_and_the_sitemap_as_zola_does() {
        let out = out("made");
        site(&test_site("valid"), &out, &LIVE).unwrap();
        let read = |path| fs::read_to_string(out.join(path)).unwrap();
        let not_found = "<!doctype html>\n<title>404 Not Found</title>\n<h1>404 Not Found</h1>\n";
        assert_eq!(read("404.html"), not_found);
        let robots =
            "User-agent: *\nDisallow:\nAllow: /\nSitemap: https://example.org/sitemap.xml\n";
        assert_eq!(read("robots.txt"), robots);
        let mut sitemap = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        sitemap.push_str("<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n");
        let urls = [
            ("/", ""),
            ("/2027/", ""),
            ("/chat/", ""),
            ("/news/post/", "2027-03-01"),
        ];
        for (url, date) in urls {
            let loc = format!("        <loc>https://example.org{url}</loc>\n");
            let lastmod = format!("        <lastmod>{date}</lastmod>\n");
            let lastmod = if date.is_empty() { "" } else { &lastmod };
            sitemap.push_str(&format!("    <url>\n{loc}{lastmod}    </url>\n"));
        }
        sitemap.push_str("</urlset>\n");
        assert_eq!(read("sitemap.xml"), sitemap);
        let chat = read("chat/index.html");
        assert!(chat.contains("url=https://example.org/chat/\">"), "{chat}");
        fs::remove_dir_all(out).unwrap();
    }

    #[test]
    fn a_preview_has_the_drafts_and_its_links_and_sitemap_are_for_its_address() {
        let out = out("preview");
        let options = Options {
            drafts: Drafts::Include,
            base_url: BaseUrl::parse("https://example.github.io/edu/"),
        };
        let built = site(&test_site("valid"), &out, &options).unwrap();
        assert_eq!((built.pages, built.files), (7, 8));
        let files = files(&out);
        let drafts = ["news/draft/index.html", "wip/index.html", "wip/notes.txt"];
        let is_built = |path: &&str| files.iter().any(|file| file == path);
        assert!(drafts.iter().all(is_built), "{files:?}");
        let read = |path| fs::read_to_string(out.join(path)).unwrap();
        let home = read("index.html");
        assert!(
            home.contains("<li><a href=\"/edu/\">Home</a></li>"),
            "{home}"
        );
        let robots = read("robots.txt");
        assert!(
            robots.ends_with(" https://example.github.io/edu/sitemap.xml\n"),
            "{robots}"
        );
        let sitemap = read("sitemap.xml");
        let loc = "<loc>https://example.github.io/edu/wip/page/</loc>";
        assert!(sitemap.contains(loc), "{sitemap}");
        fs::remove_dir_all(out).unwrap();
    }

    #[test]
    fn a_base_url_is_an_address_without_a_slash_at_its_end() {
        let cases = [
            ("https://rust-edu.org/", "https://rust-edu.org"),
            ("http://127.0.0.1:1111", "http://127.0.0.1:1111"),
            ("https://a.github.io/b/c_d~e", "https://a.github.io/b/c_d~e"),
        ];
        for (text, url) in cases {
            let base_url = BaseUrl::parse(text).map(|url| url.to_string());
            assert_eq!(base_url.as_deref(), Some(url));
        }
        let not_urls = "rust-edu.org ftp://a.org https:// https:///a https://a.org// https://a.org//b \
                        https://a.org/?a https://a.org/#a https://a.org/\" https://é.org";
        for text in not_urls.split(' ').chain(["", "https://a.org/a b"]) {
            assert_eq!(BaseUrl::parse(text), None, "{text}");
        }
        let paths = [
            ("https://a.org/", ""),
            ("https://a.github.io/b/c_d~e/", "/b/c_d~e"),
        ];
        for (text, path) in paths {
            assert_eq!(BaseUrl::parse(text).unwrap().path(), path, "{text}");
        }
    }

    #[test]
    fn a_link_from_the_top_of_the_site_goes_under_the_path_of_the_base_url() {
        let cases = [
            (
                "<a href=\"/news/\">/news/</a>",
                "<a href=\"/edu/news/\">/news/</a>",
            ),
            (
                "<img\nsrc=\"/a.png\" alt=\"/a\">",
                "<img\nsrc=\"/edu/a.png\" alt=\"/a\">",
            ),
        ];
        for (html, rebased) in cases {
            assert_eq!(rebase(html, "/edu"), rebased);
        }
        let others = "<a href=\"//a.org/\"><a data-href=\"/a\"><a href=\"https://a.org/\">\
                      <code>&lt;a href=\"/a\"&gt;</code>";
        assert_eq!(rebase(others, "/edu"), others);
    }

    #[test]
    fn a_build_replaces_the_one_before_but_no_folder_that_it_did_not_make() {
        let out = out("again");
        site(&test_site("valid"), &out, &LIVE).unwrap();
        fs::write(out.join("stray.html"), "").unwrap();
        site(&test_site("valid"), &out, &LIVE).unwrap();
        assert!(!out.join("stray.html").exists());
        fs::remove_file(out.join(MARKER)).unwrap();
        let error = site(&test_site("valid"), &out, &LIVE).unwrap_err();
        let error = error.to_string();
        let expected = "is in the way: no earlier `edu build` made it, so move it away";
        assert_eq!(error, format!("{} {expected}", out.display()));
        assert!(out.join("index.html").exists());
        fs::remove_dir_all(out).unwrap();
    }

    #[test]
    fn a_folder_that_cannot_be_written_is_an_error() {
        // A file where the output's folder would go.
        let file = out("file");
        fs::write(&file, "").unwrap();
        let result = site(&test_site("valid"), &file.join("public"), &LIVE);
        assert!(matches!(result, Err(Error::Output { .. })), "{result:?}");
        let error = result.unwrap_err().to_string();
        assert!(error.starts_with("cannot write "), "{error}");
        fs::remove_file(file).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn a_file_that_cannot_be_copied_is_an_error_and_leaves_no_output() {
        let site = out("site");
        let files = [
            ("site.yaml", "base_url: https://example.org\n"),
            ("layouts/layout.html", "{{ content | safe }}"),
            ("layouts/home.html", "{{ page.title }}"),
            ("content/_index.md", "---\ntitle: Home\n---\n"),
        ];
        for (path, text) in files {
            let path = site.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        }
        fs::create_dir(site.join("static")).unwrap();
        std::os::unix::fs::symlink("nowhere", site.join("static/gone.css")).unwrap();
        let out = out("gone");
        let result = super::site(&site, &out, &LIVE);
        let is_gone = |path: &Path| path.ends_with("gone.css");
        assert!(
            matches!(&result, Err(Error::Output { path, .. }) if is_gone(path)),
            "{result:?}"
        );
        assert!(!out.exists() && !out.with_extension("tmp").exists());
        fs::remove_dir_all(site).unwrap();
    }

    #[test]
    fn a_site_with_an_error_or_two_files_for_one_url_writes_nothing() {
        let out = out("broken");
        let result = site(&test_site("broken"), &out, &LIVE);
        assert!(matches!(result, Err(Error::Content(_))), "{result:?}");
        let error = site(&test_site("clash"), &out, &LIVE).unwrap_err();
        let expected = "content/_index.md and static/index.html would both be /index.html";
        assert_eq!(error.to_string(), expected);
        let error = site(&test_site("reserved"), &out, &LIVE).unwrap_err();
        let expected = "static/sitemap.xml would be /sitemap.xml, which `edu build` writes itself";
        assert_eq!(error.to_string(), expected);
        assert!(!out.exists() && !out.with_extension("tmp").exists());
    }
}
