//! A small template language for the site's layouts, rendered with a context of values.
//!
//! `{{ path }}` writes a text with HTML's special characters escaped, and `{{ path | safe }}` as
//! it is. A path is names joined by `.`, looked up in the context. `{# … #}` is a comment, and
//! everything else is written as it is. A `{% … %}` tag, another filter, and a value that is
//! missing or not text are errors with a line and column; [`ErrorKind`] lists them.

use std::fmt;

/// A line and a column, both counted from 1; columns count characters, not bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pos {
    pub line: usize,
    pub col: usize,
}

/// Where and why a template cannot be parsed or rendered.
#[derive(Debug, PartialEq, Eq)]
pub struct Error {
    pub pos: Pos,
    pub kind: ErrorKind,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ErrorKind {
    /// A `{{`, `{%` or `{#` without the `}}`, `%}` or `#}` that closes it, which is given.
    UnclosedTag(&'static str),
    /// The text of a `{% … %}` tag.
    BadTag(String),
    /// A filter other than `safe`.
    UnknownFilter(String),
    /// A path that is not names of ASCII letters, digits and `_` joined by `.`.
    BadPath(String),
    /// A path in `{{ }}` without a value.
    UnknownVariable(String),
    /// A path in `{{ }}` whose value is not text.
    NotText(String),
}

/// A value in the context of a template.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Str(String),
    Bool(bool),
    List(Vec<Value>),
    /// Names and their values, in order.
    Map(Vec<(String, Value)>),
}

impl Value {
    /// The value of `name` in a map.
    fn get(&self, name: &str) -> Option<&Self> {
        let Self::Map(entries) = self else {
            return None;
        };
        entries
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value)
    }
}

/// A parsed template, ready to render with any context.
#[derive(Debug)]
pub struct Template {
    nodes: Vec<Node>,
}

#[derive(Debug)]
enum Node {
    Text(String),
    Print { path: Path, safe: bool },
}

/// Names joined by `.`, and where they start in the template.
#[derive(Debug)]
struct Path {
    pos: Pos,
    text: String,
}

impl Template {
    /// Parses `text`, the source of a template.
    pub fn parse(text: &str) -> Result<Self, Error> {
        let mut nodes = Vec::new();
        let mut at = 0;
        while let Some(open) = next_tag(text, at) {
            push_text(&mut nodes, text.get(at..open).unwrap_or_default());
            let close = match text.as_bytes().get(open + 1) {
                Some(b'{') => "}}",
                Some(b'%') => "%}",
                _ => "#}",
            };
            let start = open + 2;
            let Some(len) = text.get(start..).and_then(|rest| rest.find(close)) else {
                return Err(Error::new(pos(text, open), ErrorKind::UnclosedTag(close)));
            };
            let inner = text.get(start..start + len).unwrap_or_default();
            // A tag's position is that of its first word.
            let first = pos(text, start + inner.len() - inner.trim_start().len());
            match close {
                "}}" => nodes.push(print(inner.trim(), first)?),
                "%}" => return Err(Error::new(first, ErrorKind::BadTag(inner.trim().into()))),
                _ => {}
            }
            at = start + len + close.len();
        }
        push_text(&mut nodes, text.get(at..).unwrap_or_default());
        Ok(Self { nodes })
    }

    /// Renders the template with `context`, a map of the values that its paths name.
    pub fn render(&self, context: &Value) -> Result<String, Error> {
        let mut out = String::new();
        for node in &self.nodes {
            match node {
                Node::Text(text) => out.push_str(text),
                Node::Print { path, safe } => match path.lookup(context) {
                    Some(Value::Str(text)) if *safe => out.push_str(text),
                    Some(Value::Str(text)) => escape(&mut out, text),
                    Some(_) => return Err(path.error(ErrorKind::NotText)),
                    None => return Err(path.error(ErrorKind::UnknownVariable)),
                },
            }
        }
        Ok(out)
    }
}

/// Where the next `{{`, `{%` or `{#` at or after byte `at` of `text` starts.
fn next_tag(text: &str, at: usize) -> Option<usize> {
    let mut braces = text
        .get(at..)?
        .match_indices('{')
        .map(|(index, _)| at + index);
    braces.find(|&index| matches!(text.as_bytes().get(index + 1), Some(b'{' | b'%' | b'#')))
}

/// The line and column of byte `at` of `text`.
fn pos(text: &str, at: usize) -> Pos {
    let before = text.get(..at).unwrap_or_default();
    let line = before.matches('\n').count() + 1;
    let last = before.rsplit('\n').next().unwrap_or_default();
    let col = last.chars().count() + 1;
    Pos { line, col }
}

fn push_text(nodes: &mut Vec<Node>, text: &str) {
    if !text.is_empty() {
        nodes.push(Node::Text(text.to_owned()));
    }
}

/// `{{ path }}` or `{{ path | safe }}`, without the braces.
fn print(inner: &str, pos: Pos) -> Result<Node, Error> {
    let (path, filter) = match inner.split_once('|') {
        Some((path, filter)) => (path.trim_end(), Some(filter.trim())),
        None => (inner, None),
    };
    let safe = match filter {
        None => false,
        Some("safe") => true,
        Some(filter) => return Err(Error::new(pos, ErrorKind::UnknownFilter(filter.into()))),
    };
    let path = Path::parse(path, pos)?;
    Ok(Node::Print { path, safe })
}

impl Path {
    fn parse(text: &str, pos: Pos) -> Result<Self, Error> {
        let is_name = |name: &str| {
            name.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
                && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        };
        let text = text.to_owned();
        if text.split('.').all(is_name) {
            Ok(Self { pos, text })
        } else {
            Err(Error::new(pos, ErrorKind::BadPath(text)))
        }
    }

    /// The value that the path names in `context`: each name in the map before it.
    fn lookup<'a>(&self, context: &'a Value) -> Option<&'a Value> {
        let mut names = self.text.split('.');
        let first = context.get(names.next()?)?;
        names.try_fold(first, |value, name| value.get(name))
    }

    /// An error at the path, about it.
    fn error(&self, kind: fn(String) -> ErrorKind) -> Error {
        Error::new(self.pos, kind(self.text.clone()))
    }
}

/// Appends `text` to `out` with `&`, `<`, `>`, `"` and `'` escaped, for HTML text and attributes.
fn escape(out: &mut String, text: &str) {
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
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

impl Error {
    fn new(pos: Pos, kind: ErrorKind) -> Self {
        Self { pos, kind }
    }
}

impl std::error::Error for Error {}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnclosedTag(close) => write!(f, "no `{close}` closes this tag"),
            Self::BadTag(text) => write!(
                f,
                "`{{% {text} %}}` is not a tag; a layout wraps every page instead of `extends` and \
                 `block`"
            ),
            Self::UnknownFilter(name) => {
                write!(f, "unknown filter `{name}`; the only filter is `safe`")
            }
            Self::BadPath(text) => write!(
                f,
                "`{text}` is not a path: names of ASCII letters, digits and `_` joined by `.`, \
                 such as `page.title`"
            ),
            Self::UnknownVariable(path) => write!(f, "unknown variable `{path}`"),
            Self::NotText(path) => write!(f, "`{path}` is not text"),
        }
    }
}
