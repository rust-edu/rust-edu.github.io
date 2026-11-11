//! Runs the `edu` binary as a user or CI would, for its exit statuses and output streams.

use std::fs;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::process::{Command, Output, Stdio};

fn edu(args: &[&str]) -> io::Result<Output> {
    Command::new(env!("CARGO_BIN_EXE_edu")).args(args).output()
}

/// Runs `edu check` in `crates/site/tests/sites/<site>`.
fn edu_check(site: &str) -> io::Result<Output> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/sites")
        .join(site);
    Command::new(env!("CARGO_BIN_EXE_edu"))
        .arg("check")
        .current_dir(dir)
        .output()
}

#[test]
fn help_prints_the_usage_on_stdout_and_succeeds() {
    let output = edu(&["help"]).unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.starts_with("Usage: edu <command>\n"), "{stdout}");
    assert!(output.stderr.is_empty());
}

#[test]
fn a_usage_error_goes_to_stderr_with_the_usage_and_exits_with_2() {
    let output = edu(&["biuld"]).unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.starts_with("edu: unknown command `biuld`\n\nUsage: edu <command>\n"),
        "{stderr}"
    );
}

#[test]
fn check_lists_each_bad_header_or_text_on_stderr_and_exits_with_1() {
    let output = edu_check("broken").unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let expected = concat!(
        "content/a/no-header.md:1:1: error: expected a line `---` that opens the header\n",
        "content/b.md:2:8: error: a value with `: ` in it or `:` at its end needs quotes\n",
        "content/d.md:5:11: error: HTML is not supported; write `\\<` for a `<`, or put code in \
         backticks\n",
        "site.yaml:1:8: error: a value with `: ` in it or `:` at its end needs quotes\n",
        "layouts/layout.html:1:11: error: unknown filter `upper`; the only filter is `safe`\n",
        "edu: 5 errors in the site\n",
    );
    assert_eq!(String::from_utf8(output.stderr).unwrap(), expected);
}

/// Copies the folder `from` to `to`, with everything in it.
fn copy(from: &Path, to: &Path) -> io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let to = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy(&entry.path(), &to)?;
        } else {
            fs::copy(entry.path(), to)?;
        }
    }
    Ok(())
}

#[test]
fn build_writes_the_site_into_public_and_says_how_much() {
    let sites = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/sites");
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("valid");
    let _ = fs::remove_dir_all(&dir);
    copy(&sites.join("valid"), &dir).unwrap();
    let build = |args: &[&str]| {
        let output = Command::new(env!("CARGO_BIN_EXE_edu"))
            .arg("build")
            .args(args)
            .current_dir(&dir)
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        String::from_utf8(output.stdout).unwrap()
    };

    assert_eq!(build(&[]), "wrote 5 pages and 10 files to public/\n");
    assert!(dir.join("public/news/post/index.html").is_file());
    let preview = build(&["--drafts", "--base-url", "https://example.github.io/edu"]);
    assert_eq!(preview, "wrote 8 pages and 11 files to public/\n");
    let robots = fs::read_to_string(dir.join("public/robots.txt")).unwrap();
    assert!(robots.ends_with(" https://example.github.io/edu/sitemap.xml\n"));
}

#[test]
fn check_names_what_it_cannot_read_and_why() {
    // `tests/sites` holds sites, but no `content/` of its own.
    let output = edu_check(".").unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.starts_with("edu: cannot read content: "), "{stderr}");
}

#[test]
fn serve_says_where_it_listens_and_serves_the_site_there() {
    let site = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/sites/valid");
    let mut server = Command::new(env!("CARGO_BIN_EXE_edu"))
        .args(["serve", "--bind", "127.0.0.1:0"])
        .current_dir(&site)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut line = String::new();
    BufReader::new(server.stdout.take().unwrap())
        .read_line(&mut line)
        .unwrap();
    let address = line
        .strip_prefix("serving http://")
        .and_then(|rest| rest.strip_suffix("/ until Ctrl-C\n"));
    let mut stream = TcpStream::connect(address.unwrap()).unwrap();
    stream
        .write_all(b"GET /news/post/ HTTP/1.1\r\n\r\n")
        .unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    server.kill().unwrap();
    server.wait().unwrap();

    assert!(response.starts_with("HTTP/1.1 200 OK\r\n"), "{response}");
    assert!(response.contains("<h1>Rust Week: 2027</h1>"), "{response}");
    assert!(!site.join("public").exists());
}
