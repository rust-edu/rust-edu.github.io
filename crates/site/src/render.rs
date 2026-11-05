//! The settings of `site.yaml` and the layouts of `layouts/`, with which the pages of `content/`
//! will render.

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use edu_template::{Template, Value};
use edu_yaml::{Entry, Node};

use crate::Error;
use crate::check::{ContentError, PageError, read};

/// The layout that wraps every page's own layout, at `{{ content | safe }}`.
const LAYOUT: &str = "layout.html";

/// The settings of `site.yaml` in `root`, as the value `site`; an error in them goes to `errors`.
pub(crate) fn site(root: &Path, errors: &mut Vec<ContentError>) -> Result<Value, Error> {
    let path = PathBuf::from("site.yaml");
    match edu_yaml::parse(&read(root, &path)?) {
        Ok(entries) => Ok(map(&entries)),
        Err(error) => {
            let error = PageError::Header(error);
            errors.push(ContentError { path, error });
            Ok(Value::Map(Vec::new()))
        }
    }
}

fn map(entries: &[Entry]) -> Value {
    let values = entries
        .iter()
        .map(|entry| (entry.key.clone(), value(&entry.value)));
    Value::Map(values.collect())
}

fn value(node: &Node) -> Value {
    match &node.value {
        edu_yaml::Value::Str(text) => Value::Str(text.clone()),
        edu_yaml::Value::List(items) => Value::List(items.iter().map(value).collect()),
        edu_yaml::Value::Map(entries) => map(entries),
    }
}

/// The layouts of `layouts/`, parsed, by file name.
pub(crate) struct Layouts(Vec<(String, Template)>);

impl Layouts {
    /// Parses every `.html` file of `layouts/` in `root`, which must have `layout.html`; a layout
    /// with an error goes to `errors`.
    pub(crate) fn read(root: &Path, errors: &mut Vec<ContentError>) -> Result<Self, Error> {
        let dir = Path::new("layouts");
        let read_error = |source| Error::Read {
            path: dir.to_owned(),
            source,
        };
        let mut names = Vec::new();
        for entry in fs::read_dir(root.join(dir)).map_err(read_error)? {
            let name = entry.map_err(read_error)?.file_name();
            if Path::new(&name).extension() == Some(OsStr::new("html")) {
                names.push(name.to_string_lossy().into_owned());
            }
        }
        names.sort();
        if !names.iter().any(|name| name == LAYOUT) {
            read(root, &dir.join(LAYOUT))?;
        }
        let mut layouts = Vec::new();
        for name in names {
            let path = dir.join(&name);
            match Template::parse(&read(root, &path)?) {
                Ok(template) => layouts.push((name, template)),
                Err(error) => {
                    let error = PageError::Layout(error);
                    errors.push(ContentError { path, error });
                }
            }
        }
        Ok(Self(layouts))
    }

    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }
}
