//! A strict subset of CommonMark for the pages of the site, rendered to HTML.
//!
//! The blocks are paragraphs and `#` headings, each heading with an id made from its text. Text is
//! escaped for HTML as written: inline syntax is not interpreted, and the lines of other blocks,
//! such as lists, read as paragraph text. Tabs, indented code, headings underlined with `=` or
//! `-`, and horizontal rules are errors with a line and column.

use std::fmt;

/// A line and a column, both counted from 1; columns count characters, not bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pos {
    pub line: usize,
    pub col: usize,
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
    /// A line indented 4 spaces or more past its block, which CommonMark reads as code.
    IndentedCode,
    /// A line of `=` or `-` under a paragraph, which CommonMark reads as a heading.
    UnderlinedHeading,
    /// A line of three or more `-`, `*` or `_`: a horizontal rule.
    Rule,
    /// A letter or digit outside ASCII in a heading, which Zola spells out in the heading's id.
    NonAsciiInHeading(char),
    /// A heading without an ASCII letter or digit to make its id from.
    NoHeadingId,
}

/// Renders `text`, the Markdown of a page after its header, to HTML. An error's line counts from
/// the first line of `text`.
pub fn to_html(text: &str) -> Result<String, Error> {
    let mut lines = Vec::new();
    for (full, number) in text.lines().zip(1..) {
        if let Some((index, c)) = full.chars().enumerate().find(|(_, c)| c.is_control()) {
            let (line, col, kind) = (number, index + 1, ErrorKind::ControlCharacter(c));
            let pos = Pos { line, col };
            return Err(Error { pos, kind });
        }
        lines.push(Line {
            number,
            full,
            text: full,
        });
    }
    let blocks = Parser { lines, next: 0 }.blocks()?;
    let mut html = Html::default();
    html.blocks(&blocks)?;
    Ok(html.text)
}

#[derive(Clone, Copy)]
struct Line<'a> {
    number: usize,
    /// The whole line; columns count from its start.
    full: &'a str,
    /// The end of `full` that is left to parse.
    text: &'a str,
}

impl<'a> Line<'a> {
    /// The text after its indentation.
    fn content(&self) -> &'a str {
        self.text.trim_start_matches(' ')
    }

    /// The width of the indentation: spaces only, as tabs are errors.
    fn indent(&self) -> usize {
        self.text.len() - self.content().len()
    }

    fn is_blank(&self) -> bool {
        self.content().is_empty()
    }

    fn with_text(self, text: &'a str) -> Self {
        Self { text, ..self }
    }

    /// An error at the start of the content.
    fn error(&self, kind: ErrorKind) -> Error {
        self.error_at(self.content(), kind)
    }

    /// An error at `part`, an end of this line.
    fn error_at(&self, part: &str, kind: ErrorKind) -> Error {
        let line = self.number;
        let col = self.full.chars().count() - part.chars().count() + 1;
        let pos = Pos { line, col };
        Error { pos, kind }
    }
}

enum Block<'a> {
    /// The lines of a paragraph, without their indentation.
    Paragraph(Vec<Line<'a>>),
    Heading(Heading<'a>),
}

struct Heading<'a> {
    level: usize,
    line: Line<'a>,
    /// What follows the `#`s and the spaces after them, an end of `line`.
    text: &'a str,
}

/// A line that starts a block other than a paragraph.
enum Start<'a> {
    Heading(usize, &'a str),
    Rule,
}

/// What `line` starts, if it starts a block other than a paragraph.
fn start<'a>(line: &Line<'a>) -> Option<Start<'a>> {
    let content = line.content();
    if line.indent() > 3 {
        None
    } else if is_rule(content) {
        Some(Start::Rule)
    } else {
        heading(content).map(|(level, text)| Start::Heading(level, text))
    }
}

/// Whether `content` is three or more `-`, `*` or `_` and spaces between them.
fn is_rule(content: &str) -> bool {
    let Some(mark) = content
        .chars()
        .next()
        .filter(|c| matches!(c, '-' | '*' | '_'))
    else {
        return false;
    };
    let marks = content.chars().filter(|&c| c == mark).count();
    marks >= 3 && content.chars().all(|c| c == mark || c == ' ')
}

/// The level and text of a heading: 1 to 6 `#`, then a space or the end of the line.
fn heading(content: &str) -> Option<(usize, &str)> {
    let text = content.trim_start_matches('#');
    let level = content.len() - text.len();
    let valid = (1..=6).contains(&level) && (text.is_empty() || text.starts_with(' '));
    valid.then(|| (level, text.trim_start_matches(' ')))
}

/// Whether `line`, under a paragraph line, makes that line a heading: only `=` or only `-`.
fn is_underline(line: &Line<'_>) -> bool {
    let marks = line.content().trim_end_matches(' ');
    line.indent() <= 3 && (marks.chars().all(|c| c == '=') || marks.chars().all(|c| c == '-'))
}

