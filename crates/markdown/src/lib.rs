//! A strict subset of CommonMark for the pages of the site, rendered to HTML as Zola renders it.
//!
//! The blocks are paragraphs; `#` headings, each with an id made from its text; code between
//! ```` ``` ```` or `~~~` fences; `>` quotes; and lists of `-`, `*` or `+` items or of numbered
//! items such as `1.`, nested by the column where an item's text starts. In paragraphs and
//! headings are code spans, `<url>` and `<email>` autolinks, backslash escapes, the references
//! `&amp;`, `&lt;`, `&gt;` and `&quot;`, and line breaks after two spaces or a `\`; links, images
//! and emphasis are text, and so are link reference definitions. Tabs, indented code, underlined
//! headings, horizontal rules, HTML, other character references, and lines that CommonMark reads
//! in a way that is easy to miss, such as a line that continues a list item without its
//! indentation, are errors with a line and column; [`ErrorKind`] lists them.

mod inline;

use std::fmt;

/// How many lists and quotes may hold a line; deeper nesting is an error, not a stack overflow.
const MAX_DEPTH: usize = 8;

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
    /// A line right under a list item or quote, without the item's indentation or the quote's
    /// `>`, which CommonMark may still add to the item's or quote's last paragraph.
    LazyLine,
    /// A list item without text on the line of its marker.
    EmptyListItem,
    /// An opening fence without its closing fence in the same quote, list item or page.
    UnclosedFence,
    /// A line of `=` or `-` under a paragraph, which CommonMark reads as a heading.
    UnderlinedHeading,
    /// A line of three or more `-`, `*` or `_`: a horizontal rule.
    Rule,
    /// A letter or digit outside ASCII in a heading, which Zola spells out in the heading's id.
    NonAsciiInHeading(char),
    /// A heading without an ASCII letter or digit to make its id from.
    NoHeadingId,
    /// A list or quote inside 8 others.
    TooDeep,
    /// A `<` before a letter, `/`, `!` or `?` that does not start an autolink: CommonMark may read
    /// it as HTML.
    RawHtml,
    /// A character reference other than `&amp;`, `&lt;`, `&gt;` and `&quot;`.
    Entity,
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
    let (blocks, _) = Parser::new(lines, 0).blocks()?;
    let mut html = Html::default();
    html.blocks(&blocks, false)?;
    Ok(html.text)
}

#[derive(Clone, Copy)]
struct Line<'a> {
    number: usize,
    /// The whole line; columns count from its start.
    full: &'a str,
    /// The end of `full` inside the quotes and list items that hold the line.
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

    /// The line inside a block whose text starts `columns` columns in. A blank line may have
    /// fewer columns than that.
    fn inside(self, columns: usize) -> Self {
        self.with_text(self.text.get(columns..).unwrap_or_default())
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
    /// The language named after the opening fence, and the lines between the fences.
    Code(Option<&'a str>, Vec<&'a str>),
    Quote(Vec<Block<'a>>),
    List(List<'a>),
}

struct Heading<'a> {
    level: usize,
    line: Line<'a>,
    /// What follows the `#`s and the spaces after them, an end of `line`.
    text: &'a str,
}

struct List<'a> {
    /// The first item's number; `None` for a bulleted list.
    number: Option<u32>,
    loose: bool,
    items: Vec<Vec<Block<'a>>>,
}

/// A line that starts a block other than a paragraph.
enum Start<'a> {
    Heading(usize, &'a str),
    Fence(Fence<'a>),
    Quote,
    Rule,
    Item(Item),
}

/// An opening fence: its indentation, its mark and how many of them, and the info string after it.
struct Fence<'a> {
    indent: usize,
    mark: char,
    len: usize,
    info: &'a str,
}

struct Item {
    /// The bullet, or the `.` or `)` after the number; an item with another marker starts another
    /// list.
    marker: char,
    number: Option<u32>,
    /// The column where the item's text starts, and to which its other lines are indented.
    width: usize,
    empty: bool,
}

/// What `line` starts, if it starts a block other than a paragraph.
fn start<'a>(line: &Line<'a>) -> Option<Start<'a>> {
    let (indent, content) = (line.indent(), line.content());
    if indent > 3 {
        None
    } else if is_rule(content) {
        Some(Start::Rule)
    } else if content.starts_with('>') {
        Some(Start::Quote)
    } else if let Some((level, text)) = heading(content) {
        Some(Start::Heading(level, text))
    } else if let Some(fence) = fence(content, indent) {
        Some(Start::Fence(fence))
    } else {
        item(content, indent).map(Start::Item)
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

/// An opening fence: 3 or more backticks or tildes, then an info string, which has no backtick
/// after backticks.
fn fence(content: &str, indent: usize) -> Option<Fence<'_>> {
    let mark = content.chars().next().filter(|c| matches!(c, '`' | '~'))?;
    let rest = content.trim_start_matches(mark);
    let len = content.len() - rest.len();
    let info = rest.trim_matches(' ');
    let valid = len >= 3 && !(mark == '`' && info.contains('`'));
    valid.then_some(Fence {
        indent,
        mark,
        len,
        info,
    })
}

