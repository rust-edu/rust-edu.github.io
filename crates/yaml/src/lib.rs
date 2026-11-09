//! A strict subset of YAML 1.2 for the headers of Markdown pages and for site settings.
//!
//! Only block style: `key: value` maps, a map or a list indented under `key:`, `- item` lists, and
//! lists of maps, whose keys line up after the `- `. A scalar is plain, `'single'` or `"double"`
//! quoted, and always a string: its consumer decides whether `2027-03-01` is a date and `true` a
//! boolean, so `no` never turns into `false`. The rest of YAML (tabs, anchors, aliases, tags,
//! flow collections, block scalars, values over several lines) is an error with a line and column,
//! so a YAML 1.2 parser reads whatever this crate accepts as the same maps, lists and text.

use std::fmt;

/// A line and a column, both counted from 1; columns count characters, not bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pos {
    pub line: usize,
    pub col: usize,
}

/// A value and the position of its first character.
#[derive(Debug, PartialEq, Eq)]
pub struct Node {
    pub pos: Pos,
    pub value: Value,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Value {
    /// A scalar's text, without its quotes and with its escapes resolved.
    Str(String),
    List(Vec<Node>),
    /// Entries in the order of the text; no two have the same key.
    Map(Vec<Entry>),
}

/// A key, the position where it starts, and its value.
#[derive(Debug, PartialEq, Eq)]
pub struct Entry {
    pub key: String,
    pub pos: Pos,
    pub value: Node,
}

/// Where and why a text is not in the subset.
#[derive(Debug, PartialEq, Eq)]
pub struct Error {
    pub pos: Pos,
    pub kind: ErrorKind,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ErrorKind {
    MissingHeader,
    UnclosedHeader,
    ControlCharacter(char),
    /// One of Unicode's 66 noncharacters, which HTML reports; XML refuses U+FFFE and U+FFFF.
    Noncharacter(char),
    /// A line indented deeper than its block but not under a `key:`, such as a value's second line.
    BadIndent,
    ExpectedKey,
    ExpectedListItem,
    /// A `- ` or `key:` with no value after it, and for a key, no lines indented under it.
    MissingValue,
    DuplicateKey {
        key: String,
        first_line: usize,
    },
    UnclosedQuote,
    InvalidEscape,
    TextAfterQuote,
    /// A plain value with `: ` in it or `:` at its end, which YAML reads as a key.
    ColonInValue,
    /// YAML outside the subset, by the character that starts it: `&` anchors, `*` aliases, `!`
    /// tags, `[` and `{` flow collections, `|` and `>` block scalars, `?` complex keys, `-` a list
    /// on the line of its key, or a character YAML reserves (`%`, `@`, `` ` ``, `,`, `:`, `]`, `}`).
    Indicator(char),
}

/// Where a page starts, and where its header errors point when no line is to blame.
const TOP: Pos = Pos { line: 1, col: 1 };

/// Parses a text whose top level is a map, such as a settings file.
pub fn parse(text: &str) -> Result<Vec<Entry>, Error> {
    parse_lines(without_bom(text).lines().zip(1..))
}

/// Splits a Markdown page into its header and the text after it. The header is the map between
/// a first line `---` and the next line `---`; its positions count lines from the top of the page.
pub fn parse_front_matter(text: &str) -> Result<(Vec<Entry>, &str), Error> {
    let (first, mut rest) = split_line(without_bom(text));
    if first != "---" {
        let kind = ErrorKind::MissingHeader;
        return Err(Error { pos: TOP, kind });
    }
    let mut header = Vec::new();
    while !rest.is_empty() {
        let (line, after) = split_line(rest);
        if line == "---" {
            return Ok((parse_lines(header)?, after));
        }
        // Line 1 is the opening `---`.
        header.push((line, header.len() + 2));
        rest = after;
    }
    let kind = ErrorKind::UnclosedHeader;
    Err(Error { pos: TOP, kind })
}

/// `text` without the byte order mark, U+FEFF, that some editors write at the start of a UTF-8
/// file, and that would hide its first `---` or key.
fn without_bom(text: &str) -> &str {
    text.strip_prefix('\u{feff}').unwrap_or(text)
}

/// The first line of `text` without its `\n` or `\r\n`, and the text after that line.
fn split_line(text: &str) -> (&str, &str) {
    let (line, rest) = text.split_once('\n').unwrap_or((text, ""));
    (line.strip_suffix('\r').unwrap_or(line), rest)
}

/// Parses lines, each with its line number, as a top-level map.
fn parse_lines<'a>(
    numbered: impl IntoIterator<Item = (&'a str, usize)>,
) -> Result<Vec<Entry>, Error> {
    let mut lines = Vec::new();
    for (raw, number) in numbered {
        let found = raw
            .chars()
            .enumerate()
            .find_map(|(index, c)| Some((index, refused(c)?)));
        if let Some((index, kind)) = found {
            let (line, col) = (number, index + 1);
            let pos = Pos { line, col };
            return Err(Error { pos, kind });
        }
        let text = raw.trim_end_matches(' ');
        let content = text.trim_start_matches(' ');
        // Blank lines and comment lines carry nothing.
        if !content.is_empty() && !content.starts_with('#') {
            lines.push(Line {
                number,
                text,
                content,
            });
        }
    }
    Parser { lines, next: 0 }.map(0)
}

/// The error for `c` if a line cannot hold it: a control character, or a noncharacter, which is
/// U+FDD0 to U+FDEF or one whose last 16 bits are FFFE or FFFF.
fn refused(c: char) -> Option<ErrorKind> {
    let code = u32::from(c);
    if c.is_control() {
        Some(ErrorKind::ControlCharacter(c))
    } else if (0xFDD0..=0xFDEF).contains(&code) || code & 0xFFFE == 0xFFFE {
        Some(ErrorKind::Noncharacter(c))
    } else {
        None
    }
}

#[derive(Clone, Copy)]
struct Line<'a> {
    number: usize,
    /// The whole line without trailing spaces; columns count from its start.
    text: &'a str,
    /// The end of `text` left to parse: after the indentation, or after the `- ` of a list item
    /// that holds a map.
    content: &'a str,
}

impl Line<'_> {
    /// The column, counted from 0, where `content` starts. Only spaces and `-` come before it, so
    /// bytes count as columns.
    fn indent(&self) -> usize {
        self.text.len() - self.content.len()
    }

