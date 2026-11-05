//! A small template language for the site's layouts, rendered with a context of values.
//!
//! `{{ path }}` writes a text with HTML's special characters escaped, and `{{ path | safe }}` as
//! it is. `{% if path %}`, `{% else %}` and `{% endif %}` keep one of two parts: the first when the
//! value is `true`, or a text, list or map that is not empty. `{% for name in path %}` and
//! `{% endfor %}` repeat a part for each value of a list. `{# … #}` is a comment. A path is names
//! joined by `.`, looked up in the loops around it, innermost first, and then in the context.
//! Everything else is written as it is. Other tags and filters, and values that are missing or of
//! the wrong kind where they are needed, are errors with a line and column; [`ErrorKind`] lists
//! them.

mod parse;

use std::fmt;

/// How many `if` and `for` may hold a part; deeper nesting is an error, not a stack overflow.
const MAX_DEPTH: usize = 8;

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
    /// The text of a tag other than `if path`, `else`, `endif`, `for name in path` and `endfor`.
    BadTag(String),
    /// A filter other than `safe`.
    UnknownFilter(String),
    /// A path, or the name of a loop's item, that is not names of ASCII letters, digits and `_`
    /// joined by `.`, or for the item, one name.
    BadPath(String),
    /// An `else` outside an `if`, or after another `else`.
    StrayElse,
    /// An `endif` that ends no `if`.
    StrayEndif,
    /// An `endfor` that ends no `for`.
    StrayEndfor,
    /// An `if` without its `endif`.
    UnendedIf,
    /// A `for` without its `endfor`.
    UnendedFor,
    /// An `if` or `for` inside 8 others.
    TooDeep,
    /// A path without a value, in `{{ }}` or after `in`.
    UnknownVariable(String),
    /// A path in `{{ }}` whose value is not text.
    NotText(String),
    /// A path after `in` whose value is not a list.
    NotList(String),
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

    /// Whether `if` keeps its first part for this value.
    fn is_true(&self) -> bool {
        match self {
            Self::Str(text) => !text.is_empty(),
            Self::Bool(value) => *value,
            Self::List(items) => !items.is_empty(),
            Self::Map(entries) => !entries.is_empty(),
        }
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
    Print {
        path: Path,
        safe: bool,
    },
    If {
        path: Path,
        then: Vec<Node>,
        otherwise: Vec<Node>,
    },
    For {
        name: String,
        path: Path,
        body: Vec<Node>,
    },
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
        let nodes = parse::parse(text)?;
        Ok(Self { nodes })
    }

    /// Renders the template with `context`, a map of the values that its paths name.
    pub fn render(&self, context: &Value) -> Result<String, Error> {
        let mut out = String::new();
        render(&self.nodes, context, &mut Vec::new(), &mut out)?;
        Ok(out)
    }
}

/// The loop items around a node, by name, innermost last.
type Scope<'a> = Vec<(&'a str, &'a Value)>;

/// Appends what `nodes` render with `context` and `scope` to `out`.
fn render<'a>(
    nodes: &'a [Node],
    context: &'a Value,
    scope: &mut Scope<'a>,
    out: &mut String,
) -> Result<(), Error> {
    for node in nodes {
        match node {
            Node::Text(text) => out.push_str(text),
            Node::Print { path, safe } => match path.lookup(context, scope) {
                Some(Value::Str(text)) if *safe => out.push_str(text),
                Some(Value::Str(text)) => escape(out, text),
                Some(Value::Bool(_) | Value::List(_) | Value::Map(_)) => {
                    return Err(path.error(ErrorKind::NotText));
                }
                None => return Err(path.error(ErrorKind::UnknownVariable)),
            },
            Node::If {
                path,
                then,
                otherwise,
            } => {
                let is_true = path.lookup(context, scope).is_some_and(Value::is_true);
                render(if is_true { then } else { otherwise }, context, scope, out)?;
            }
            Node::For { name, path, body } => {
                let items = match path.lookup(context, scope) {
                    Some(Value::List(items)) => items,
                    Some(Value::Str(_) | Value::Bool(_) | Value::Map(_)) => {
                        return Err(path.error(ErrorKind::NotList));
                    }
                    None => return Err(path.error(ErrorKind::UnknownVariable)),
                };
                for item in items {
                    scope.push((name, item));
                    render(body, context, scope, out)?;
                    scope.pop();
                }
            }
        }
    }
    Ok(())
}

impl Path {
    /// The value that the path names: its first name in the innermost loop that has it, or else in
    /// `context`, then each further name in the map before it.
    fn lookup<'a>(&self, context: &'a Value, scope: &Scope<'a>) -> Option<&'a Value> {
        let mut names = self.text.split('.');
        let first = names.next()?;
        let item = scope.iter().rev().find(|(name, _)| *name == first);
        let start = item
            .map(|(_, value)| *value)
            .or_else(|| context.get(first))?;
        names.try_fold(start, |value, name| value.get(name))
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
                "`{{% {text} %}}` is not a tag; the tags are `if path`, `else`, `endif`, \
                 `for name in path` and `endfor`, and a layout wraps every page instead of \
                 `extends` and `block`"
            ),
            Self::UnknownFilter(name) => {
                write!(f, "unknown filter `{name}`; the only filter is `safe`")
            }
            Self::BadPath(text) => write!(
                f,
                "`{text}` is not a path: names of ASCII letters, digits and `_` joined by `.`, \
                 such as `page.title`"
            ),
            Self::StrayElse => f.write_str("an `else` goes once inside an `if`"),
            Self::StrayEndif => f.write_str("this `endif` ends no `if`"),
            Self::StrayEndfor => f.write_str("this `endfor` ends no `for`"),
            Self::UnendedIf => f.write_str("this `if` has no `endif`"),
            Self::UnendedFor => f.write_str("this `for` has no `endfor`"),
            Self::TooDeep => write!(f, "`if` and `for` nest at most {MAX_DEPTH} deep"),
            Self::UnknownVariable(path) => write!(f, "unknown variable `{path}`"),
            Self::NotText(path) => write!(f, "`{path}` is not text"),
            Self::NotList(path) => write!(f, "`{path}` is not a list"),
        }
    }
}
