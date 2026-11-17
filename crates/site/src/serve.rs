//! `edu serve`: the site on a web server for a preview, built in memory, and built again on a
//! request once one of its files has changed. It answers as GitHub Pages does: a page at its URL,
//! `/news` sent on to `/news/`, and `404.html` with the status 404 for a path the site lacks.

use std::fs;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};
use std::thread;
use std::time::{Duration, SystemTime};

use crate::build::{self, Options, Output, Site};
use crate::{check, count, links, report, urls};

/// How long the server waits for the request of a connection, which a browser may open early.
const TIMEOUT: Duration = Duration::from_secs(5);

/// The most bytes of the line and headers of a request that the server reads.
const MAX_HEAD: usize = 8 * 1024;

/// The files that the site is built from, each with its size and the time it last changed.
type Sources = Vec<(PathBuf, u64, Option<SystemTime>)>;

/// The last build: the site, or the report of the errors that stopped it, and its sources then.
struct Built {
    sources: Sources,
    site: Result<Site, String>,
}

/// The method of a request that the server answers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Method {
    Get,
    Head,
}

/// The status of a response.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Status {
    Ok,
    Moved,
    BadRequest,
    NotFound,
    NotAllowed,
    Error,
}

/// A response: its status, the type of its body, where it sends a redirect, and the body.
#[derive(Debug)]
struct Response {
    status: Status,
    kind: &'static str,
    location: Option<String>,
    body: Vec<u8>,
}

/// Serves the site in `root`, built with `options`, on `listener`, with a thread for each
/// connection, until the process ends.
pub(crate) fn serve(listener: &TcpListener, root: &Path, options: &Options) {
    let built = Mutex::new(None);
    // The first build says at once whether the site has errors.
    refresh(&mut lock(&built), root, options);
    thread::scope(|scope| {
        for stream in listener.incoming().flatten() {
            let built = &built;
            scope.spawn(move || {
                // A connection that fails ends only itself.
                let _ = respond(stream, root, options, built);
            });
        }
    });
}

fn lock(built: &Mutex<Option<Built>>) -> std::sync::MutexGuard<'_, Option<Built>> {
    built.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Answers the request on `stream` from the site as its files are now.
fn respond(
    mut stream: TcpStream,
    root: &Path,
    options: &Options,
    built: &Mutex<Option<Built>>,
) -> io::Result<()> {
    stream.set_read_timeout(Some(TIMEOUT))?;
    let (method, response) = match request(&mut stream)? {
        Ok((method, target)) => {
            let mut built = lock(built);
            let response = match &refresh(&mut built, root, options).site {
                Ok(site) => answer(site, root, &target),
                Err(report) => Response::text(Status::Error, report),
            };
            (method, response)
        }
        Err(status) => (Method::Get, Response::text(status, status.line())),
    };
    response.write(&mut stream, method)
}

/// The last build, or a new one if a file of `root` changed since.
fn refresh<'a>(built: &'a mut Option<Built>, root: &Path, options: &Options) -> &'a Built {
    let sources = sources(root);
    match built.take() {
        Some(last) if last.sources == sources => built.insert(last),
        _ => {
            let site = build::plan(root, options).map_err(|error| report(&error));
            match &site {
                Ok(site) => {
                    let files = count(site.files.len() - site.pages, "file");
                    eprintln!("edu: built {} and {files}", count(site.pages, "page"));
                }
                Err(report) => eprint!("{report}"),
            }
            built.insert(Built { sources, site })
        }
    }
}

/// The files of `root` that the site is built from, as they are now.
fn sources(root: &Path) -> Sources {
    let mut paths = vec![PathBuf::from("site.yaml"), PathBuf::from(urls::FILE)];
    for dir in ["content", "layouts", "static"] {
        // A folder that cannot be read stops the build, which says why.
        let _ = check::walk(root, Path::new(dir), &mut paths);
    }
    let source = |path: PathBuf| {
        let metadata = fs::metadata(root.join(&path)).ok();
        let len = metadata.as_ref().map_or(0, fs::Metadata::len);
        let modified = metadata.and_then(|metadata| metadata.modified().ok());
        (path, len, modified)
    };
    paths.into_iter().map(source).collect()
}