struct Parser<'a> {
    lines: Vec<Line<'a>>,
    /// The first line not parsed yet.
    next: usize,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<Line<'a>> {
        self.lines.get(self.next).copied()
    }

    fn blocks(mut self) -> Result<Vec<Block<'a>>, Error> {
        let mut blocks = Vec::new();
        while let Some(line) = self.peek() {
            if line.is_blank() {
                self.next += 1;
                continue;
            }
            let block = match start(&line) {
                Some(Start::Heading(level, text)) => {
                    self.next += 1;
                    Block::Heading(Heading { level, line, text })
                }
                Some(Start::Rule) => return Err(line.error(ErrorKind::Rule)),
                None if line.indent() > 3 => return Err(line.error(ErrorKind::IndentedCode)),
                None => self.paragraph(line)?,
            };
            blocks.push(block);
        }
        Ok(blocks)
    }

    /// A paragraph: its first line, and the lines under it up to a blank line or another block.
    /// Indentation does not matter under the first line.
    fn paragraph(&mut self, first: Line<'a>) -> Result<Block<'a>, Error> {
        let mut lines = vec![first.with_text(first.content())];
        self.next += 1;
        while let Some(line) = self.peek().filter(|line| !line.is_blank()) {
            if is_underline(&line) {
                return Err(line.error(ErrorKind::UnderlinedHeading));
            }
            if start(&line).is_some() {
                break;
            }
            lines.push(line.with_text(line.content()));
            self.next += 1;
        }
        Ok(Block::Paragraph(lines))
    }
}

/// The HTML so far, and the ids of its headings.
#[derive(Default)]
struct Html {
    text: String,
    ids: Vec<String>,
}

impl Html {
    fn blocks(&mut self, blocks: &[Block<'_>]) -> Result<(), Error> {
        for block in blocks {
            match block {
                Block::Paragraph(lines) => {
                    self.text.push_str("<p>");
                    self.paragraph(lines);
                    self.text.push_str("</p>\n");
                }
                Block::Heading(heading) => self.heading(heading)?,
            }
        }
        Ok(())
    }

    /// The text of a paragraph's lines, without the spaces that end them.
    fn paragraph(&mut self, lines: &[Line<'_>]) {
        for (index, line) in lines.iter().enumerate() {
            if index > 0 {
                self.text.push('\n');
            }
            escape(&mut self.text, line.text.trim_end_matches(' '));
        }
    }

    /// A heading with Zola's id for it, made unique on the page with `-1`, `-2` and so on.
    fn heading(&mut self, heading: &Heading<'_>) -> Result<(), Error> {
        let Heading { level, line, text } = heading;
        let shown = without_closing_marks(text);
        let is_foreign = |&(_, c): &(usize, char)| c.is_alphanumeric() && !c.is_ascii();
        if let Some((index, c)) = shown.char_indices().find(is_foreign) {
            let part = text.get(index..).unwrap_or_default();
            return Err(line.error_at(part, ErrorKind::NonAsciiInHeading(c)));
        }
        let base = slug(shown);
        if base.is_empty() {
            return Err(line.error(ErrorKind::NoHeadingId));
        }
        let (mut id, mut n) = (base.clone(), 0);
        while self.ids.contains(&id) {
            n += 1;
            id = format!("{base}-{n}");
        }
        self.text.push_str(&format!("<h{level} id=\"{id}\">"));
        escape(&mut self.text, shown);
        self.text.push_str(&format!("</h{level}>\n"));
        self.ids.push(id);
        Ok(())
    }
}

/// A heading's text without the optional closing `#`s, which follow a space.
fn without_closing_marks(text: &str) -> &str {
    let text = text.trim_end_matches(' ');
    let open = text.trim_end_matches('#');
    if open.ends_with(' ') {
        open.trim_end_matches(' ')
    } else {
        text
    }
}

/// The id Zola gives a heading whose letters and digits are ASCII: the runs of them, in lowercase,
/// joined by `-`. Zola spells out emoji and some symbols (`🦀` as `crab`), which here only
/// separate the runs.
fn slug(text: &str) -> String {
    let words: Vec<String> = text
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .map(str::to_ascii_lowercase)
        .collect();
    words.join("-")
}

/// Appends `text` to `html` with `&`, `<` and `>` escaped, as Zola escapes text.
fn escape(html: &mut String, text: &str) {
    for c in text.chars() {
        match c {
            '&' => html.push_str("&amp;"),
            '<' => html.push_str("&lt;"),
            '>' => html.push_str("&gt;"),
            _ => html.push(c),
        }
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
            Self::NonAsciiInHeading(c) => {
                return write!(
                    f,
                    "`{c}` in a heading; an id is made of ASCII letters and digits"
                );
            }
            Self::ControlCharacter('\t') => "tab character; indent with spaces",
            Self::ControlCharacter(_) => "control character",
            Self::IndentedCode => "indented code is not supported; indent the line less",
            Self::UnderlinedHeading => "start a heading with `#` instead of underlining it",
            Self::Rule => "horizontal rules are not supported",
            Self::NoHeadingId => "a heading needs an ASCII letter or digit, for its id",
        };
        f.write_str(message)
    }
}