/// A list item: a bullet, or 1 to 9 digits and `.` or `)`, then spaces or the end of the line.
fn item(content: &str, indent: usize) -> Option<Item> {
    let digits = content.trim_start_matches(|c: char| c.is_ascii_digit());
    let (marker, number, rest) = match content.len() - digits.len() {
        0 => {
            let rest = content.strip_prefix(['-', '*', '+'])?;
            (content.chars().next()?, None, rest)
        }
        1..=9 => {
            let rest = digits.strip_prefix(['.', ')'])?;
            let number = content.strip_suffix(digits)?.parse().ok()?;
            (digits.chars().next()?, Some(number), rest)
        }
        _ => return None,
    };
    let text = rest.trim_start_matches(' ');
    let spaces = match rest.len() - text.len() {
        0 if !text.is_empty() => return None,
        // Text 5 spaces or more after the marker is indented code, which starts 1 space after it.
        0 | 5.. => 1,
        spaces => spaces,
    };
    let width = indent + content.len() - rest.len() + spaces;
    let empty = text.is_empty();
    Some(Item {
        marker,
        number,
        width,
        empty,
    })
}

/// Whether `line`, under a paragraph line, makes that line a heading: only `=` or only `-`.
fn is_underline(line: &Line<'_>) -> bool {
    let marks = line.content().trim_end_matches(' ');
    line.indent() <= 3 && (marks.chars().all(|c| c == '=') || marks.chars().all(|c| c == '-'))
}

/// Whether `line` ends the paragraph above it: any block but an empty list item or one numbered
/// from other than 1.
fn interrupts(line: &Line<'_>) -> bool {
    match start(line) {
        Some(Start::Item(item)) => !item.empty && item.number.is_none_or(|n| n == 1),
        Some(_) => true,
        None => false,
    }
}

struct Parser<'a> {
    lines: Vec<Line<'a>>,
    /// The first line not parsed yet.
    next: usize,
    /// How many lists and quotes hold these lines.
    depth: usize,
}

impl<'a> Parser<'a> {
    fn new(lines: Vec<Line<'a>>, depth: usize) -> Self {
        let next = 0;
        Self { lines, next, depth }
    }

