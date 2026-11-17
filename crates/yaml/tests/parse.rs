//! The subset through its public API: what each text parses to, and where each error points.

use edu_yaml::ErrorKind::{
    BadIndent, ColonInValue, ControlCharacter, DuplicateKey, ExpectedKey, ExpectedListItem,
    Indicator, InvalidEscape, MissingHeader, MissingValue, Noncharacter, TextAfterQuote,
    UnclosedHeader, UnclosedQuote,
};
use edu_yaml::{Entry, ErrorKind, Node, Pos, Value, parse, parse_front_matter};

/// The parsed text in short form, as in `{key: "text", list: ["item"], map: {key: "text"}}`, or
/// the error.
fn shape(text: &str) -> String {
    match parse(text) {
        Ok(entries) => map_shape(&entries),
        Err(error) => format!("error {error}"),
    }
}

fn map_shape(entries: &[Entry]) -> String {
    let entries: Vec<_> = entries
        .iter()
        .map(|entry| format!("{}: {}", entry.key, node_shape(&entry.value)))
        .collect();
    format!("{{{}}}", entries.join(", "))
}

fn node_shape(node: &Node) -> String {
    match &node.value {
        Value::Str(text) => format!("{text:?}"),
        Value::List(items) => {
            let items: Vec<_> = items.iter().map(node_shape).collect();
            format!("[{}]", items.join(", "))
        }
        Value::Map(entries) => map_shape(entries),
    }
}

/// The line, column and kind of the error in `text`, or `None` if it parses.
fn error(text: &str) -> Option<(usize, usize, ErrorKind)> {
    parse(text)
        .err()
        .map(|error| (error.pos.line, error.pos.col, error.kind))
}

#[test]
fn a_page_header_is_a_map_of_strings_and_lists() {
    let text =
        "title: Rust Week 2027\ndate: 2027-03-01\nauthors:\n  - Jane Doe\n  - Zoë Ng # a comment\n";
    let expected =
        r#"{title: "Rust Week 2027", date: "2027-03-01", authors: ["Jane Doe", "Zoë Ng"]}"#;
    assert_eq!(shape(text), expected);
}

#[test]
fn maps_nest_under_keys() {
    let text = "footer:\n    license:\n      name: CC0\n    year: 2026\nnext: x\n";
    let expected = r#"{footer: {license: {name: "CC0"}, year: "2026"}, next: "x"}"#;
    assert_eq!(shape(text), expected);
}

#[test]
fn a_list_item_can_be_a_map_whose_keys_line_up_after_the_dash() {
    let text = "menu:\n  - name: News\n    url: /news/\n  -   name: Zulip\n      url: /zulip/\n";
    let expected = r#"{menu: [{name: "News", url: "/news/"}, {name: "Zulip", url: "/zulip/"}]}"#;
    assert_eq!(shape(text), expected);
}

#[test]
fn every_scalar_is_a_string() {
    let text = "draft: true\ncountry: no\ncount: 007\nnothing: ~\ntime: 10:30\n";
    let expected = r#"{draft: "true", country: "no", count: "007", nothing: "~", time: "10:30"}"#;
    assert_eq!(shape(text), expected);
}

#[test]
fn quotes_hold_any_text_and_double_quotes_have_four_escapes() {
    let text = r#"single: 'it''s: #1'
double: "a \"b\" \\ c\n\t"
empty: ''
plain: it's "quoted" text
url: https://rust-edu.org/#news
"#;
    let expected = concat!(
        r#"{single: "it's: #1", double: "a \"b\" \\ c\n\t", empty: "", "#,
        r#"plain: "it's \"quoted\" text", url: "https://rust-edu.org/#news"}"#,
    );
    assert_eq!(shape(text), expected);
}

#[test]
fn comments_and_blank_lines_carry_nothing() {
    let text = "\
# A comment line, then a blank line.

title: C# for Rust users # a comment after a space
  # an indented comment line
footer: # a comment instead of a value
  license: CC0 # zero
";
    let expected = r#"{title: "C# for Rust users", footer: {license: "CC0"}}"#;
    assert_eq!(shape(text), expected);
}