    /// The position of `part`, an end of this line's `text`.
    fn pos(&self, part: &str) -> Pos {
        let line = self.number;
        let col = self.text.chars().count() - part.chars().count() + 1;
        Pos { line, col }
    }

    fn error(&self, part: &str, kind: ErrorKind) -> Error {
        let pos = self.pos(part);
        Error { pos, kind }
    }
}

struct Parser<'a> {
    /// The lines with content; blank lines and comment lines are left out.
    lines: Vec<Line<'a>>,
    /// The first line not parsed yet.
    next: usize,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<Line<'a>> {
        self.lines.get(self.next).copied()
    }

    /// The next line if it belongs to the block whose lines start at column `indent`, or `None`
    /// when the block ends. A deeper line is an error: only the lines after `key:` go deeper.
    fn peek_at(&self, indent: usize) -> Result<Option<Line<'a>>, Error> {
        match self.peek() {
            Some(line) if line.indent() > indent => {
                Err(line.error(line.content, ErrorKind::BadIndent))
            }
            Some(line) => Ok((line.indent() == indent).then_some(line)),
            None => Ok(None),
        }
    }

    fn map(&mut self, indent: usize) -> Result<Vec<Entry>, Error> {
        let mut entries: Vec<Entry> = Vec::new();
        while let Some(line) = self.peek_at(indent)? {
            self.next += 1;
            let Some((key, rest)) = split_key(line.content) else {
                let kind =
                    indicator(line.content).map_or(ErrorKind::ExpectedKey, ErrorKind::Indicator);
                return Err(line.error(line.content, kind));
            };
            let pos = line.pos(line.content);
            if let Some(first) = entries.iter().find(|entry| entry.key == key) {
                let (key, first_line) = (key.to_owned(), first.pos.line);
                let kind = ErrorKind::DuplicateKey { key, first_line };
                return Err(Error { pos, kind });
            }
            let rest = rest.trim_start_matches(' ');
            let value = if rest.is_empty() || rest.starts_with('#') {
                self.block(indent, pos)?
            } else {
                scalar(&line, rest)?
            };
            let key = key.to_owned();
            entries.push(Entry { key, pos, value });
        }
        Ok(entries)
    }

    /// The map or list on the lines indented under the `key:` at `pos`, column `indent`.
    fn block(&mut self, indent: usize, pos: Pos) -> Result<Node, Error> {
        let Some(first) = self.peek().filter(|line| line.indent() > indent) else {
            let kind = ErrorKind::MissingValue;
            return Err(Error { pos, kind });
        };
        let value = if list_item(first.content).is_some() {
            Value::List(self.list(first.indent())?)
        } else {
            Value::Map(self.map(first.indent())?)
        };
        let pos = first.pos(first.content);
        Ok(Node { pos, value })
    }

    fn list(&mut self, indent: usize) -> Result<Vec<Node>, Error> {
        let mut items = Vec::new();
        while let Some(line) = self.peek_at(indent)? {
            let Some(item) = list_item(line.content) else {
                return Err(line.error(line.content, ErrorKind::ExpectedListItem));
            };
            if item.is_empty() || item.starts_with('#') {
                return Err(line.error(line.content, ErrorKind::MissingValue));
            }
            let node = if split_key(item).is_some() {
                // A map: its first key follows the `- `, and its other keys line up under the first.
                if let Some(current) = self.lines.get_mut(self.next) {
                    current.content = item;
                }
                let pos = line.pos(item);
                let value = Value::Map(self.map(pos.col - 1)?);
                Node { pos, value }
            } else {
                self.next += 1;
                scalar(&line, item)?
            };
            items.push(node);
        }
        Ok(items)
    }
}

