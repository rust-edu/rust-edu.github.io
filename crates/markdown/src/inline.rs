//! The inline syntax of paragraphs and headings, rendered as Zola renders it: code spans,
//! emphasis, links, images, autolinks, backslash escapes, the character references `&amp;`,
//! `&lt;`, `&gt;` and `&quot;`, and hard line breaks.

use crate::{ErrorKind, escape, escape_attribute};

/// A piece of the text of a paragraph or heading.
pub(crate) enum Inline<'a> {
    /// Text, escaped when written.
    Text(&'a str),
    /// The text of a code span, with its line breaks made spaces.
    Code(String),
    /// `*text*` or `_text_`, or with `strong` `**text**` or `__text__`.
    Emphasis { strong: bool, text: Vec<Inline<'a>> },
    /// A run of `*` or `_` that may start or end emphasis, until [`emphasis`] pairs it.
    Delimiter(Delimiter),
    /// A link: where it points, and its text.
    Link { url: String, text: Vec<Inline<'a>> },
    /// An image: where it is, and its description.
    Image { url: String, text: Vec<Inline<'a>> },
    /// The end of a line inside the text.
    LineBreak,
    /// A line break kept in the HTML, after two spaces or more, or a `\`.
    HardBreak,
}

/// A run of one or two `*` or `_` that may open or close emphasis.
#[derive(Clone, Copy)]
pub(crate) struct Delimiter {
    /// Where it is in the text.
    at: usize,
    mark: char,
    len: usize,
    open: bool,
    close: bool,
}

impl Delimiter {
    /// The run as written, for a delimiter left unpaired.
    fn run(self) -> String {
        self.mark.to_string().repeat(self.len)
    }
}

/// A `[`, or a `![`, that a later `]` may close.
struct Bracket {
    /// Where it is in the text.
    at: usize,
    /// The node that holds it as text until it is closed.
    node: usize,
    image: bool,
    /// Whether it may still start a link or image. A link makes the `[`s around it inactive, as
    /// in CommonMark; an image makes the `![`s around it inactive, as Zola fails on an image in an
    /// image.
    active: bool,
}

/// The parser of one text.
struct Inlines<'a> {
    text: &'a str,
    /// The first byte not parsed yet.
    next: usize,
    /// The first byte of the text not in a node yet.
    start: usize,
    nodes: Vec<Inline<'a>>,
    brackets: Vec<Bracket>,
    /// Where text outside links and images has a URL that GitHub links, so far.
    bare: Vec<usize>,
}

/// Parses `text`: the lines of a paragraph joined by `\n`, without the spaces that end it, or the
/// text of a heading. An error has the byte of `text` where it starts.
pub(crate) fn parse(text: &str) -> Result<Vec<Inline<'_>>, (usize, ErrorKind)> {
    let (next, start, nodes, brackets, bare) = (0, 0, Vec::new(), Vec::new(), Vec::new());
    let mut inlines = Inlines {
        text,
        next,
        start,
        nodes,
        brackets,
        bare,
    };
    while let Some(&byte) = text.as_bytes().get(inlines.next) {
        match byte {
            b'\\' => inlines.escape(),
            b'`' => inlines.code(),
            b'<' => inlines.angle()?,
            b'&' => inlines.reference()?,
            b'*' | b'_' => inlines.delimiter()?,
            b'~' => inlines.tilde()?,
            b'[' if inlines.rest().starts_with("[^") => {
                return Err((inlines.next, ErrorKind::Footnote));
            }
            b'[' => inlines.open(false),
            b'!' if inlines.rest().starts_with("![") => inlines.open(true),
            b']' => inlines.close()?,
            b'\n' => inlines.line_break(),
            _ => inlines.next += 1,
        }
    }
    inlines.flush();
    if let Some(&at) = inlines.bare.first() {
        return Err((at, ErrorKind::BareUrl));
    }
    emphasis(&mut inlines.nodes)?;
    Ok(inlines.nodes)
}

impl<'a> Inlines<'a> {
    /// The text from the next byte on.
    fn rest(&self) -> &'a str {
        self.text.get(self.next..).unwrap_or_default()
    }

    /// The character before the next byte.
    fn before(&self) -> Option<char> {
        self.text.get(..self.next)?.chars().next_back()
    }

    /// Ends the text before the next byte as a node.
    fn flush(&mut self) {
        let text = self.text.get(self.start..self.next).unwrap_or_default();
        if !text.is_empty() {
            if let Some(index) = bare_url(text) {
                self.bare.push(self.start + index);
            }
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
            self.deactivate(false);
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

    /// A run of `*` or `_`: a delimiter if it may open or close emphasis, else text.
    fn delimiter(&mut self) -> Result<(), (usize, ErrorKind)> {
        let (at, rest) = (self.next, self.rest());
        let mark = if rest.starts_with('*') { '*' } else { '_' };
        let len = rest.len() - rest.trim_start_matches(mark).len();
        let before = self.before();
        let after = rest.get(len..).and_then(|t| t.chars().next());
        let Some((open, close)) = can_open_close(mark, before, after) else {
            let mut neighbors = [before, after].into_iter().flatten();
            let c = neighbors.find(|&c| class(Some(c)).is_none());
            return Err((at, ErrorKind::EmphasisNextTo(c.unwrap_or(mark))));
        };
        if !open && !close {
            self.next += len;
        } else if len > 2 {
            return Err((at, ErrorKind::EmphasisRun));
        } else {
            let delimiter = Delimiter {
                at,
                mark,
                len,
                open,
                close,
            };
            self.push(Inline::Delimiter(delimiter), len);
        }
        Ok(())
    }

    /// A run of `~`, which may start or end strikethrough on GitHub and in Zola: text only with
    /// space on both sides, where `*` could neither open nor close emphasis.
    fn tilde(&mut self) -> Result<(), (usize, ErrorKind)> {
        let rest = self.rest();
        let len = rest.len() - rest.trim_start_matches('~').len();
        let after = rest.get(len..).and_then(|t| t.chars().next());
        let is_space = |c: Option<char>| c.is_none_or(char::is_whitespace);
        if !is_space(self.before()) || !is_space(after) {
            return Err((self.next, ErrorKind::Strikethrough));
        }
        self.next += len;
        Ok(())
    }

    /// A `[`, or with `image` a `![`, kept as text unless a `]` closes it.
    fn open(&mut self, image: bool) {
        let (at, len) = (self.next, if image { 2 } else { 1 });
        let marker = self.text.get(at..at + len).unwrap_or_default();
        self.push(Inline::Text(marker), len);
        let node = self.nodes.len() - 1;
        let active = true;
        self.brackets.push(Bracket {
            at,
            node,
            image,
            active,
        });
    }

    /// A `]`: with `(` after it, the end of the text of the link or image that the last bracket
    /// starts; else text.
    fn close(&mut self) -> Result<(), (usize, ErrorKind)> {
        let after = self.rest().get(1..).unwrap_or_default();
        let Some(bracket) = self.brackets.pop() else {
            self.next += 1;
            return Ok(());
        };
        // CommonMark may read `[label]:` at the start of a paragraph as a link's definition.
        let definition = bracket.at == 0 && !bracket.image && after.starts_with(':');
        if after.starts_with('[') || definition {
            return Err((bracket.at, ErrorKind::ReferenceLink));
        }
        if !after.starts_with('(') {
            self.next += 1;
            return Ok(());
        }
        if !bracket.active {
            return Err((bracket.at, ErrorKind::NestedLink));
        }
        let open = self.next + 1;
        let error = (open, ErrorKind::LinkDestination);
        let destination = after.get(1..).and_then(destination).ok_or(error)?;
        let url = decode(destination).map_err(|index| (open + 1 + index, ErrorKind::Entity))?;
        self.flush();
        // GitHub does not link a URL in a link's text or an image's description.
        self.bare.retain(|&at| at < bracket.at);
        let text = self.nodes.split_off(bracket.node + 1);
        self.nodes.truncate(bracket.node);
        self.deactivate(bracket.image);
        self.nodes.push(if bracket.image {
            Inline::Image { url, text }
        } else {
            Inline::Link { url, text }
        });
        self.next = open + destination.len() + 2;
        self.start = self.next;
        Ok(())
    }

    /// Makes the brackets around a new link, or around a new image, unable to start another.
    fn deactivate(&mut self, image: bool) {
        for bracket in &mut self.brackets {
            if bracket.image == image {
                bracket.active = false;
            }
        }
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

/// How CommonMark's rules for emphasis count a character next to a run of `*` or `_`.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Class {
    Space,
    Punctuation,
    Other,
}

/// The class of `c`, the start or end of the text counting as space; `None` for a character
/// outside ASCII that is not a letter, digit or space, which Unicode's tables may count as
/// punctuation or not.
fn class(c: Option<char>) -> Option<Class> {
    match c {
        None => Some(Class::Space),
        Some(c) if c.is_whitespace() => Some(Class::Space),
        Some(c) if c.is_ascii_punctuation() => Some(Class::Punctuation),
        Some(c) if c.is_ascii() || c.is_alphanumeric() => Some(Class::Other),
        Some(_) => None,
    }
}

/// Whether a run of `mark` between `before` and `after` may open and may close emphasis, by
/// rules 1 to 8 of CommonMark; `None` if that depends on a character [`class`] cannot tell.
fn can_open_close(mark: char, before: Option<char>, after: Option<char>) -> Option<(bool, bool)> {
    let rules = |before: Class, after: Class| {
        let left = after != Class::Space && (after != Class::Punctuation || before != Class::Other);
        let right =
            before != Class::Space && (before != Class::Punctuation || after != Class::Other);
        if mark == '*' {
            (left, right)
        } else {
            // `_` opens or closes inside a word only next to punctuation.
            let open = left && (!right || before == Class::Punctuation);
            (open, right && (!left || after == Class::Punctuation))
        }
    };
    // A character that cannot be classed is tried as punctuation and as a letter.
    let tries = |c| class(c).map_or([Class::Punctuation, Class::Other], |class| [class; 2]);
    let results = tries(before).map(|before| tries(after).map(|after| rules(before, after)));
    let results = results.as_flattened();
    let first = *results.first()?;
    results
        .iter()
        .all(|&result| result == first)
        .then_some(first)
}

/// Pairs the delimiters of `inlines`, and of the links and images in them, into emphasis: each
/// closer with the nearest opener of the same mark, which must be as long and have no unpaired
/// opener after it. Any other delimiter is an error.
fn emphasis(inlines: &mut Vec<Inline<'_>>) -> Result<(), (usize, ErrorKind)> {
    for inline in inlines.iter_mut() {
        if let Inline::Link { text, .. } | Inline::Image { text, .. } = inline {
            emphasis(text)?;
        }
    }
    let mut openers: Vec<(usize, Delimiter)> = Vec::new();
    let mut index = 0;
    while let Some(inline) = inlines.get(index) {
        let &Inline::Delimiter(delimiter) = inline else {
            index += 1;
            continue;
        };
        let same_mark = |&(_, opener): &(usize, Delimiter)| opener.mark == delimiter.mark;
        let found = openers.iter().rposition(same_mark);
        match found.zip(found.and_then(|found| openers.get(found))) {
            Some((found, &(start, opener))) if delimiter.close => {
                if let Some((_, between)) = openers.get(found + 1) {
                    return Err((between.at, ErrorKind::UnmatchedEmphasis));
                }
                if opener.len != delimiter.len {
                    return Err((delimiter.at, ErrorKind::UnmatchedEmphasis));
                }
                openers.truncate(found);
                let text: Vec<_> = inlines.drain(start + 1..index).collect();
                inlines.remove(start + 1);
                let strong = delimiter.len == 2;
                if let Some(inline) = inlines.get_mut(start) {
                    *inline = Inline::Emphasis { strong, text };
                }
                index = start + 1;
            }
            _ if delimiter.open => {
                openers.push((index, delimiter));
                index += 1;
            }
            _ => return Err((delimiter.at, ErrorKind::UnmatchedEmphasis)),
        }
    }
    match openers.first() {
        Some((_, opener)) => Err((opener.at, ErrorKind::UnmatchedEmphasis)),
        None => Ok(()),
    }
}

/// Where `text` has a URL or an email address that GitHub links and CommonMark leaves as text:
/// `https://`, `http://` or `www.` not after a letter or digit, or `name@domain.tld`.
fn bare_url(text: &str) -> Option<usize> {
    let after_word = |index: usize| {
        let before = text.get(..index).and_then(|t| t.chars().next_back());
        before.is_some_and(char::is_alphanumeric)
    };
    let urls = ["https://", "http://", "www."].into_iter();
    let url = urls.flat_map(|prefix| text.match_indices(prefix).map(|(index, _)| index));
    let url = url.filter(|&index| !after_word(index)).min();
    let email = text.match_indices('@').find_map(|(at, _)| {
        let is_user = |c: char| c.is_ascii_alphanumeric() || ".+-_".contains(c);
        let user = text.get(..at)?.rsplit(|c| !is_user(c)).next()?;
        let domain = text
            .get(at + 1..)?
            .split(|c| !is_user(c) || c == '+')
            .next()?;
        let dotted = domain.trim_end_matches('.').contains('.');
        (!user.is_empty() && dotted).then(|| at - user.len())
    });
    url.into_iter().chain(email).min()
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

/// The destination that starts `text`, after a link's `(`: the text up to the `)` that balances
/// the `(`s in it, if it is not empty, does not start with `<` and has no space or line break.
fn destination(text: &str) -> Option<&str> {
    let mut depth = 0_usize;
    let mut chars = text.char_indices().peekable();
    while let Some((index, c)) = chars.next() {
        match c {
            // An escaped punctuation character neither opens nor closes.
            '\\' => _ = chars.next_if(|&(_, c)| c.is_ascii_punctuation()),
            '(' => depth += 1,
            ')' if depth == 0 => return text.get(..index).filter(|url| !url.is_empty()),
            ')' => depth -= 1,
            '<' if index == 0 => return None,
            ' ' => return None,
            c if c.is_control() => return None,
            _ => {}
        }
    }
    None
}

/// A link's destination with its backslash escapes and character references replaced, or the
/// byte where a reference other than the four starts.
fn decode(url: &str) -> Result<String, usize> {
    let mut decoded = String::new();
    let mut index = 0;
    while let Some(rest) = url.get(index..).filter(|rest| !rest.is_empty()) {
        let escaped = rest.strip_prefix('\\').and_then(|after| after.get(..1));
        let (text, len) =
            if let Some(c) = escaped.filter(|c| c.bytes().all(|b| b.is_ascii_punctuation())) {
                (c, 2)
            } else if let Some((len, text)) = reference(rest) {
                (text.ok_or(index)?, len)
            } else {
                let len = rest.chars().next().map_or(1, char::len_utf8);
                (rest.get(..len).unwrap_or_default(), len)
            };
        decoded.push_str(text);
        index += len;
    }
    Ok(decoded)
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
            Inline::Emphasis { strong, text } => {
                let tag = if *strong { "strong" } else { "em" };
                html.push_str(&format!("<{tag}>"));
                write(html, text);
                html.push_str(&format!("</{tag}>"));
            }
            Inline::Delimiter(delimiter) => html.push_str(&delimiter.run()),
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
            Inline::Image { url, text } => {
                html.push_str("<img src=\"");
                escape_url(html, url);
                html.push_str("\" alt=\"");
                escape_attribute(html, &plain(text));
                html.push_str("\" />");
            }
            Inline::LineBreak => html.push('\n'),
            Inline::HardBreak => html.push_str("<br />\n"),
        }
    }
}

/// The text of `inlines` without markup, from which Zola makes a heading's id and an image's
/// description: line breaks and images add nothing.
pub(crate) fn plain(inlines: &[Inline<'_>]) -> String {
    let mut plain = String::new();
    for inline in inlines {
        match inline {
            Inline::Text(text) => plain.push_str(text),
            Inline::Code(code) => plain.push_str(code),
            Inline::Emphasis { text, .. } | Inline::Link { text, .. } => {
                plain.push_str(&self::plain(text));
            }
            Inline::Delimiter(delimiter) => plain.push_str(&delimiter.run()),
            Inline::Image { .. } | Inline::LineBreak | Inline::HardBreak => {}
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
