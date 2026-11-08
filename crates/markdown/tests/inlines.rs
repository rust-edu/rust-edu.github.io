//! The inlines of the subset through the public API: the HTML of each, and where each error
//! points.

use edu_markdown::ErrorKind::{
    BareUrl, EmphasisNextTo, EmphasisRun, Entity, Footnote, LinkDestination, NestedLink,
    NoHeadingId, RawHtml, ReferenceLink, Strikethrough, UnmatchedEmphasis,
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
fn a_code_span_ends_at_as_many_backticks_and_keeps_its_text() {
    let text = "a `code` b `` x`y `` c ` d ` e `  ` f `&amp; <b>`\n";
    let expected = concat!(
        "<p>a <code>code</code> b <code>x`y</code> c <code>d</code> e <code>  </code> f ",
        "<code>&amp;amp; &lt;b&gt;</code></p>\n",
    );
    assert_eq!(html(text), expected);
    // Line breaks become spaces; a run of backticks without its match is text.
    let expected = "<p><code>a b</code> <code>x   y</code> <code>a``b</code> ``z`</p>\n";
    assert_eq!(html("`a\nb` `x  \ny` `a``b` ``z`\n"), expected);
}

#[test]
fn an_autolink_is_a_url_or_an_email_address_between_angle_brackets() {
    let text = concat!(
        "<https://rust-edu.org> <https://a.org/é?x=1&y> <irc://foo> <a.b@c-d.e> ",
        "a < b <1a:b> <@b>\n",
    );
    let expected = concat!(
        "<p><a rel=\"external\" href=\"https://rust-edu.org\">https://rust-edu.org</a> ",
        "<a rel=\"external\" href=\"https://a.org/%C3%A9?x=1&amp;y\">",
        "https://a.org/é?x=1&amp;y</a> ",
        "<a href=\"irc://foo\">irc://foo</a> <a href=\"mailto:a.b@c-d.e\">a.b@c-d.e</a> ",
        "a &lt; b &lt;1a:b&gt; &lt;@b&gt;</p>\n",
    );
    assert_eq!(html(text), expected);
    // Zola makes this URL relative to the page too, as it does relative links; CommonMark keeps it.
    let expected = "<p><a href=\"svn+ssh://x.org\">svn+ssh://x.org</a></p>\n";
    assert_eq!(html("<svn+ssh://x.org>\n"), expected);
}

#[test]
fn a_url_is_escaped_and_marked_external_as_in_zola() {
    let text = "<https://x.org/\"'{|}^[]`\\(é)> <HTTP://x.org> <httpx://z>\n";
    let expected = concat!(
        "<p><a rel=\"external\" href=\"https://x.org/%22&#x27;%7B%7C%7D^%5B%5D%60%5C(%C3%A9)\">",
        "https://x.org/\"'{|}^[]`\\(é)</a> <a href=\"HTTP://x.org\">HTTP://x.org</a> ",
        "<a href=\"httpx://z\">httpx://z</a></p>\n",
    );
    assert_eq!(html(text), expected);
    // Zola's safe characters are kept as typed.
    let url = "http://x.org/!#$%()*+,-./:;=?@^_~";
    let expected = format!("<p><a rel=\"external\" href=\"{url}\">{url}</a></p>\n");
    assert_eq!(html(&format!("<{url}>\n")), expected);
}

#[test]
fn backslash_escapes_and_the_four_references_are_text() {
    let text = "\\* \\_ \\[ \\` \\< \\\\ \\a \\é \\&amp; &amp; &lt; &gt; &quot; AT&T & x; &#; &;\n";
    let expected = concat!(
        "<p>* _ [ ` &lt; \\ \\a \\é &amp;amp; &amp; &lt; &gt; \" AT&amp;T &amp; x; ",
        "&amp;#; &amp;;</p>\n",
    );
    assert_eq!(html(text), expected);
}

#[test]
fn two_spaces_or_a_backslash_before_a_line_break_keep_it() {
    let text = "one  \ntwo \nthree\\\nfour  \\\n`five`   \nsix\\  \n";
    let expected =
        "<p>one<br />\ntwo\nthree<br />\nfour  <br />\n<code>five</code><br />\nsix\\</p>\n";
    assert_eq!(html(text), expected);
    let expected = "<ul>\n<li>a<br />\nb</li>\n<li>c</li>\n</ul>\n";
    assert_eq!(html("- a  \n  b\n- c  \n"), expected);
    let expected = "<blockquote>\n<p>a<br />\nb</p>\n</blockquote>\n";
    assert_eq!(html("> a  \n> b  \n"), expected);
}

#[test]
fn a_link_s_url_balances_its_parentheses_and_is_escaped_as_in_zola() {
    let text =
        "[a](https://e.org/a_(b)?x=1&y=2) [`b`](/news/) [c [d] e](/x/#top) [f](mailto:a@b.c)\n";
    let expected = concat!(
        "<p><a rel=\"external\" href=\"https://e.org/a_(b)?x=1&amp;y=2\">a</a> ",
        "<a href=\"/news/\"><code>b</code></a> <a href=\"/x/#top\">c [d] e</a> ",
        "<a href=\"mailto:a@b.c\">f</a></p>\n",
    );
    assert_eq!(html(text), expected);
    let text =
        "[a](/é?q=ü#ß) [b](/x%20y\\)z) [c](/a\"b'c<d>{e}|^\\[\\]`\\\\) [d](/?q=&quot;&amp;&lt;)\n";
    let expected = concat!(
        "<p><a href=\"/%C3%A9?q=%C3%BC#%C3%9F\">a</a> <a href=\"/x%20y)z\">b</a> ",
        "<a href=\"/a%22b&#x27;c%3Cd%3E%7Be%7D%7C^%5B%5D%60%5C\">c</a> ",
        "<a href=\"/?q=%22&amp;%3C\">d</a></p>\n",
    );
    assert_eq!(html(text), expected);
    // A `\` before anything but ASCII punctuation stays in the URL.
    let expected = "<p><a href=\"/%5Ca%5C%C3%A9\">e</a></p>\n";
    assert_eq!(html("[e](/\\a\\é)\n"), expected);
}

#[test]
fn brackets_without_a_url_right_after_them_are_text() {
    let expected = "<p>[a] (b) [c]\n(d) e](f) [g [h <a href=\"/j\">i</a></p>\n";
    assert_eq!(html("[a] (b) [c]\n(d) e](f) [g [h [i](/j)\n"), expected);
}

#[test]
fn an_image_s_description_is_its_text_without_markup() {
    let text = concat!(
        "![Ferris & \"friends\" `<x>`](/a.png) [![b](/b.png)](/c) ![](/d.png) ",
        "![e [f](/g)\nh](/i)\n",
    );
    let expected = concat!(
        "<p><img src=\"/a.png\" alt=\"Ferris &amp; &quot;friends&quot; &lt;x&gt;\" /> ",
        "<a href=\"/c\"><img src=\"/b.png\" alt=\"b\" /></a> <img src=\"/d.png\" alt=\"\" /> ",
        "<img src=\"/i\" alt=\"e fh\" /></p>\n",
    );
    assert_eq!(html(text), expected);
}

#[test]
fn a_heading_s_id_is_made_from_its_text_without_markup_or_images() {
    let text = concat!(
        "## The `foo` crate\n## A &amp; B\n## a\\*b\\#\n## <https://x.org> and `<b>`\n",
        "## [Docs](/x) &amp; more\n## ![logo](/i.png) [![x](/i.png) Rust](/l)\n",
    );
    let expected = concat!(
        "<h2 id=\"the-foo-crate\">The <code>foo</code> crate</h2>\n",
        "<h2 id=\"a-b\">A &amp; B</h2>\n",
        "<h2 id=\"a-b-1\">a*b#</h2>\n",
        "<h2 id=\"https-x-org-and-b\"><a rel=\"external\" href=\"https://x.org\">",
        "https://x.org</a> and <code>&lt;b&gt;</code></h2>\n",
        "<h2 id=\"docs-more\"><a href=\"/x\">Docs</a> &amp; more</h2>\n",
        "<h2 id=\"rust\"><img src=\"/i.png\" alt=\"logo\" /> ",
        "<a href=\"/l\"><img src=\"/i.png\" alt=\"x\" /> Rust</a></h2>\n",
    );
    assert_eq!(html(text), expected);
    let expected = "<h2 id=\"a-é\">a <code>é</code></h2>\n";
    assert_eq!(html("## a `é`\n"), expected);
    assert_eq!(error("## `--`\n"), Some((1, 1, NoHeadingId)));
    assert_eq!(error("## ![Rust](/i.png)\n"), Some((1, 1, NoHeadingId)));
}

#[test]
fn html_and_other_references_are_errors() {
    let cases = [
        ("<b>bold</b>\n", 1, 1, RawHtml),
        ("a\n`b` Vec<T>\n", 2, 8, RawHtml),
        ("x </i> <!-- c -->\n", 1, 3, RawHtml),
        ("<?php ?>\n", 1, 1, RawHtml),
        ("<http://a b>\n", 1, 1, RawHtml),
        ("<https://a<b>\n", 1, 1, RawHtml),
        ("<x:y>\n", 1, 1, RawHtml),
        ("<a@b..c>\n", 1, 1, RawHtml),
        ("<a@-b.c>\n", 1, 1, RawHtml),
        ("<a@b-.c>\n", 1, 1, RawHtml),
        ("&nbsp;\n", 1, 1, Entity),
        ("- a &#160;\n", 1, 5, Entity),
        ("## &copy; 2026\n", 1, 4, Entity),
        ("[a](/?&copy;)\n", 1, 7, Entity),
    ];
    for (text, line, col, kind) in cases {
        assert_eq!(error(text), Some((line, col, kind)), "{text:?}");
    }
}

#[test]
fn a_link_needs_its_url_right_after_it_and_cannot_hold_a_link() {
    let cases = [
        ("[a](b c)\n", 1, 4, LinkDestination),
        ("[a](/x \"title\")\n", 1, 4, LinkDestination),
        ("[a]()\n", 1, 4, LinkDestination),
        ("[a](</x>)\n", 1, 4, LinkDestination),
        ("text [a](/x\n", 1, 9, LinkDestination),
        ("- a\n  [b](/c\n  d)\n", 2, 6, LinkDestination),
        ("[a [b](/c) d](/e)\n", 1, 1, NestedLink),
        ("x [<https://x.org>](/y)\n", 1, 3, NestedLink),
        ("![a ![b](/c)](/d)\n", 1, 1, NestedLink),
    ];
    for (text, line, col, kind) in cases {
        assert_eq!(error(text), Some((line, col, kind)), "{text:?}");
    }
}

#[test]
fn reference_links_and_their_definitions_are_errors() {
    let cases = [
        ("[a][b]\n", 1, 1, ReferenceLink),
        ("see ![a][] here\n", 1, 5, ReferenceLink),
        ("[x]: /url\n", 1, 1, ReferenceLink),
        ("> [x\n> y]: /url\n", 1, 3, ReferenceLink),
        ("## [x]: y\n", 1, 4, ReferenceLink),
    ];
    for (text, line, col, kind) in cases {
        assert_eq!(error(text), Some((line, col, kind)), "{text:?}");
    }
    // Elsewhere, `[label]:` is text, as in CommonMark.
    let expected = "<p>![y]: c</p>\n<p>a [x]: b</p>\n";
    assert_eq!(html("![y]: c\n\na [x]: b\n"), expected);
}

#[test]
fn emphasis_pairs_marks_of_the_same_kind_and_length() {
    let text = "*a* _b_ **c** __d__ *a **b** c* **a *b* c** _a __b__ c_\n";
    let expected = concat!(
        "<p><em>a</em> <em>b</em> <strong>c</strong> <strong>d</strong> ",
        "<em>a <strong>b</strong> c</em> <strong>a <em>b</em> c</strong> ",
        "<em>a <strong>b</strong> c</em></p>\n",
    );
    assert_eq!(html(text), expected);
    // `*` works inside a word and `_` does not; with spaces around them, both are text.
    let text = "2*3*4 a**b**c snake_case a__b__c x * y x _ y a*é*b\n";
    let expected = concat!(
        "<p>2<em>3</em>4 a<strong>b</strong>c snake_case a__b__c x * y x _ y ",
        "a<em>é</em>b</p>\n",
    );
    assert_eq!(html(text), expected);
}

#[test]
fn emphasis_holds_and_wraps_links_code_and_lines() {
    let text = "*[Registration](/r) Paper:* [**PROMOTIC | Group**](/p) *`code`* **Email:**\n";
    let expected = concat!(
        "<p><em><a href=\"/r\">Registration</a> Paper:</em> ",
        "<a href=\"/p\"><strong>PROMOTIC | Group</strong></a> <em><code>code</code></em> ",
        "<strong>Email:</strong></p>\n",
    );
    assert_eq!(html(text), expected);
    let expected = "<p><em>A Case Study on\n<code>rebalance</code></em> — Crichton</p>\n";
    assert_eq!(
        html("*A Case Study on\n`rebalance`* — Crichton\n"),
        expected
    );
    let text = "_“Adopting Rust.”_ *“quoted”* \\*a\\* \\_b\\_ *a\\*b*\n";
    let expected = "<p><em>“Adopting Rust.”</em> <em>“quoted”</em> *a* _b_ <em>a*b</em></p>\n";
    assert_eq!(html(text), expected);
    let expected = concat!(
        "<h2 id=\"head-bold\"><em>Head</em> <strong>bold</strong></h2>\n",
        "<p><img src=\"/i.png\" alt=\"a b\" /> <a href=\"/x\"><em>a</em></a> ",
        "<em><a href=\"/x\">a</a></em></p>\n",
    );
    assert_eq!(
        html("## *Head* **bold**\n![*a* **b**](/i.png) [*a*](/x) *[a](/x)*\n"),
        expected
    );
}

#[test]
fn emphasis_that_commonmark_resolves_by_subtler_rules_is_an_error() {
    let cases = [
        ("***a***\n", 1, 1, EmphasisRun),
        ("**a *b***\n", 1, 7, EmphasisRun),
        ("a ___b___\n", 1, 3, EmphasisRun),
        ("*a\n", 1, 1, UnmatchedEmphasis),
        ("a*\n", 1, 2, UnmatchedEmphasis),
        ("*a**\n", 1, 3, UnmatchedEmphasis),
        ("**a*\n", 1, 4, UnmatchedEmphasis),
        ("*a _b* c_\n", 1, 4, UnmatchedEmphasis),
        ("*[a*](/x)\n", 1, 4, UnmatchedEmphasis),
        ("*a.*b\n", 1, 1, UnmatchedEmphasis),
        // CommonMark counts `“` as punctuation, from tables std does not have.
        ("a*“b”*c\n", 1, 2, EmphasisNextTo('“')),
        ("x_“y”_z\n", 1, 2, EmphasisNextTo('“')),
    ];
    for (text, line, col, kind) in cases {
        assert_eq!(error(text), Some((line, col, kind)), "{text:?}");
    }
}

#[test]
fn strikethrough_footnotes_and_bare_urls_are_errors() {
    let cases = [
        ("~~a~~\n", 1, 1, Strikethrough),
        ("a~b\n", 1, 2, Strikethrough),
        ("~10 minutes\n", 1, 1, Strikethrough),
        ("[^1]\n", 1, 1, Footnote),
        ("a [^note]\n", 1, 3, Footnote),
        ("see https://x.org\n", 1, 5, BareUrl),
        ("http://x.org\n", 1, 1, BareUrl),
        ("www.x.org\n", 1, 1, BareUrl),
        ("mail a@b.org now\n", 1, 6, BareUrl),
        ("*https://x.org*\n", 1, 2, BareUrl),
    ];
    for (text, line, col, kind) in cases {
        assert_eq!(error(text), Some((line, col, kind)), "{text:?}");
    }
    // GitHub does not link these either.
    let text = "a ~ b, [https://x.org](https://x.org) `www.x.org` a@b a@b. @x.org xhttps://y ~\n";
    let expected = concat!(
        "<p>a ~ b, <a rel=\"external\" href=\"https://x.org\">https://x.org</a> ",
        "<code>www.x.org</code> a@b a@b. @x.org xhttps://y ~</p>\n",
    );
    assert_eq!(html(text), expected);
}

/// Every inline of the subset, for the random test to damage. The spaces of a hard line break are
/// written before `\n`, as editors may remove spaces at the end of a line.
const SAMPLE: &str = "\
## The `edu` [crate](/crates/) &amp; <https://rust-edu.org>

Text with `code` and ``more `code` ``, a [**link**](https://example.org/a_(b)?c=1&d=2), an
![image](/i.png), <https://example.org> or <a@b.c>, \\*escapes\\*, a line break  \n\
and _another_\\
end, as 1 < 2 & 3 > 2 ~ 4.

- [![badge](/b.png)](/x) *`item`*  \n  **next**
";

#[test]
fn a_damaged_text_gives_html_or_an_error_never_a_panic() {
    to_html(SAMPLE).unwrap();
    let sample: Vec<char> = SAMPLE.chars().collect();
    let damage = [
        '*', '_', '~', '^', '[', ']', '(', ')', '!', '<', '>', '`', '\\', '&', ';', ':', '@', '/',
        ' ', '\n', 'é', '“',
    ];
    // xorshift64 with a fixed seed: every run tries the same 5,000 texts, each with two random
    // characters replaced, inserted or removed.
    let mut state: u64 = 0x2545_F491_4F6C_DD1D;
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
