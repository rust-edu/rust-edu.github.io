//! A strict subset of YAML 1.2 for the headers of Markdown pages and for site settings.
//!
//! Only block style: `key: value` lines, and maps indented under `key:`. A scalar is plain,
//! `'single'` or `"double"` quoted, and always a string: its consumer decides whether
//! `2027-03-01` is a date and `true` a boolean, so `no` never turns into `false`. The rest of YAML
//! (tabs, anchors, aliases, tags, flow collections, block scalars, values over several lines) is
//! an error with a line and column, so a YAML 1.2 parser reads whatever this crate accepts as the
//! same maps and text.

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
    ControlCharacter(char),
    /// A line indented deeper than its block but not under a `key:`, such as a value's second line.
    BadIndent,
    ExpectedKey,
    /// A `key:` with no value after it and no lines indented under it.
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
    /// tags, `[` and `{` flow collections, `|` and `>` block scalars, `?` complex keys, `-` lists,
    /// or a character YAML reserves (`%`, `@`, `` ` ``, `,`, `:`, `]`, `}`).
    Indicator(char),
}

/// Parses a text whose top level is a map, such as a settings file.
pub fn parse(text: &str) -> Result<Vec<Entry>, Error> {
    let mut lines = Vec::new();
    for (raw, number) in text.lines().zip(1..) {
        if let Some((index, c)) = raw.chars().enumerate().find(|(_, c)| c.is_control()) {
            let (line, col, kind) = (number, index + 1, ErrorKind::ControlCharacter(c));
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

#[derive(Clone, Copy)]
struct Line<'a> {
    number: usize,
    /// The whole line without trailing spaces; columns count from its start.
    text: &'a str,
    /// The end of `text` left to parse, after the indentation.
    content: &'a str,
}

impl Line<'_> {
    /// The column, counted from 0, where `content` starts. Only spaces come before it, so bytes
    /// count as columns.
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

    /// The map on the lines indented under the `key:` at `pos`, column `indent`.
    fn block(&mut self, indent: usize, pos: Pos) -> Result<Node, Error> {
        let Some(first) = self.peek().filter(|line| line.indent() > indent) else {
            let kind = ErrorKind::MissingValue;
            return Err(Error { pos, kind });
        };
        let value = Value::Map(self.map(first.indent())?);
        let pos = first.pos(first.content);
        Ok(Node { pos, value })
    }
}

/// Splits `key: value` or `key:` into the key and what follows the colon.
fn split_key(content: &str) -> Option<(&str, &str)> {
    let (key, rest) = content.split_once(':')?;
    let is_key_char = |c: char| c.is_ascii_alphanumeric() || c == '-' || c == '_';
    let is_key = !key.is_empty() && key.chars().all(is_key_char);
    (is_key && (rest.is_empty() || rest.starts_with(' '))).then_some((key, rest))
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
            Self::ControlCharacter('\t') => "tab character; indent with spaces",
            Self::ControlCharacter(_) => "control character",
            Self::BadIndent => "unexpected indentation; a value fits on one line",
            Self::ExpectedKey => "expected `key: value`, a key of letters, digits, `-` or `_`",
            Self::MissingValue => "missing value; put it on this line, or indented below",
            Self::UnclosedQuote => "the quote is not closed on this line",
            Self::InvalidEscape => "unknown escape; use `\\\"`, `\\\\`, `\\n` or `\\t`",
            Self::TextAfterQuote => "unexpected text after the closing quote",
            Self::ColonInValue => "a value with `: ` in it or `:` at its end needs quotes",
            Self::Indicator('&' | '*' | '!') => "anchors, aliases and tags are not supported",
            Self::Indicator('[' | '{') => "`[` and `{` are not supported; write one item per line",
            Self::Indicator('|' | '>') => "block scalars are not supported; use one line",
            Self::Indicator('?') => "complex keys are not supported",
            Self::Indicator('-') => "lists are not supported",
            Self::Indicator(_) => "a value cannot start with this character unless quoted",
        };
        f.write_str(message)
    }
}