/// The method and the target of the request on `stream`, or the status that refuses it.
fn request(stream: &mut impl Read) -> io::Result<Result<(Method, String), Status>> {
    let mut head = Vec::new();
    let mut buffer = [0; 1024];
    // The head ends at its first empty line.
    let end = loop {
        if let Some(end) = head.windows(4).position(|end| end == b"\r\n\r\n") {
            break end;
        }
        let read = stream.read(&mut buffer)?;
        if read == 0 || head.len() > MAX_HEAD {
            break head.len();
        }
        head.extend_from_slice(buffer.get(..read).unwrap_or_default());
    };
    if end > MAX_HEAD {
        return Ok(Err(Status::BadRequest));
    }
    let head = String::from_utf8_lossy(head.get(..end).unwrap_or_default());
    let mut words = head.lines().next().unwrap_or_default().split(' ');
    let words = (words.next(), words.next(), words.next(), words.next());
    let (Some(method), Some(target), Some(version), None) = words else {
        return Ok(Err(Status::BadRequest));
    };
    if !version.starts_with("HTTP/1.") || !target.starts_with('/') {
        return Ok(Err(Status::BadRequest));
    }
    let method = match method {
        "GET" => Method::Get,
        "HEAD" => Method::Head,
        _ => return Ok(Err(Status::NotAllowed)),
    };
    Ok(Ok((method, target.to_owned())))
}

/// The response to a request for `target`: a page for a path that ends with `/`, a file at its
/// path, a redirect to the page of a path without its `/`, or else the file of the path with
/// `.html` added, as `404.html` for `/404`.
fn answer(site: &Site, root: &Path, target: &str) -> Response {
    let raw = target.split(['?', '#']).next().unwrap_or_default();
    let Some(path) = raw.strip_prefix('/').and_then(links::decode) else {
        return not_found(site, root);
    };
    let has = |path: &str| site.files.contains_key(Path::new(path));
    let file = if path.is_empty() || path.ends_with('/') {
        format!("{path}index.html")
    } else if has(&path) {
        path
    } else if has(&format!("{path}/index.html")) {
        let mut moved = Response::text(Status::Moved, "");
        moved.location = Some(format!("{raw}/"));
        return moved;
    } else {
        format!("{path}.html")
    };
    match body(site, root, &file) {
        Some(body) => Response::new(Status::Ok, kind(&file), body),
        None => not_found(site, root),
    }
}

/// The response for a path that the site does not have: its `404.html`.
fn not_found(site: &Site, root: &Path) -> Response {
    let body = body(site, root, "404.html").unwrap_or_default();
    Response::new(Status::NotFound, "text/html; charset=utf-8", body)
}

/// The bytes of the file `path` of `site`, if it has one that can be read.
fn body(site: &Site, root: &Path, path: &str) -> Option<Vec<u8>> {
    match site.files.get(Path::new(path))? {
        Output::Page(_, text) | Output::Made(text) => Some(text.clone().into_bytes()),
        Output::Copy(source) => fs::read(root.join(source)).ok(),
    }
}

/// The media type of the file `path`, by its extension; with an unknown one the file is a download.
fn kind(path: &str) -> &'static str {
    let extension = Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str());
    match extension.unwrap_or_default().to_ascii_lowercase().as_str() {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "txt" => "text/plain; charset=utf-8",
        "xml" => "application/xml",
        "json" => "application/json",
        "pdf" => "application/pdf",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        _ => "application/octet-stream",
    }
}

impl Status {
    /// The status line's code and reason.
    fn line(self) -> &'static str {
        match self {
            Self::Ok => "200 OK",
            Self::Moved => "301 Moved Permanently",
            Self::BadRequest => "400 Bad Request",
            Self::NotFound => "404 Not Found",
            Self::NotAllowed => "405 Method Not Allowed",
            Self::Error => "500 Internal Server Error",
        }
    }
}

