//! `edu build`: the site as files. Each page that does not redirect becomes `index.html` in the
//! folder of its URL, the files of `static/` are copied to the top, and the other files of
//! `content/` beside the pages of their folder.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::Error;
use crate::check;

/// The file that marks a folder as made by `edu build`, which may then delete it.
const MARKER: &str = ".edu-output";

/// How many pages `edu build` wrote, and how many files it copied.
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

/// Builds the site in `root` into `out`, relative to `root`. The files go to `out` with `.tmp`
/// added first, which then takes the place of `out`; a folder in the way is deleted only if an
/// earlier build made it.
pub(crate) fn site(root: &Path, out: &Path) -> Result<Built, Error> {
    let checked = check::pages(root)?;
    let mut statics = Vec::new();
    check::walk(root, Path::new("static"), &mut statics)?;
    // Each file of the output, with the file of the site that it comes from.
    let mut outputs = BTreeMap::new();
    for page in &checked.rendered {
        let path = Path::new(page.url.trim_start_matches('/')).join("index.html");
        add(&mut outputs, path, &page.path, Output::Page(&page.html))?;
    }
    for (dir, files) in [("content", &checked.files), ("static", &statics)] {
        for file in files {
            let path = file.strip_prefix(dir).unwrap_or(file).to_owned();
            add(&mut outputs, path, file, Output::Copy(file))?;
        }
    }
    let tmp = out.with_extension("tmp");
    clear(root, &tmp)?;
    let written = write(root, &tmp, &outputs);
    if written.is_err() {
        // A folder with the marker is deleted by the next build anyway.
        let _ = fs::remove_dir_all(root.join(&tmp));
    }
    written?;
    clear(root, out)?;
    fs::rename(root.join(&tmp), root.join(out)).map_err(output_error(out))?;
    let pages = checked.rendered.len();
    let files = checked.files.len() + statics.len();
    Ok(Built { pages, files })
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

/// Writes the marker and every file of `outputs` into the folder `dir`, relative to `root`.
fn write(
    root: &Path,
    dir: &Path,
    outputs: &BTreeMap<PathBuf, (&Path, Output<'_>)>,
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
        let built = site(&test_site("valid"), &out).unwrap();
        assert_eq!((built.pages, built.files), (2, 4));
        let expected = [
            ".edu-output",
            "images/logo.svg",
            "index.html",
            "news/notes.txt",
            "news/post/index.html",
            "news/sample.bib",
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
    fn a_build_replaces_the_one_before_but_no_folder_that_it_did_not_make() {
        let out = out("again");
        site(&test_site("valid"), &out).unwrap();
        fs::write(out.join("stray.html"), "").unwrap();
        site(&test_site("valid"), &out).unwrap();
        assert!(!out.join("stray.html").exists());
        fs::remove_file(out.join(MARKER)).unwrap();
        let error = site(&test_site("valid"), &out).unwrap_err().to_string();
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
        let result = site(&test_site("valid"), &file.join("public"));
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
            ("site.yaml", "title: Test\n"),
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
        let result = super::site(&site, &out);
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
        let result = site(&test_site("broken"), &out);
        assert!(matches!(result, Err(Error::Content(_))), "{result:?}");
        let error = site(&test_site("clash"), &out).unwrap_err().to_string();
        let expected = "content/_index.md and static/index.html would both be /index.html";
        assert_eq!(error, expected);
        assert!(!out.exists() && !out.with_extension("tmp").exists());
    }
}
