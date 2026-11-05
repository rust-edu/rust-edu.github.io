//! Runs the `edu` binary as a user or CI would, for its exit statuses and output streams.

use std::io;
use std::path::Path;
use std::process::{Command, Output};

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

#[test]
fn check_names_what_it_cannot_read_and_why() {
    // `tests/sites` holds sites, but no `content/` of its own.
    let output = edu_check(".").unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.starts_with("edu: cannot read content: "), "{stderr}");
}