impl Response {
    /// A response of `status` with `body`, of the media type `kind`.
    fn new(status: Status, kind: &'static str, body: Vec<u8>) -> Self {
        let location = None;
        Self {
            status,
            kind,
            location,
            body,
        }
    }

    /// A response of `status` with `text` as its body.
    fn text(status: Status, text: &str) -> Self {
        Self::new(
            status,
            "text/plain; charset=utf-8",
            text.as_bytes().to_vec(),
        )
    }

    /// Writes the response to `stream`, without its body for `HEAD`, and closes the connection.
    fn write(&self, stream: &mut impl Write, method: Method) -> io::Result<()> {
        let mut head = format!(
            "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nCache-Control: no-cache\r\n\
             Connection: close\r\n",
            self.status.line(),
            self.kind,
            self.body.len()
        );
        if let Some(location) = &self.location {
            head.push_str(&format!("Location: {location}\r\n"));
        }
        if self.status == Status::NotAllowed {
            head.push_str("Allow: GET, HEAD\r\n");
        }
        head.push_str("\r\n");
        stream.write_all(head.as_bytes())?;
        match method {
            Method::Get => stream.write_all(&self.body)?,
            Method::Head => {}
        }
        stream.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build::Drafts;

    /// The options of the live site.
    const LIVE: Options = Options {
        drafts: Drafts::Exclude,
        base_url: None,
    };

    /// The site `valid` under `crates/site/tests/sites/`.
    fn valid() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/sites/valid")
    }

    #[test]
    fn a_request_is_a_get_or_a_head_of_a_path() {
        let parse = |text: &str| request(&mut text.as_bytes()).unwrap();
        let get = Ok((Method::Get, "/news/?a=1".to_owned()));
        assert_eq!(parse("GET /news/?a=1 HTTP/1.1\r\nHost: a\r\n\r\n"), get);
        assert_eq!(
            parse("HEAD / HTTP/1.0\r\n\r\n"),
            Ok((Method::Head, "/".into()))
        );
        assert_eq!(parse("POST / HTTP/1.1\r\n\r\n"), Err(Status::NotAllowed));
        let bad = [
            "",
            "GET\r\n\r\n",
            "GET / HTTP/2\r\n\r\n",
            "GET http://a/ HTTP/1.1\r\n\r\n",
        ];
        for text in bad.into_iter().chain(["GET / HTTP/1.1 x\r\n\r\n"]) {
            assert_eq!(parse(text), Err(Status::BadRequest), "{text:?}");
        }
        let long = format!("GET / HTTP/1.1\r\nCookie: {}\r\n\r\n", "a".repeat(MAX_HEAD));
        assert_eq!(parse(&long), Err(Status::BadRequest));
    }

    #[test]
    fn a_path_is_answered_as_github_pages_answers_it() {
        let site = build::plan(&valid(), &LIVE).unwrap();
        let answer = |target| answer(&site, &valid(), target);
        let home = String::from_utf8(answer("/").body).unwrap();
        assert!(home.starts_with("<title>Home - Test</title>"), "{home}");
        let moved = answer("/news/post?a=1");
        let location = Some("/news/post/".to_owned());
        assert_eq!((moved.status, moved.location), (Status::Moved, location));
        let not_found = answer("/nothing/");
        assert!(!not_found.body.is_empty());
        assert_eq!(not_found.body, answer("/404").body);
        let (html, css) = ("text/html; charset=utf-8", "text/css; charset=utf-8");
        let cases = [
            ("/", Status::Ok, html),
            ("/news/post/", Status::Ok, html),
            ("/style.css?v=2", Status::Ok, css),
            ("/images/logo.svg", Status::Ok, "image/svg+xml"),
            ("/news/sample.bib", Status::Ok, "application/octet-stream"),
            ("/404", Status::Ok, html),
            ("/nothing/", Status::NotFound, html),
            ("/../site.yaml", Status::NotFound, html),
            ("/%2", Status::NotFound, html),
        ];
        for (target, status, kind) in cases {
            let response = answer(target);
            assert_eq!((response.status, response.kind), (status, kind), "{target}");
        }
        assert_eq!(kind("images/A.PNG"), "image/png");
    }