    fn peek(&self) -> Option<Line<'a>> {
        self.lines.get(self.next).copied()
    }

    /// Parses the lines into blocks, and tells whether a blank line separates two of them.
    fn blocks(mut self) -> Result<(Vec<Block<'a>>, bool), Error> {
        let mut blocks = Vec::new();
        let (mut blank, mut gap) = (false, false);
        while let Some(line) = self.peek() {
            if line.is_blank() {
                self.next += 1;
                blank = !blocks.is_empty();
                continue;
            }
            gap |= blank;
            let block = match start(&line) {
                Some(Start::Heading(level, text)) => {
                    self.next += 1;
                    Block::Heading(Heading { level, line, text })
                }
                Some(Start::Fence(fence)) => self.code(line, &fence)?,
                Some(Start::Quote | Start::Item(_)) if self.depth == MAX_DEPTH => {
                    return Err(line.error(ErrorKind::TooDeep));
                }
                Some(Start::Quote) => self.quote()?,
                Some(Start::Item(item)) => self.list(item)?,
                Some(Start::Rule) => return Err(line.error(ErrorKind::Rule)),
                None if line.indent() > 3 => return Err(line.error(ErrorKind::IndentedCode)),
                None => self.paragraph(line)?,
            };
            blocks.push(block);
        }
        Ok((blocks, gap))
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
            if interrupts(&line) {
                break;
            }
            lines.push(line.with_text(line.content()));
            self.next += 1;
        }
        Ok(Block::Paragraph(lines))
    }

    /// Fenced code, from the line after the opening fence to a closing fence: at least as many of
    /// the same marks, and nothing else.
    fn code(&mut self, open: Line<'a>, fence: &Fence<'a>) -> Result<Block<'a>, Error> {
        self.next += 1;
        let mut code = Vec::new();
        while let Some(line) = self.peek() {
            self.next += 1;
            let marks = line.content().trim_end_matches(' ');
            let closes =
                marks.trim_start_matches(fence.mark).is_empty() && marks.len() >= fence.len;
            if line.indent() <= 3 && closes {
                let language = fence.info.split(' ').next().filter(|word| !word.is_empty());
                return Ok(Block::Code(language, code));
            }
            // As much of the opening fence's indentation as each line has is not code.
            code.push(line.inside(fence.indent.min(line.indent())).text);
        }
        Err(open.error(ErrorKind::UnclosedFence))
    }

    /// A quote: the lines that start with `>`, without it and one space after it.
    fn quote(&mut self) -> Result<Block<'a>, Error> {
        let mut lines = Vec::new();
        while let Some(line) = self.peek().filter(|line| line.indent() <= 3) {
            let Some(text) = line.content().strip_prefix('>') else {
                break;
            };
            lines.push(line.with_text(text.strip_prefix(' ').unwrap_or(text)));
            self.next += 1;
        }
        self.reject_lazy(lines.last())?;
        let (blocks, _) = Parser::new(lines, self.depth + 1).blocks()?;
        Ok(Block::Quote(blocks))
    }

    /// A list: items with the same marker, each made of its first line and the lines after it
    /// that are blank or indented to its text.
    fn list(&mut self, first: Item) -> Result<Block<'a>, Error> {
        let number = first.number;
        let (mut items, mut loose, mut item) = (Vec::new(), false, first);
        while let Some(line) = self.peek() {
            if item.empty {
                return Err(line.error(ErrorKind::EmptyListItem));
            }
            let mut lines = vec![line.inside(item.width)];
            self.next += 1;
            let in_item = |line: &Line<'_>| line.is_blank() || line.indent() >= item.width;
            while let Some(line) = self.peek().filter(in_item) {
                lines.push(line.inside(item.width));
                self.next += 1;
            }
            // Blank lines at the end separate the item from what follows. The first line stays,
            // so each item moves the parser on.
            while lines.len() > 1 && lines.pop_if(|line| line.is_blank()).is_some() {
                self.next -= 1;
            }
            self.reject_lazy(lines.last())?;
            let (blocks, gap) = Parser::new(lines, self.depth + 1).blocks()?;
            items.push(blocks);
            loose |= gap;
            let end = self.next;
            while self.peek().is_some_and(|line| line.is_blank()) {
                self.next += 1;
            }
            match self.peek().and_then(|line| start(&line)) {
                Some(Start::Item(next)) if next.marker == item.marker => {
                    loose |= self.next > end;
                    item = next;
                }
                _ => {
                    self.next = end;
                    break;
                }
            }
        }
        Ok(Block::List(List {
            number,
            loose,
            items,
        }))
    }

    /// Rejects the line after a list item or quote whose last line is `last` if CommonMark may add
    /// it to them: text right under text.
    fn reject_lazy(&self, last: Option<&Line<'a>>) -> Result<(), Error> {
        match (last, self.peek()) {
            (Some(last), Some(line))
                if !last.is_blank() && !line.is_blank() && start(&line).is_none() =>
            {
                Err(line.error(ErrorKind::LazyLine))
            }
            _ => Ok(()),
        }
    }
}

/// The HTML so far, and the ids of its headings.
#[derive(Default)]
struct Html {
    text: String,
    ids: Vec<String>,
}

impl Html {
    /// Starts a paragraph, quote or list on a new line. Zola writes headings and code blocks
    /// without one, even right after `<li>` or an item's text.
    fn newline(&mut self) {
        if !self.text.is_empty() && !self.text.ends_with('\n') {
            self.text.push('\n');
        }
    }

