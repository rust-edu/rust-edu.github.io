//! The blocks of the subset through the public API: the HTML of each, and where each error points.

use edu_markdown::ErrorKind::{
    ControlCharacter, EmptyListItem, HeadingAttributes, IndentedCode, LazyLine, NoHeadingId,
    Noncharacter, Rule, Table, TaskListItem, TooDeep, UnclosedFence, UnderlinedHeading,
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
fn a_heading_s_id_keeps_its_letters_and_digits_in_any_script_in_lowercase() {
    let cases = [
        (
            "## Rust 🦀 Café ##\n",
            "<h2 id=\"rust-café\">Rust 🦀 Café</h2>\n",
        ),
        (
            "## Taller en PERÚ: ¿qué sigue?\n",
            "<h2 id=\"taller-en-perú-qué-sigue\">",
        ),
        ("### 中文 标题 2027\n", "<h3 id=\"中文-标题-2027\">"),
        ("## Step ²\n", "<h2 id=\"step-²\">"),
    ];
    for (text, start) in cases {
        assert!(html(text).starts_with(start), "{text:?}: {}", html(text));
    }
}

#[test]
fn a_heading_needs_a_letter_or_digit_for_its_id() {
    let cases = [
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
fn tables_task_lists_and_heading_attributes_are_errors() {
    let cases = [
        ("a | b\n--|--\n", 2, 1, Table),
        ("text\n| a | b |\n|---|---|\n", 3, 1, Table),
        ("- a\n  |-|\n", 2, 3, Table),
        ("| a |\n| - |\n", 2, 1, Table),
        ("- [ ] a\n", 1, 3, TaskListItem),
        ("1. [x] b\n", 1, 4, TaskListItem),
        ("- [X]\n", 1, 3, TaskListItem),
        ("## Head {#custom}\n", 1, 9, HeadingAttributes),
        ("## Head{.x}\n", 1, 8, HeadingAttributes),
        ("## a {b} ##\n", 1, 6, HeadingAttributes),
    ];
    for (text, line, col, kind) in cases {
        assert_eq!(error(text), Some((line, col, kind)), "{text:?}");
    }
    // Without a `|`, a checkbox's space or a `{`, they are text in Zola too.
    let expected =
        "<p>a\n:-:</p>\n<ul>\n<li>[x]a</li>\n<li>[y] b</li>\n</ul>\n<h2 id=\"a\">a }</h2>\n";
    assert_eq!(html("a\n:-:\n\n- [x]a\n- [y] b\n\n## a }\n"), expected);
}

#[test]
fn tabs_and_other_control_characters_are_errors() {
    assert_eq!(error("a\n#\tb\n"), Some((2, 2, ControlCharacter('\t'))));
    assert_eq!(error("é\u{7}\n"), Some((1, 2, ControlCharacter('\u{7}'))));
}

#[test]
fn noncharacters_are_errors_in_code_too() {
    for c in [
        '\u{fdd0}',
        '\u{fdef}',
        '\u{fffe}',
        '\u{ffff}',
        '\u{1fffe}',
        '\u{10ffff}',
    ] {
        assert_eq!(
            error(&format!("a{c}\n")),
            Some((1, 2, Noncharacter(c))),
            "{c:?}"
        );
    }
    assert_eq!(
        error("`a\u{ffff}`\n"),
        Some((1, 3, Noncharacter('\u{ffff}')))
    );
    let message = "1:1: U+FFFE is a Unicode noncharacter, not text; delete it";
    assert_eq!(to_html("\u{fffe}").unwrap_err().to_string(), message);
    // The code points next to them are characters.
    let text = "\u{fdcf}\u{fdf0}\u{fffd}";
    assert_eq!(html(&format!("{text}\n")), format!("<p>{text}</p>\n"));
}

#[test]
fn fenced_code_keeps_its_lines_as_written() {
    let text = "  ```rust title\n  let x = a < b;\n\n    *y*\n z\n ```\n~~~~\n```\n~~~~~\n";
    let expected = concat!(
        "<pre><code class=\"language-rust\">let x = a &lt; b;\n\n  *y*\nz\n</code></pre>\n",
        "<pre><code>```\n</code></pre>\n",
    );
    assert_eq!(html(text), expected);
    let expected = "<pre><code class=\"language-a&quot;b\"></code></pre>\n";
    assert_eq!(html("~~~ a\"b\n~~~\n"), expected);
}

#[test]
fn a_code_block_needs_its_closing_fence() {
    assert_eq!(
        error("text\n\n  ~~~\n  code\n  ```\n"),
        Some((3, 3, UnclosedFence))
    );
    assert_eq!(error("````\n```\n"), Some((1, 1, UnclosedFence)));
    assert_eq!(error("```\n    ```\n"), Some((1, 1, UnclosedFence)));
}

#[test]
fn a_fence_is_3_marks_or_more_and_after_backticks_has_no_backtick() {
    // The first line is paragraph text, so the fence under it has no closing fence.
    assert_eq!(error("``\n```\n"), Some((2, 1, UnclosedFence)));
    assert_eq!(error("```a`b\n```\n"), Some((2, 1, UnclosedFence)));
    let expected = "<pre><code class=\"language-a`b\"></code></pre>\n";
    assert_eq!(html("~~~a`b\n~~~\n"), expected);
}

#[test]
fn a_quote_holds_blocks_and_ends_at_a_blank_line() {
    let text = "> a\n> b\n>\n> - c\n\n> d\n";
    let expected = concat!(
        "<blockquote>\n<p>a\nb</p>\n<ul>\n<li>c</li>\n</ul>\n</blockquote>\n",
        "<blockquote>\n<p>d</p>\n</blockquote>\n",
    );
    assert_eq!(html(text), expected);
    let expected = "<ul>\n<li>a\n<blockquote>\n<p>b</p>\n</blockquote>\n</li>\n</ul>\n";
    assert_eq!(html("- a\n  > b\n"), expected);
}

#[test]
fn a_list_is_loose_when_a_blank_line_separates_its_items_or_their_blocks() {
    let tight = "- a\n- b\n  c\n";
    assert_eq!(html(tight), "<ul>\n<li>a</li>\n<li>b\nc</li>\n</ul>\n");
    let expected = "<ul>\n<li>\n<p>a</p>\n</li>\n<li>\n<p>b</p>\n</li>\n</ul>\n";
    assert_eq!(html("- a\n\n- b\n"), expected);
    let expected = "<ul>\n<li>\n<p>a</p>\n<p>b</p>\n</li>\n<li>\n<p>c</p>\n</li>\n</ul>\n";
    assert_eq!(html("* a\n\n  b\n* c\n"), expected);
    let expected = "<ul>\n<li>\n<p>a</p>\n<ul>\n<li>b</li>\n</ul>\n<p>c</p>\n</li>\n</ul>\n";
    assert_eq!(html("- a\n  - b\n\n  c\n"), expected);
}

#[test]
fn a_list_nests_in_an_item_by_the_column_of_the_item_s_text() {
    let text = "* a\n\n  * b\n\n  * c\n* d\n  - e\n";
    let expected = concat!(
        "<ul>\n<li>\n<p>a</p>\n<ul>\n<li>\n<p>b</p>\n</li>\n<li>\n<p>c</p>\n</li>\n</ul>\n</li>\n",
        "<li>\n<p>d</p>\n<ul>\n<li>e</li>\n</ul>\n</li>\n</ul>\n",
    );
    assert_eq!(html(text), expected);
    let expected = "<ul>\n<li>a\n<ul>\n<li>b</li>\n</ul>\n</li>\n<li>c</li>\n</ul>\n";
    assert_eq!(html("-   a\n    - b\n  - c\n"), expected);
}

#[test]
fn a_heading_or_code_in_a_list_item_follows_the_item_s_text_as_in_zola() {
    let expected = "<ul>\n<li><h1 id=\"a\">a</h1>\n</li>\n</ul>\n";
    assert_eq!(html("- # a\n"), expected);
    let expected = "<ul>\n<li>a<pre><code>x\n</code></pre>\n</li>\n</ul>\n";
    assert_eq!(html("* a\n  ```\n  x\n  ```\n"), expected);
}

#[test]
fn a_numbered_list_starts_at_its_first_number_of_up_to_9_digits() {
    assert_eq!(
        html("1. a\n1. b\n"),
        "<ol>\n<li>a</li>\n<li>b</li>\n</ol>\n"
    );
    assert_eq!(html("007) a\n"), "<ol start=\"7\">\n<li>a</li>\n</ol>\n");
    let expected = "<ol start=\"123456789\">\n<li>a</li>\n</ol>\n";
    assert_eq!(html("123456789. a\n"), expected);
    assert_eq!(html("1234567890. a\n"), "<p>1234567890. a</p>\n");
}

#[test]
fn another_marker_starts_another_list() {
    let expected = concat!(
        "<ul>\n<li>a</li>\n</ul>\n<ul>\n<li>b</li>\n</ul>\n<ul>\n<li>c</li>\n</ul>\n",
        "<ol>\n<li>d</li>\n</ol>\n<ol start=\"2\">\n<li>e</li>\n</ol>\n",
    );
    assert_eq!(html("* a\n- b\n+ c\n1. d\n2) e\n"), expected);
}

#[test]
fn a_list_ends_a_paragraph_if_its_first_item_has_text_and_a_bullet_or_1() {
    assert_eq!(html("a\n- b\n"), "<p>a</p>\n<ul>\n<li>b</li>\n</ul>\n");
    assert_eq!(html("a\n1. b\n"), "<p>a</p>\n<ol>\n<li>b</li>\n</ol>\n");
    assert_eq!(html("In\n2026. b\n"), "<p>In\n2026. b</p>\n");
    assert_eq!(html("a\n*\n1.\n"), "<p>a\n*\n1.</p>\n");
    assert_eq!(html("- a\n  2. b\n"), "<ul>\n<li>a\n2. b</li>\n</ul>\n");
}

#[test]
fn a_line_that_continues_an_item_or_quote_needs_its_indentation_or_marker() {
    assert_eq!(error("- a\nb\n"), Some((2, 1, LazyLine)));
    assert_eq!(error("1. a\n  b\n"), Some((2, 3, LazyLine)));
    assert_eq!(error(" - a\n  b\n"), Some((2, 3, LazyLine)));
    assert_eq!(error("> a\n    b\n"), Some((2, 5, LazyLine)));
    assert_eq!(error("> a\n    > b\n"), Some((2, 5, LazyLine)));
    assert_eq!(html("- a\n\nb\n"), "<ul>\n<li>a</li>\n</ul>\n<p>b</p>\n");
    let expected = "<blockquote>\n<p>a</p>\n</blockquote>\n<p>b</p>\n";
    assert_eq!(html("> a\n>\nb\n"), expected);
    // A list item ends a quote, even one that could not end a paragraph.
    let expected = "<blockquote>\n<p>a</p>\n</blockquote>\n<ol start=\"2\">\n<li>b</li>\n</ol>\n";
    assert_eq!(html("> a\n2. b\n"), expected);
}

#[test]
fn a_list_item_needs_a_space_and_text_after_its_marker() {
    assert_eq!(html("-b\n1.c\n"), "<p>-b\n1.c</p>\n");
    assert_eq!(error("- a\n-\n"), Some((2, 1, EmptyListItem)));
    assert_eq!(error("text\n\n1.   \n"), Some((3, 1, EmptyListItem)));
}

#[test]
fn lists_and_quotes_nest_at_most_8_deep() {
    let eight = format!("{}a\n", "> ".repeat(8));
    let (open, close) = ("<blockquote>\n".repeat(8), "</blockquote>\n".repeat(8));
    assert_eq!(html(&eight), format!("{open}<p>a</p>\n{close}"));
    let nine = format!("{}a\n", "- ".repeat(9));
    assert_eq!(error(&nine), Some((1, 17, TooDeep)));
}

#[test]
fn indented_code_in_a_list_item_or_quote_counts_from_its_text() {
    assert_eq!(error("- a\n\n      code\n"), Some((3, 7, IndentedCode)));
    // Text 1 to 4 spaces after the marker sets the item's column; 5 spaces make indented code.
    assert_eq!(
        html("-    a\n\n  b\n"),
        "<ul>\n<li>a</li>\n</ul>\n<p>b</p>\n"
    );
    assert_eq!(error("-     code\n"), Some((1, 7, IndentedCode)));
    assert_eq!(error(">     code\n"), Some((1, 7, IndentedCode)));
    assert_eq!(html(">    a\n"), "<blockquote>\n<p>a</p>\n</blockquote>\n");
}

/// Every block of the subset, and lines one character away from an error, for the random test to
/// damage.
const SAMPLE: &str = "\
## What’s next?

Text & more
   on two lines.
# Rust 🦀 or not ##

> A quote
>
> - with a list

* a

  1. b
  2. c
     d
- e

```rust
let x = 1;
```

   Three spaces
= =

--
";

#[test]
fn a_damaged_text_gives_html_or_an_error_never_a_panic() {
    to_html(SAMPLE).unwrap();
    let sample: Vec<char> = SAMPLE.chars().collect();
    let damage = [' ', '-', '*', '#', '>', '`', '1', '.', '=', '\n', 'é', '\t'];
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
