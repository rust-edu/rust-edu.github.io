//! The template language through its public API: what each template renders with a context, and
//! where each error points.

use edu_template::ErrorKind::{
    BadPath, BadTag, NotText, UnclosedTag, UnknownFilter, UnknownVariable,
};
use edu_template::{ErrorKind, Template, Value};

fn text(text: &str) -> Value {
    Value::Str(text.to_owned())
}

fn map(entries: &[(&str, Value)]) -> Value {
    let entries = entries
        .iter()
        .map(|(name, value)| ((*name).to_owned(), value.clone()));
    Value::Map(entries.collect())
}

/// The context of the tests: a site with two pages, and a page of its own.
fn context() -> Value {
    let pages = Value::List(vec![
        map(&[("title", text("One")), ("url", text("/one/"))]),
        map(&[("title", text("Two & <three>")), ("url", text("/two/"))]),
    ]);
    let site = map(&[("title", text("Rust Edu")), ("pages", pages)]);
    let page = map(&[
        ("title", text("A \"quoted\" 'title'")),
        ("content", text("<p>Text</p>")),
        ("draft", Value::Bool(false)),
        ("note", text("1 < 2 & 3 > 2")),
    ]);
    map(&[("site", site), ("page", page), ("empty", text(""))])
}

/// What `template` renders with the test context, or its error.
fn render(template: &str) -> String {
    let rendered = Template::parse(template).and_then(|template| template.render(&context()));
    rendered.unwrap_or_else(|error| format!("error {error}"))
}

/// The line, column and kind of the error in `template`, or `None` if it renders.
fn error(template: &str) -> Option<(usize, usize, ErrorKind)> {
    let rendered = Template::parse(template).and_then(|template| template.render(&context()));
    rendered
        .err()
        .map(|error| (error.pos.line, error.pos.col, error.kind))
}

#[test]
fn a_value_is_escaped_unless_it_is_safe() {
    let template = "<h1 title=\"{{ page.title }}\">{{page.title}}</h1>\n{{ page.content | safe }}";
    let title = "A &quot;quoted&quot; &#39;title&#39;";
    let expected = format!("<h1 title=\"{title}\">{title}</h1>\n<p>Text</p>");
    assert_eq!(render(template), expected);
    let template = "{{ site.title }}: {{ page.content|safe }}{{ empty }}. {{ page.note }}";
    let expected = "Rust Edu: <p>Text</p>. 1 &lt; 2 &amp; 3 &gt; 2";
    assert_eq!(render(template), expected);
}

#[test]
fn other_text_is_kept_and_comments_are_left_out() {
    assert_eq!(
        render("a {b} { %c %} }} %}{# {{ note }} #}d"),
        "a {b} { %c %} }} %}d"
    );
}

#[test]
fn a_tag_outside_the_language_is_an_error_at_its_first_word() {
    let cases = [
        ("a {{ page.title", 1, 3, UnclosedTag("}}")),
        ("{% if page.draft\n", 1, 1, UnclosedTag("%}")),
        ("{# note", 1, 1, UnclosedTag("#}")),
        (
            "{% extends \"base.html\" %}",
            1,
            4,
            BadTag("extends \"base.html\"".into()),
        ),
        (
            "a\n{%  if page.draft %}",
            2,
            5,
            BadTag("if page.draft".into()),
        ),
        (
            "{{ page.authors | join }}",
            1,
            4,
            UnknownFilter("join".into()),
        ),
        ("\n  {{ page-title }}", 2, 6, BadPath("page-title".into())),
        ("{{ }}", 1, 4, BadPath(String::new())),
        ("{{ page..title }}", 1, 4, BadPath("page..title".into())),
        ("{{ 1st }}", 1, 4, BadPath("1st".into())),
    ];
    for (template, line, col, kind) in cases {
        assert_eq!(error(template), Some((line, col, kind)), "{template:?}");
    }
}

#[test]
fn a_value_that_is_missing_or_not_text_is_an_error_at_its_path() {
    let cases = [
        (
            "<p>{{ page.titel }}</p>",
            1,
            7,
            UnknownVariable("page.titel".into()),
        ),
        (
            "é {{ page.titel }}",
            1,
            6,
            UnknownVariable("page.titel".into()),
        ),
        (
            "{{ site.title.text }}",
            1,
            4,
            UnknownVariable("site.title.text".into()),
        ),
        ("a\n {{ site.pages }}", 2, 5, NotText("site.pages".into())),
        ("{{ page.draft }}", 1, 4, NotText("page.draft".into())),
        ("{{ site | safe }}", 1, 4, NotText("site".into())),
    ];
    for (template, line, col, kind) in cases {
        assert_eq!(error(template), Some((line, col, kind)), "{template:?}");
    }
}

/// Every construct of the language, for the random test to damage.
const SAMPLE: &str = "\
<title>{{ page.title }} - {{ site.title }}</title>{# a comment #}
<main>{{ page.content | safe }}</main>
";

#[test]
fn a_damaged_template_gives_html_or_an_error_never_a_panic() {
    let context = context();
    Template::parse(SAMPLE).unwrap().render(&context).unwrap();
    let sample: Vec<char> = SAMPLE.chars().collect();
    let damage = ['{', '}', '%', '#', '|', '.', ' ', '\n', 'é'];
    // xorshift64 with a fixed seed: every run tries the same 5,000 templates, each with two random
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
        let _ = Template::parse(&text).and_then(|template| template.render(&context));
    }
}