    /// Renders blocks; in an item of a tight list, a paragraph is its bare text.
    fn blocks(&mut self, blocks: &[Block<'_>], tight: bool) -> Result<(), Error> {
        for block in blocks {
            match block {
                Block::Paragraph(lines) if tight => self.paragraph(lines)?,
                Block::Paragraph(lines) => {
                    self.newline();
                    self.text.push_str("<p>");
                    self.paragraph(lines)?;
                    self.text.push_str("</p>\n");
                }
                Block::Heading(heading) => self.heading(heading)?,
                Block::Code(language, lines) => self.code(*language, lines),
                Block::Quote(blocks) => {
                    self.newline();
                    self.text.push_str("<blockquote>\n");
                    self.blocks(blocks, false)?;
                    self.text.push_str("</blockquote>\n");
                }
                Block::List(list) => self.list(list)?,
            }
        }
        Ok(())
    }

    /// The inlines of a paragraph: its lines joined by line breaks, without the spaces that end it.
    fn paragraph(&mut self, lines: &[Line<'_>]) -> Result<(), Error> {
        let Some(first) = lines.first() else {
            return Ok(());
        };
        let texts: Vec<&str> = lines.iter().map(|line| line.text).collect();
        let joined = texts.join("\n");
        let text = joined.trim_end_matches(' ');
        let inlines = inline::parse(text).map_err(|(at, kind)| {
            // The line that holds byte `at`, and where that line starts.
            let before = text.get(..at).unwrap_or_default();
            let line = lines.get(before.matches('\n').count()).unwrap_or(first);
            let start = before.rfind('\n').map_or(0, |index| index + 1);
            line.error_at(line.text.get(at - start..).unwrap_or_default(), kind)
        })?;
        inline::write(&mut self.text, &inlines);
        Ok(())
    }

    /// A heading with Zola's id for it, made unique on the page with `-1`, `-2` and so on.
    fn heading(&mut self, heading: &Heading<'_>) -> Result<(), Error> {
        let Heading { level, line, text } = heading;
        let error = |at: usize, kind| line.error_at(text.get(at..).unwrap_or_default(), kind);
        let shown = without_closing_marks(text);
        let inlines = inline::parse(shown).map_err(|(at, kind)| error(at, kind))?;
        // Zola makes the id from the heading's text without its markup.
        let words = inline::plain(&inlines);
        if let Some(c) = words.chars().find(|c| c.is_alphanumeric() && !c.is_ascii()) {
            let at = text.find(c).unwrap_or_default();
            return Err(error(at, ErrorKind::NonAsciiInHeading(c)));
        }
        let base = slug(&words);
        if base.is_empty() {
            return Err(line.error(ErrorKind::NoHeadingId));
        }
        let (mut id, mut n) = (base.clone(), 0);
        while self.ids.contains(&id) {
            n += 1;
            id = format!("{base}-{n}");
        }
        self.text.push_str(&format!("<h{level} id=\"{id}\">"));
        inline::write(&mut self.text, &inlines);
        self.text.push_str(&format!("</h{level}>\n"));
        self.ids.push(id);
        Ok(())
    }

    /// A code block, with its language as CommonMark marks it, where Zola writes `data-lang`.
    fn code(&mut self, language: Option<&str>, lines: &[&str]) {
        self.text.push_str("<pre><code");
        if let Some(language) = language {
            self.text.push_str(" class=\"language-");
            escape_attribute(&mut self.text, language);
            self.text.push('"');
        }
        self.text.push('>');
        for line in lines {
            escape(&mut self.text, line);
            self.text.push('\n');
        }
        self.text.push_str("</code></pre>\n");
    }

    fn list(&mut self, list: &List<'_>) -> Result<(), Error> {
        let tag = if list.number.is_some() { "ol" } else { "ul" };
        self.newline();
        match list.number {
            Some(start) if start != 1 => self.text.push_str(&format!("<ol start=\"{start}\">\n")),
            _ => self.text.push_str(&format!("<{tag}>\n")),
        }
        for item in &list.items {
            self.text.push_str("<li>");
            self.blocks(item, !list.loose)?;
            self.text.push_str("</li>\n");
        }
        self.text.push_str(&format!("</{tag}>\n"));
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

/// Appends an attribute value to `html`: escaped as text, and `"` too.
fn escape_attribute(html: &mut String, value: &str) {
    for (index, part) in value.split('"').enumerate() {
        if index > 0 {
            html.push_str("&quot;");
        }
        escape(html, part);
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
            Self::IndentedCode => "indented code is not supported; indent less, or use a ``` fence",
            Self::LazyLine => {
                "indent the line to continue the list item, or start it with `>` to continue the \
                 quote, or leave a blank line before it"
            }
            Self::EmptyListItem => "a list item needs text after its marker",
            Self::UnclosedFence => "the code block has no closing fence",
            Self::UnderlinedHeading => "start a heading with `#` instead of underlining it",
            Self::Rule => "horizontal rules are not supported",
            Self::NoHeadingId => "a heading needs an ASCII letter or digit, for its id",
            Self::TooDeep => {
                return write!(f, "lists and quotes nest at most {MAX_DEPTH} deep");
            }
            Self::RawHtml => {
                "HTML is not supported; write `\\<` for a `<`, or put code in backticks"
            }
            Self::Entity => {
                "the only character references are `&amp;`, `&lt;`, `&gt;` and `&quot;`; type the \
                 character itself"
            }
        };
        f.write_str(message)
    }
}
