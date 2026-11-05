//! The inline syntax of paragraphs and headings, rendered as Zola renders it: code spans,
//! autolinks, backslash escapes, the character references `&amp;`, `&lt;`, `&gt;` and `&quot;`,
//! and hard line breaks. Links, images and emphasis are text.

use crate::{ErrorKind, escape};

/// A piece of the text of a paragraph or heading.
pub(crate) enum Inline<'a> {
    /// Text, escaped when written.
    Text(&'a str),
    /// The text of a code span, with its line breaks made spaces.
    Code(String),
    /// A link: where it points, and its text.
    Link { url: String, text: Vec<Inline<'a>> },
    /// The end of a line inside the text.
    LineBreak,
    /// A line break kept in the HTML, after two spaces or more, or a `\`.
    HardBreak,
}

/// The parser of one text.
struct Inlines<'a> {
    text: &'a str,
    /// The first byte not parsed yet.
    next: usize,
    /// The first byte of the text not in a node yet.
    start: usize,
    nodes: Vec<Inline<'a>>,
}

/// Parses `text`: the lines of a paragraph joined by `\n`, without the spaces that end it, or the
/// text of a heading. An error has the byte of `text` where it starts.
pub(crate) fn parse(text: &str) -> Result<Vec<Inline<'_>>, (usize, ErrorKind)> {
    let (next, start, nodes) = (0, 0, Vec::new());
    let mut inlines = Inlines {
        text,
        next,
        start,
        nodes,
    };
    while let Some(&byte) = text.as_bytes().get(inlines.next) {
        match byte {
            b'\\' => inlines.escape(),
            b'`' => inlines.code(),
            b'<' => inlines.angle()?,
            b'&' => inlines.reference()?,
            b'\n' => inlines.line_break(),
            _ => inlines.next += 1,
        }
    }
    inlines.flush();
    Ok(inlines.nodes)
}

impl<'a> Inlines<'a> {
    /// The text from the next byte on.
    fn rest(&self) -> &'a str {
        self.text.get(self.next..).unwrap_or_default()
    }

    /// Ends the text before the next byte as a node.
    fn flush(&mut self) {
        let text = self.text.get(self.start..self.next).unwrap_or_default();
        if !text.is_empty() {
            self.nodes.push(Inline::Text(text));
        }
        self.start = self.next;
    }

    /// Adds `node`, made of the `len` bytes from the next one.
    fn push(&mut self, node: Inline<'a>, len: usize) {
        self.flush();
        self.nodes.push(node);
        self.next += len;
        self.start = self.next;
    }

    /// A `\`: before ASCII punctuation, that character as text; before a line break, a hard one;
    /// else text.
    fn escape(&mut self) {
        match self.rest().as_bytes().get(1) {
            Some(b'\n') => self.push(Inline::HardBreak, 2),
            Some(byte) if byte.is_ascii_punctuation() => {
                self.flush();
                // The text goes on from the escaped character.
                self.start = self.next + 1;
                self.next += 2;
            }
            _ => self.next += 1,
        }
    }

    /// A code span: a run of backticks, and the text up to a run of as many. A run without its
    /// match is text.
    fn code(&mut self) {
        let rest = self.rest();
        let len = backticks(rest);
        let mut search = len;
        while let Some(found) = rest.get(search..).and_then(|after| after.find('`')) {
            let close = search + found;
            let close_len = backticks(rest.get(close..).unwrap_or_default());
            if close_len == len {
                let code = rest.get(len..close).unwrap_or_default().replace('\n', " ");
                // One space goes from each end when both have one, unless all are spaces.
                let inner = code
                    .strip_prefix(' ')
                    .and_then(|code| code.strip_suffix(' '));
                let inner = inner.filter(|inner| inner.contains(|c| c != ' '));
                let code = inner.map(str::to_owned).unwrap_or(code);
                self.push(Inline::Code(code), close + len);
                return;
            }
            search = close + close_len;
        }
        self.next += len;
    }

    /// A `<`: an autolink to a URL or an email address, else HTML before a letter, `/`, `!` or
    /// `?`, else text.
    fn angle(&mut self) -> Result<(), (usize, ErrorKind)> {
        let inside = self.rest().get(1..).unwrap_or_default();
        let target = inside.split_once('>').map_or("", |(target, _)| target);
        let url = if is_url(target) {
            Some(target.to_owned())
        } else if is_email(target) {
            Some(format!("mailto:{target}"))
        } else {
            None
        };
        if let Some(url) = url {
            let text = vec![Inline::Text(target)];
            self.push(Inline::Link { url, text }, target.len() + 2);
        } else if inside.starts_with(|c: char| c.is_ascii_alphabetic() || "/!?".contains(c)) {
            return Err((self.next, ErrorKind::RawHtml));
        } else {
            self.next += 1;
        }
        Ok(())
    }

    /// A `&`: one of the four character references, an error for another one, else text.
    fn reference(&mut self) -> Result<(), (usize, ErrorKind)> {
        match reference(self.rest()) {
            Some((len, Some(text))) => self.push(Inline::Text(text), len),
            Some((_, None)) => return Err((self.next, ErrorKind::Entity)),
            None => self.next += 1,
        }
        Ok(())
    }

    /// A `\n`, without the spaces before it: a hard line break after two of them or more.
    fn line_break(&mut self) {
        let before = self.text.get(self.start..self.next).unwrap_or_default();
        let spaces = before.len() - before.trim_end_matches(' ').len();
        self.next -= spaces;
        let node = if spaces >= 2 {
            Inline::HardBreak
        } else {
            Inline::LineBreak
        };
        self.push(node, spaces + 1);
    }
}