    /// Sends `request` to the server at `address`, and gives its whole response.
    fn send(address: std::net::SocketAddr, request: &str) -> String {
        let mut stream = TcpStream::connect(address).unwrap();
        stream.write_all(request.as_bytes()).unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        response
    }

    #[test]
    fn the_server_builds_the_site_again_after_a_change_and_shows_its_errors() {
        let root = std::env::temp_dir().join(format!("edu-serve-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let layouts = ["layouts/layout.html", "layouts/home.html"];
        let contents = ["content/_index.md", "content/chat/_index.md"];
        for path in ["site.yaml"].iter().chain(&layouts).chain(&contents) {
            fs::create_dir_all(root.join(path).parent().unwrap()).unwrap();
            fs::copy(valid().join(path), root.join(path)).unwrap();
        }
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let served = root.clone();
        thread::spawn(move || serve(&listener, &served, &LIVE));
        let get = |path: &str| send(address, &format!("GET {path} HTTP/1.1\r\nHost: a\r\n\r\n"));
        let home = get("/");
        let html = "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n";
        assert!(home.starts_with(html), "{home}");
        assert!(home.contains("<h1>Home</h1>\n<p>Welcome.</p>\n"), "{home}");
        let (head, _) = home.split_once("\r\n\r\n").unwrap();
        let head = format!("{head}\r\n\r\n");
        assert_eq!(send(address, "HEAD / HTTP/1.1\r\n\r\n"), head);
        let (moved, missing) = (get("/chat"), get("/nothing/"));
        let bad = send(address, "nonsense\r\n\r\n");
        let post = send(address, "POST / HTTP/1.1\r\n\r\n");
        let lines = [
            (&moved, "HTTP/1.1 301 Moved Permanently\r\n"),
            (&missing, "HTTP/1.1 404 Not Found\r\n"),
            (&bad, "HTTP/1.1 400 Bad Request\r\n"),
            (&post, "HTTP/1.1 405 Method Not Allowed\r\n"),
        ];
        for (response, line) in lines {
            assert!(response.starts_with(line), "{response}");
        }
        assert!(moved.contains("\r\nLocation: /chat/\r\n"), "{moved}");
        assert!(post.contains("\r\nAllow: GET, HEAD\r\n"), "{post}");
        // A change to a page, to a layout or to the files of `static/` shows at the next request.
        let page = root.join("content/_index.md");
        fs::write(&page, "---\ntitle: Home\n---\nWelcome again.\n").unwrap();
        assert!(get("/").contains("<p>Welcome again.</p>\n"));
        fs::write(root.join(layouts[1]), "<h2>{{ page.title }}</h2>\n").unwrap();
        assert!(get("/").contains("<h2>Home</h2>\n"));
        fs::create_dir(root.join("static")).unwrap();
        fs::write(root.join("static/new.css"), "a {}\n").unwrap();
        assert!(get("/new.css").ends_with("\r\n\r\na {}\n"));
        fs::write(
            root.join("urls.yaml"),
            "redirects:\n  - from: /old/\n    to: https://example.org/\n",
        )
        .unwrap();
        assert!(get("/old/").contains("url=https://example.org/\">"));
        fs::write(&page, "---\ntitle: Home\n").unwrap();
        let error = get("/");
        assert!(
            error.starts_with("HTTP/1.1 500 Internal Server Error\r\n"),
            "{error}"
        );
        assert!(error.ends_with("edu: 1 error in the site\n"), "{error}");
        fs::write(&page, "---\ntitle: Home\n---\n").unwrap();
        assert!(get("/").starts_with("HTTP/1.1 200 OK\r\n"));
        fs::remove_dir_all(root).unwrap();
    }
}
