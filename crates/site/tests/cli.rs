//! Runs the `edu` binary as a user or CI would, for its exit statuses and output streams.

use std::io;
use std::process::{Command, Output};

fn edu(args: &[&str]) -> io::Result<Output> {
    Command::new(env!("CARGO_BIN_EXE_edu")).args(args).output()
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