/// How many backticks start `text`.
fn backticks(text: &str) -> usize {
    text.len() - text.trim_start_matches('`').len()
}

/// Whether `target`, between `<` and `>`, is a URL: a scheme of 2 to 32 letters, digits, `+`, `.`
/// or `-` that starts with a letter, a `:`, and no space, control character or `<`.
fn is_url(target: &str) -> bool {
    let Some((scheme, rest)) = target.split_once(':') else {
        return false;
    };
    (2..=32).contains(&scheme.len())
        && scheme.starts_with(|c: char| c.is_ascii_alphabetic())
        && scheme
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "+.-".contains(c))
        && !rest.contains(|c: char| c == ' ' || c == '<' || c.is_control())
}

/// Whether `target`, between `<` and `>`, is an email address as CommonMark defines it.
fn is_email(target: &str) -> bool {
    let Some((user, domain)) = target.split_once('@') else {
        return false;
    };
    let is_user = |c: char| c.is_ascii_alphanumeric() || ".!#$%&'*+/=?^_`{|}~-".contains(c);
    let is_label = |label: &str| {
        (1..=63).contains(&label.len())
            && !label.starts_with('-')
            && !label.ends_with('-')
            && label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
    };
    !user.is_empty() && user.chars().all(is_user) && domain.split('.').all(is_label)
}

/// The length of the character reference that starts `text` (`&`, a name or `#` and a number,
/// and `;`), and what it stands for if it is `&amp;`, `&lt;`, `&gt;` or `&quot;`.
fn reference(text: &str) -> Option<(usize, Option<&'static str>)> {
    let (body, _) = text.strip_prefix('&')?.split_once(';')?;
    let name = body.strip_prefix('#').unwrap_or(body);
    if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric()) {
        return None;
    }
    let decoded = match body {
        "amp" => Some("&"),
        "lt" => Some("<"),
        "gt" => Some(">"),
        "quot" => Some("\""),
        _ => None,
    };
    Some((body.len() + 2, decoded))
}

/// Appends `inlines` to `html`.
pub(crate) fn write(html: &mut String, inlines: &[Inline<'_>]) {
    for inline in inlines {
        match inline {
            Inline::Text(text) => escape(html, text),
            Inline::Code(code) => {
                html.push_str("<code>");
                escape(html, code);
                html.push_str("</code>");
            }
            Inline::Link { url, text } => {
                // Zola marks links to `http` and `https` URLs, its own site's too, as external.
                if url.starts_with("http:") || url.starts_with("https:") {
                    html.push_str("<a rel=\"external\" href=\"");
                } else {
                    html.push_str("<a href=\"");
                }
                escape_url(html, url);
                html.push_str("\">");
                write(html, text);
                html.push_str("</a>");
            }
            Inline::LineBreak => html.push('\n'),
            Inline::HardBreak => html.push_str("<br />\n"),
        }
    }
}

/// The text of `inlines` without markup, from which Zola makes a heading's id: line breaks add
/// nothing.
pub(crate) fn plain(inlines: &[Inline<'_>]) -> String {
    let mut plain = String::new();
    for inline in inlines {
        match inline {
            Inline::Text(text) => plain.push_str(text),
            Inline::Code(code) => plain.push_str(code),
            Inline::Link { text, .. } => plain.push_str(&self::plain(text)),
            Inline::LineBreak | Inline::HardBreak => {}
        }
    }
    plain
}

/// Appends `url` to `html` as Zola writes a link's destination: `&` and `'` as references, and
/// the bytes of other characters outside its safe set as `%` and two hex digits.
fn escape_url(html: &mut String, url: &str) {
    for c in url.chars() {
        match c {
            '&' => html.push_str("&amp;"),
            '\'' => html.push_str("&#x27;"),
            c if c.is_ascii_alphanumeric() || "!#$%()*+,-./:;=?@^_~".contains(c) => html.push(c),
            c => {
                let mut bytes = [0; 4];
                for byte in c.encode_utf8(&mut bytes).bytes() {
                    html.push_str(&format!("%{byte:02X}"));
                }
            }
        }
    }
}