#[test]
fn keys_and_values_know_where_they_start() {
    let entries = parse("a: x\nmap:\n  k: 'v'\nlist:\n  - y\n  - k: v\n").unwrap();
    let Value::Map(map) = &entries[1].value.value else {
        panic!("not a map: {entries:?}");
    };
    let Value::List(items) = &entries[2].value.value else {
        panic!("not a list: {entries:?}");
    };
    let Value::Map(item) = &items[1].value else {
        panic!("not a map: {items:?}");
    };
    let at = |pos: Pos| (pos.line, pos.col);
    assert_eq!(at(entries[0].pos), (1, 1));
    assert_eq!(at(entries[0].value.pos), (1, 4));
    assert_eq!(at(entries[1].pos), (2, 1));
    assert_eq!(at(entries[1].value.pos), (3, 3));
    assert_eq!(at(map[0].pos), (3, 3));
    assert_eq!(at(map[0].value.pos), (3, 6));
    assert_eq!(at(entries[2].value.pos), (5, 3));
    assert_eq!(at(items[0].pos), (5, 5));
    assert_eq!(at(items[1].pos), (6, 5));
    assert_eq!(at(item[0].pos), (6, 5));
    assert_eq!(at(item[0].value.pos), (6, 8));
}

#[test]
fn columns_count_characters_not_bytes() {
    // `ë` takes two bytes in UTF-8.
    assert_eq!(error("a: \"Zoë\" x\n"), Some((1, 10, TextAfterQuote)));
}

