//! The blocks of the subset through the public API: the HTML of each, and where each error points.

use edu_markdown::ErrorKind::{
    ControlCharacter, IndentedCode, NoHeadingId, NonAsciiInHeading, Rule, UnderlinedHeading,
};
use edu_markdown::{ErrorKind, to_html};

/// The HTML of `text`, or its error.
fn html(text: &str) -> String {
    to_html(text).unwrap_or_else(|error| format!("error {error}"))
}

/// The line, column and kind of the error in `text`, or `None` if it renders.
fn error(text: &str) -> Option<(usize, usize, ErrorKind)> {
    to_html(text)
        .err()
        .map(|error| (error.pos.line, error.pos.col, error.kind))
}

#[test]
fn a_paragraph_runs_to_a_blank_line_and_its_text_is_escaped() {
    let text = "One & two\n   a < b, c > d \n\n\n  Three\n";
    let expected = "<p>One &amp; two\na &lt; b, c &gt; d</p>\n<p>Three</p>\n";
    assert_eq!(html(text), expected);
}

#[test]
fn a_heading_has_zola_s_id_made_unique_on_the_page() {
    let text = "# What’s Next?\n## C# ##\n### F#\n###### Q&A\n## What's next\n## what-s-next-1 #\n## WHAT’S NEXT\n";
    let expected = concat!(
        "<h1 id=\"what-s-next\">What’s Next?</h1>\n",
        "<h2 id=\"c\">C#</h2>\n",
        "<h3 id=\"f\">F#</h3>\n",
        "<h6 id=\"q-a\">Q&amp;A</h6>\n",
        "<h2 id=\"what-s-next-1\">What's next</h2>\n",
        "<h2 id=\"what-s-next-1-1\">what-s-next-1</h2>\n",
        "<h2 id=\"what-s-next-2\">WHAT’S NEXT</h2>\n",
    );
    assert_eq!(html(text), expected);
    // Emoji and punctuation outside ASCII only separate the words of an id.
    let expected = "<h2 id=\"rust-docs\">Rust 🦀 — “docs”</h2>\n";
    assert_eq!(html("## Rust 🦀 — “docs”\n"), expected);
}

#[test]
fn a_heading_ends_a_paragraph_but_needs_a_space_after_1_to_6_marks() {
    let expected = "<p>Text\n#hashtag\n####### seven</p>\n<h2 id=\"next\">Next</h2>\n";
    assert_eq!(
        html("Text\n#hashtag\n####### seven\n   ## Next\n"),
        expected
    );
}

#[test]
fn a_heading_needs_an_id_of_ascii_letters_and_digits() {
    let cases = [
        ("## Rust 🦀 Café ##\n", 1, 14, NonAsciiInHeading('é')),
        ("## Step ²\n", 1, 9, NonAsciiInHeading('²')),
        ("\n### ¿?\n", 2, 1, NoHeadingId),
        ("#\n", 1, 1, NoHeadingId),
    ];
    for (text, line, col, kind) in cases {
        assert_eq!(error(text), Some((line, col, kind)), "{text:?}");
    }
}

#[test]
fn indented_code_is_an_error_but_a_paragraph_s_lines_may_be_indented() {
    assert_eq!(error("    code\n"), Some((1, 5, IndentedCode)));
    assert_eq!(error("a\n\n    code\n"), Some((3, 5, IndentedCode)));
    assert_eq!(
        html("a\n     b\n    # c\n    ===\n"),
        "<p>a\nb\n# c\n===</p>\n"
    );
}

#[test]
fn underlined_headings_and_rules_are_errors() {
    let cases = [
        ("Title\n=====\n", 2, 1, UnderlinedHeading),
        ("Title\n -\n", 2, 2, UnderlinedHeading),
        ("***\n", 1, 1, Rule),
        ("text\n - - -\n", 2, 2, Rule),
    ];
    for (text, line, col, kind) in cases {
        assert_eq!(error(text), Some((line, col, kind)), "{text:?}");
    }
}

#[test]
fn tabs_and_other_control_characters_are_errors() {
    assert_eq!(error("a\n#\tb\n"), Some((2, 2, ControlCharacter('\t'))));
    assert_eq!(error("é\u{7}\n"), Some((1, 2, ControlCharacter('\u{7}'))));
}

/// Every block of the subset, and lines one character away from an error, for the random test to
/// damage.
const SAMPLE: &str = "\
## What’s next?

Text & more
   on two lines.
# Rust 🦀 or not ##

   Three spaces
= =

--
";

#[test]
fn a_damaged_text_gives_html_or_an_error_never_a_panic() {
    to_html(SAMPLE).unwrap();
    let sample: Vec<char> = SAMPLE.chars().collect();
    let damage = [' ', '-', '*', '#', '=', '\n', 'é', '\t'];
    // xorshift64 with a fixed seed: every run tries the same 5,000 texts, each with two random
    // characters replaced, inserted or removed.
    let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
    for _ in 0..5_000 {
        let mut text = sample.clone();
        for _ in 0..2 {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let [low, high, how, what, ..] = state.to_le_bytes();
            let at = usize::from(u16::from_le_bytes([low, high])) % text.len();
            let c = damage[usize::from(what) % damage.len()];
            match how % 3 {
                0 => text[at] = c,
                1 => text.insert(at, c),
                _ => _ = text.remove(at),
            }
        }
        let text: String = text.into_iter().collect();
        let _ = to_html(&text);
    }
}