/// Splits `key: value` or `key:` into the key and what follows the colon.
fn split_key(content: &str) -> Option<(&str, &str)> {
    let (key, rest) = content.split_once(':')?;
    let is_key_char = |c: char| c.is_ascii_alphanumeric() || c == '-' || c == '_';
    let is_key = !key.is_empty() && key.chars().all(is_key_char);
    (is_key && (rest.is_empty() || rest.starts_with(' '))).then_some((key, rest))
}

/// The value after the `- ` of a list item, or `None` if `content` is not a list item.
fn list_item(content: &str) -> Option<&str> {
    let rest = content.strip_prefix('-')?;
    (rest.is_empty() || rest.starts_with(' ')).then_some(rest.trim_start_matches(' '))
}

/// The character that starts `text` if YAML gives it a meaning outside the subset.
fn indicator(text: &str) -> Option<char> {
    let mut chars = text.chars();
    let first = chars.next()?;
    let alone = matches!(chars.next(), None | Some(' '));
    match first {
        '-' | '?' | ':' if alone => Some(first),
        '&' | '*' | '!' | '[' | '{' | '|' | '>' | '%' | '@' | '`' | ',' | ']' | '}' => Some(first),
        _ => None,
    }
}

/// Reads the scalar that runs from `text` to the end of `line`.
fn scalar(line: &Line<'_>, text: &str) -> Result<Node, Error> {
    let value = match text.chars().next() {
        Some(quote @ ('\'' | '"')) => quoted(line, text, quote)?,
        _ => plain(line, text)?,
    };
    let (pos, value) = (line.pos(text), Value::Str(value));
    Ok(Node { pos, value })
}

