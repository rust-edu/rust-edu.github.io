//! Parses the source of a template into its nodes: text, `{{ … }}`, `{% if … %}`, `{% else %}`,
//! `{% endif %}`, `{% for … %}` and `{% endfor %}`, leaving `{# … #}` comments out.

use std::mem;

use crate::{Error, ErrorKind, MAX_DEPTH, Node, Path, Pos};

pub(crate) fn parse(text: &str) -> Result<Vec<Node>, Error> {
    let mut parser = Parser::default();
    let mut at = 0;
    while let Some((open, kind)) = next_tag(text, at) {
        push_text(parser.nodes(), text.get(at..open).unwrap_or_default());
        let close = match kind {
            Open::Print => "}}",
            Open::Tag => "%}",
            Open::Comment => "#}",
        };
        let start = open + 2;
        let Some(len) = text.get(start..).and_then(|rest| rest.find(close)) else {
            return Err(Error::new(pos(text, open), ErrorKind::UnclosedTag(close)));
        };
        let inner = text.get(start..start + len).unwrap_or_default();
        // A tag's position is that of its first word.
        let first = start + inner.len() - inner.trim_start().len();
        match kind {
            Open::Print => parser.nodes().push(print(inner.trim(), pos(text, first))?),
            Open::Tag => parser.tag(inner.trim(), |at| pos(text, first + at))?,
            Open::Comment => {}
        }
        at = start + len + close.len();
    }
    push_text(parser.nodes(), text.get(at..).unwrap_or_default());
    parser.finish()
}

/// The parser's state: the template's nodes so far, and the `if` and `for` not ended yet.
#[derive(Default)]
struct Parser {
    nodes: Vec<Node>,
    blocks: Vec<Block>,
}

/// An `if` or `for` whose end is not parsed yet.
struct Block {
    pos: Pos,
    start: Start,
    nodes: Vec<Node>,
    /// The nodes before the `else` of an `if`, once its `else` is parsed.
    then: Option<Vec<Node>>,
}

enum Start {
    If(Path),
    For(String, Path),
}

impl Parser {
    /// The nodes that a new node goes into: those of the innermost block, or of the template.
    fn nodes(&mut self) -> &mut Vec<Node> {
        match self.blocks.last_mut() {
            Some(block) => &mut block.nodes,
            None => &mut self.nodes,
        }
    }

    /// A `{% … %}` tag without its marks; `pos` gives the position of a byte of it.
    fn tag(&mut self, inner: &str, pos: impl Fn(usize) -> Pos) -> Result<(), Error> {
        let words: Vec<&str> = inner.split_whitespace().collect();
        // The path of `if` and `for` is the tag's last word.
        let last = |word: &str| pos(inner.rfind(word).unwrap_or_default());
        match words.as_slice() {
            ["if", path] => {
                let path = Path::parse(path, last(path))?;
                self.open(pos(0), Start::If(path))
            }
            ["else"] => self.otherwise(pos(0)),
            ["endif"] => self.end_if(pos(0)),
            ["for", name, "in", path] => {
                let after_for = inner.get(3..).unwrap_or_default().find(name);
                let name = Path::parse(name, pos(3 + after_for.unwrap_or_default()))?;
                if name.text.contains('.') {
                    return Err(Error::new(name.pos, ErrorKind::BadPath(name.text)));
                }
                let path = Path::parse(path, last(path))?;
                self.open(pos(0), Start::For(name.text, path))
            }
            ["endfor"] => self.end_for(pos(0)),
            _ => Err(Error::new(pos(0), ErrorKind::BadTag(inner.to_owned()))),
        }
    }

    fn open(&mut self, pos: Pos, start: Start) -> Result<(), Error> {
        if self.blocks.len() == MAX_DEPTH {
            return Err(Error::new(pos, ErrorKind::TooDeep));
        }
        let (nodes, then) = (Vec::new(), None);
        self.blocks.push(Block {
            pos,
            start,
            nodes,
            then,
        });
        Ok(())
    }

    /// An `else`, which ends the first part of the innermost block, an `if`.
    fn otherwise(&mut self, pos: Pos) -> Result<(), Error> {
        match self.blocks.last_mut() {
            Some(Block {
                start: Start::If(_),
                then: then @ None,
                nodes,
                ..
            }) => {
                *then = Some(mem::take(nodes));
                Ok(())
            }
            None
            | Some(
                Block {
                    start: Start::For(..),
                    ..
                }
                | Block { then: Some(_), .. },
            ) => Err(Error::new(pos, ErrorKind::StrayElse)),
        }
    }

    /// An `endif`, which ends the innermost block, an `if`.
    fn end_if(&mut self, pos: Pos) -> Result<(), Error> {
        let block = self
            .blocks
            .pop_if(|block| matches!(block.start, Start::If(_)));
        let Some(Block {
            start: Start::If(path),
            nodes,
            then,
            ..
        }) = block
        else {
            return Err(Error::new(pos, ErrorKind::StrayEndif));
        };
        let (then, otherwise) = match then {
            Some(then) => (then, nodes),
            None => (nodes, Vec::new()),
        };
        self.nodes().push(Node::If {
            path,
            then,
            otherwise,
        });
        Ok(())
    }

    /// An `endfor`, which ends the innermost block, a `for`.
    fn end_for(&mut self, pos: Pos) -> Result<(), Error> {
        let block = self
            .blocks
            .pop_if(|block| matches!(block.start, Start::For(..)));
        let Some(Block {
            start: Start::For(name, path),
            nodes: body,
            ..
        }) = block
        else {
            return Err(Error::new(pos, ErrorKind::StrayEndfor));
        };
        self.nodes().push(Node::For { name, path, body });
        Ok(())
    }

    fn finish(mut self) -> Result<Vec<Node>, Error> {
        if let Some(block) = self.blocks.pop() {
            let kind = match block.start {
                Start::If(_) => ErrorKind::UnendedIf,
                Start::For(..) => ErrorKind::UnendedFor,
            };
            return Err(Error::new(block.pos, kind));
        }
        Ok(self.nodes)
    }
}

/// What `{{`, `{%` or `{#` opens.
enum Open {
    Print,
    Tag,
    Comment,
}

/// Where the next `{{`, `{%` or `{#` at or after byte `at` of `text` starts, and what it opens.
fn next_tag(text: &str, at: usize) -> Option<(usize, Open)> {
    text.get(at..)?.match_indices('{').find_map(|(index, _)| {
        let open = at + index;
        let kind = match text.as_bytes().get(open + 1)? {
            b'{' => Open::Print,
            b'%' => Open::Tag,
            b'#' => Open::Comment,
            _ => return None,
        };
        Some((open, kind))
    })
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
}