#[test]
fn a_page_header_lies_between_two_dashed_lines() {
    let (header, body) = parse_front_matter("---\ntitle: A\n---\nText\n---\nmore\n").unwrap();
    assert_eq!(map_shape(&header), r#"{title: "A"}"#);
    assert_eq!(body, "Text\n---\nmore\n");
    let (header, body) = parse_front_matter("---\r\ntitle: A\r\n---").unwrap();
    assert_eq!((map_shape(&header).as_str(), body), (r#"{title: "A"}"#, ""));
}

#[test]
fn a_byte_order_mark_at_the_start_is_left_out() {
    // Some editors on Windows start a UTF-8 file with U+FEFF, which shows as nothing.
    let (header, body) = parse_front_matter("\u{feff}---\ntitle: A\n---\nText\n").unwrap();
    assert_eq!(
        (map_shape(&header).as_str(), body),
        (r#"{title: "A"}"#, "Text\n")
    );
    assert_eq!(shape("\u{feff}a: b\n"), r#"{a: "b"}"#);
    let error = parse_front_matter("\u{feff}title: A\n").unwrap_err();
    assert_eq!(
        (error.pos.line, error.pos.col, error.kind),
        (1, 1, MissingHeader)
    );
}

#[test]
fn header_errors_count_lines_from_the_top_of_the_page() {
    let error = parse_front_matter("---\ntitle: A\ndraft: no: yes\n---\n").unwrap_err();
    assert_eq!(
        (error.pos.line, error.pos.col, error.kind),
        (3, 8, ColonInValue)
    );
    let pages = [
        ("title: A\n", MissingHeader),
        ("", MissingHeader),
        (" ---\ntitle: A\n---\n", MissingHeader),
        ("---\ntitle: A\n", UnclosedHeader),
    ];
    for (page, kind) in pages {
        let error = parse_front_matter(page).unwrap_err();
        assert_eq!(
            (error.pos.line, error.pos.col, error.kind),
            (1, 1, kind),
            "{page:?}"
        );
    }
}

#[test]
fn tabs_and_other_control_characters_are_errors() {
    assert_eq!(error("a:\n\tb: x\n"), Some((2, 1, ControlCharacter('\t'))));
    assert_eq!(error("a: \"b\tc\"\n"), Some((1, 6, ControlCharacter('\t'))));
    assert_eq!(error("a: b\rc\n"), Some((1, 5, ControlCharacter('\r'))));
}

#[test]
fn noncharacters_are_errors() {
    let c = '\u{fffe}';
    assert_eq!(
        error(&format!("a: \"b{c}\"\n")),
        Some((1, 6, Noncharacter(c)))
    );
    let c = '\u{fdd0}';
    assert_eq!(
        error(&format!("a:\n  - b{c}\n")),
        Some((2, 6, Noncharacter(c)))
    );
    let header = parse_front_matter("---\ntitle: \u{10ffff}\n---\n").unwrap_err();
    let found = (header.pos.line, header.pos.col, header.kind);
    assert_eq!(found, (2, 8, Noncharacter('\u{10ffff}')));
}

#[test]
fn a_line_deeper_than_its_block_must_follow_a_key() {
    // YAML would join these lines to the value above.
    assert_eq!(error("title: A long\n  title\n"), Some((2, 3, BadIndent)));
    assert_eq!(error("a:\n  b: x\n    y\n"), Some((3, 5, BadIndent)));
    assert_eq!(error("a:\n  - x\n    y\n"), Some((3, 5, BadIndent)));
    // These line up with no block.
    assert_eq!(error("a:\n    b: x\n  c: y\n"), Some((3, 3, BadIndent)));
    assert_eq!(error("  a: x\n"), Some((1, 3, BadIndent)));
}

#[test]
fn a_map_line_is_a_key_a_colon_and_a_space() {
    let texts = [
        "title = A\n",
        "title:A\n",
        "title : A\n",
        "\"title\": A\n",
        "café: A\n",
    ];
    for text in texts.into_iter().chain(["<<: A\n"]) {
        assert_eq!(error(text), Some((1, 1, ExpectedKey)), "{text:?}");
    }
    let key = "a".to_owned();
    let duplicate = DuplicateKey { key, first_line: 1 };
    assert_eq!(error("a: 1\nb: 2\na: 3\n"), Some((3, 1, duplicate)));
}

#[test]
fn a_list_line_is_a_dash_a_space_and_a_value() {
    assert_eq!(error("a:\n  - x\n  y\n"), Some((3, 3, ExpectedListItem)));
    assert_eq!(error("a:\n  - x\n  -y\n"), Some((3, 3, ExpectedListItem)));
    assert_eq!(error("a:\n  -\n"), Some((2, 3, MissingValue)));
    assert_eq!(error("a:\n  - # nothing\n"), Some((2, 3, MissingValue)));
}

#[test]
fn a_key_has_a_value_on_its_line_or_indented_below() {
    assert_eq!(error("draft:\n"), Some((1, 1, MissingValue)));
    assert_eq!(error("footer:\ntitle: A\n"), Some((1, 1, MissingValue)));
    // YAML allows a list under its key without indentation; the subset does not.
    assert_eq!(error("authors:\n- Jane\n"), Some((1, 1, MissingValue)));
}

#[test]
fn a_quote_closes_on_its_line_and_ends_the_value() {
    assert_eq!(error("a: 'b\n"), Some((1, 4, UnclosedQuote)));
    assert_eq!(error("a: \"b\\\"\n"), Some((1, 4, UnclosedQuote)));
    assert_eq!(error("a: \"caf\\u00e9\"\n"), Some((1, 8, InvalidEscape)));
    assert_eq!(error("a: \"b\" c\n"), Some((1, 8, TextAfterQuote)));
    assert_eq!(error("a: 'b'# c\n"), Some((1, 7, TextAfterQuote)));
}

#[test]
fn a_plain_value_with_a_colon_and_a_space_needs_quotes() {
    let campus_tour = "title: Campus Tour: Rust Africa\n";
    assert_eq!(error(campus_tour), Some((1, 8, ColonInValue)));
    assert_eq!(error("title: Note:\n"), Some((1, 8, ColonInValue)));
    assert_eq!(error("a:\n  b: c: d\n"), Some((2, 6, ColonInValue)));
    assert_eq!(error("a:\n  - b: c: d\n"), Some((2, 8, ColonInValue)));
}

#[test]
fn yaml_outside_the_subset_is_named_by_its_first_character() {
    let cases = [
        ("a: &x b\n", 1, 4, '&'),
        ("a: *x\n", 1, 4, '*'),
        ("a: !!str b\n", 1, 4, '!'),
        ("a: [b, c]\n", 1, 4, '['),
        ("a: {b: c}\n", 1, 4, '{'),
        ("a: |\n  b\n", 1, 4, '|'),
        ("a: >\n  b\n", 1, 4, '>'),
        ("? a\n", 1, 1, '?'),
        ("a: - b\n", 1, 4, '-'),
        ("- a\n", 1, 1, '-'),
        ("a:\n  - - b\n", 2, 5, '-'),
        ("a: @b\n", 1, 4, '@'),
        ("a:\n  b: [c]\n", 2, 6, '['),
        ("a:\n  - [b]\n", 2, 5, '['),
    ];
    for (text, line, col, c) in cases {
        assert_eq!(error(text), Some((line, col, Indicator(c))), "{text:?}");
    }
}

/// Every construct of the subset, for the random test to damage.
const SAMPLE: &str = "\
title: 'It''s: \"quoted\"' # a comment
date: 2027-03-01
authors:
  - Zoë Ng
  - \"Jane \\\"JD\\\" Doe\"

# The menu
menu:
  - name: News
    url: /news/#top
  -   name: Zulip
      links:
        - a
footer:
    license:
      name: CC0
";

#[test]
fn a_damaged_text_gives_an_error_or_a_value_never_a_panic() {
    parse(SAMPLE).unwrap();
    let sample: Vec<char> = SAMPLE.chars().collect();
    let damage = [
        ' ', '-', ':', '#', '\'', '"', '\\', '\n', 'é', '[', '|', '\t',
    ];
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
        let _ = parse(&text);
        let _ = parse_front_matter(&format!("---\n{text}---\n"));
    }
}