/// Reads a plain scalar, which ends at the end of the line or at a ` #` comment.
fn plain(line: &Line<'_>, text: &str) -> Result<String, Error> {
    let value = text.split_once(" #").map_or(text, |(value, _)| value);
    let value = value.trim_end_matches(' ');
    if let Some(c) = indicator(value) {
        return Err(line.error(text, ErrorKind::Indicator(c)));
    }
    if value.contains(": ") || value.ends_with(':') {
        return Err(line.error(text, ErrorKind::ColonInValue));
    }
    Ok(value.to_owned())
}

/// Reads a quoted scalar, then an optional comment. In single quotes `''` stands for `'`; in
/// double quotes `\"`, `\\`, `\n` and `\t` are the escapes.
fn quoted(line: &Line<'_>, text: &str, quote: char) -> Result<String, Error> {
    let mut value = String::new();
    let mut chars = text.chars();
    chars.next(); // the opening quote
    loop {
        let here = chars.as_str();
        match chars.next() {
            None => return Err(line.error(text, ErrorKind::UnclosedQuote)),
            Some('\'') if quote == '\'' && chars.as_str().starts_with('\'') => {
                chars.next();
                value.push('\'');
            }
            Some(c) if c == quote => return after_quote(line, chars.as_str()).map(|()| value),
            Some('\\') if quote == '"' => value.push(match chars.next() {
                Some('"') => '"',
                Some('\\') => '\\',
                Some('n') => '\n',
                Some('t') => '\t',
                Some(_) => return Err(line.error(here, ErrorKind::InvalidEscape)),
                None => return Err(line.error(text, ErrorKind::UnclosedQuote)),
            }),
            Some(c) => value.push(c),
        }
    }
}

/// Checks the end of a line after a closing quote: nothing, or a comment after a space.
fn after_quote(line: &Line<'_>, rest: &str) -> Result<(), Error> {
    let comment = rest.trim_start_matches(' ');
    if rest.is_empty() || (rest.starts_with(' ') && comment.starts_with('#')) {
        Ok(())
    } else {
        Err(line.error(comment, ErrorKind::TextAfterQuote))
    }
}

impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.pos, self.kind)
    }
}

impl std::error::Error for Error {}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::DuplicateKey { key, first_line } => {
                return write!(f, "duplicate key `{key}`, first on line {first_line}");
            }
            Self::MissingHeader => "expected a line `---` that opens the header",
            Self::UnclosedHeader => "the header has no closing line `---`",
            Self::ControlCharacter('\t') => "tab character; indent with spaces",
            Self::ControlCharacter(_) => "control character",
            Self::Noncharacter(c) => {
                let code = u32::from(*c);
                return write!(
                    f,
                    "U+{code:04X} is a Unicode noncharacter, not text; delete it"
                );
            }
            Self::BadIndent => "unexpected indentation; a value fits on one line",
            Self::ExpectedKey => "expected `key: value`, a key of letters, digits, `-` or `_`",
            Self::ExpectedListItem => "expected a list item: `- ` and a value",
            Self::MissingValue => "missing value; put it on this line, or indented below",
            Self::UnclosedQuote => "the quote is not closed on this line",
            Self::InvalidEscape => "unknown escape; use `\\\"`, `\\\\`, `\\n` or `\\t`",
            Self::TextAfterQuote => "unexpected text after the closing quote",
            Self::ColonInValue => "a value with `: ` in it or `:` at its end needs quotes",
            Self::Indicator('&' | '*' | '!') => "anchors, aliases and tags are not supported",
            Self::Indicator('[' | '{') => "`[` and `{` are not supported; write one item per line",
            Self::Indicator('|' | '>') => "block scalars are not supported; use one line",
            Self::Indicator('?') => "complex keys are not supported",
            Self::Indicator('-') => "a list starts on the line below its key",
            Self::Indicator(_) => "a value cannot start with this character unless quoted",
        };
        f.write_str(message)
    }
}
