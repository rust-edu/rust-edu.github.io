//! The scan of an SVG image before the site publishes it. A browser that opens an SVG image at its
//! own address runs the image's scripts as if it were a page of the site, so the scan refuses each
//! way for an image to run a script or to show HTML, and any markup that could hide one from it.

use std::fmt;

use edu_yaml::Pos;

/// The XML namespace of HTML, which would show HTML inside an image.
const XHTML: &str = "http://www.w3.org/1999/xhtml";

/// The elements that run a script or show another document, by their name without a prefix.
const ELEMENTS: [&str; 7] = [
    "embed",
    "foreignObject",
    "handler",
    "iframe",
    "listener",
    "object",
    "script",
];

/// XML's own references, the only ones an image may have.
const REFERENCES: [&str; 5] = ["&amp;", "&lt;", "&gt;", "&quot;", "&apos;"];

/// Why the site does not publish an SVG image.
#[derive(Debug, PartialEq, Eq)]
pub enum SvgError {
    /// Not UTF-8 text, or text with a NUL character.
    NotText,
    /// Markup that the scan cannot follow, such as a tag without its `>`; a browser would not show
    /// the image either.
    Malformed,
    /// An element that runs a script or shows another document, such as `script` or
    /// `foreignObject`.
    Element(String),
    /// An attribute that runs a script, such as `onload`, or an animation of one.
    Event(String),
    /// A `javascript:` address, in any letter case.
    Javascript,
    /// The XML namespace of HTML.
    Html,
    /// A reference other than `&amp;`, `&lt;`, `&gt;`, `&quot;` and `&apos;`, which could spell any
    /// of the above.
    Reference,
    /// A processing instruction other than the XML declaration, such as a stylesheet.
    Instruction,
    /// A `<!DOCTYPE>` with declarations between `[` and `]`, which the scan does not read.
    Declarations,
}

/// Scans `text`, an SVG image, and gives the first thing in it that the site does not publish,
/// with where it starts.
pub(crate) fn scan(text: &str) -> Result<(), (Pos, SvgError)> {
    let at = |offset| pos(text, offset);
    if let Some(offset) = text.find('\0') {
        return Err((at(offset), SvgError::NotText));
    }
    let mut scanner = Scanner { text, offset: 0 };
    scanner
        .markup()
        .map_err(|(offset, error)| (at(offset), error))?;
    // A browser runs a `javascript:` address in many attributes and in any letter case, and the
    // scan has refused every reference that could spell one.
    match text.to_ascii_lowercase().find("javascript:") {
        Some(offset) => Err((at(offset), SvgError::Javascript)),
        None => Ok(()),
    }
}

/// The first error of a scan, at a byte offset of the image.
type Scan = Result<(), (usize, SvgError)>;

/// A scan of the markup of an image, at a byte offset that only ever moves forward.
struct Scanner<'a> {
    text: &'a str,
    offset: usize,
}

impl<'a> Scanner<'a> {
    /// The text from the offset on.
    fn rest(&self) -> &'a str {
        self.text.get(self.offset..).unwrap_or_default()
    }

    /// Reads the image's text and markup to its end.
    fn markup(&mut self) -> Scan {
        while let Some(next) = self.rest().find(['<', '&']) {
            self.offset += next;
            let rest = self.rest();
            if rest.starts_with('&') {
                self.reference()?;
            } else if rest.starts_with("<!--") {
                self.past(4, "-->")?;
            } else if rest.starts_with("<![CDATA[") {
                self.past(9, "]]>")?;
            } else if rest.starts_with("<!DOCTYPE") {
                self.doctype()?;
            } else if rest.starts_with("<?") {
                self.instruction()?;
            } else if rest.starts_with("</") {
                self.past(2, ">")?;
            } else {
                self.tag()?;
            }
        }
        Ok(())
    }

    /// Moves past the first `end` after the `skip` bytes at the offset; there is none in markup
    /// that does not close.
    fn past(&mut self, skip: usize, end: &str) -> Scan {
        let found = self.rest().get(skip..).and_then(|rest| rest.find(end));
        let Some(at) = found else {
            return Err((self.offset, SvgError::Malformed));
        };
        self.offset += skip + at + end.len();
        Ok(())
    }

    /// Moves past one of XML's own references.
    fn reference(&mut self) -> Scan {
        let rest = self.rest();
        let Some(reference) = REFERENCES.iter().find(|&&known| rest.starts_with(known)) else {
            return Err((self.offset, SvgError::Reference));
        };
        self.offset += reference.len();
        Ok(())
    }

    /// Moves past a `<!DOCTYPE …>` without declarations, whose names may be quoted.
    fn doctype(&mut self) -> Scan {
        let start = self.offset;
        let mut quote = None;
        for (at, c) in self.rest().char_indices() {
            match quote {
                Some(open) if c == open => quote = None,
                Some(_) => {}
                None => match c {
                    '"' | '\'' => quote = Some(c),
                    '[' => return Err((start + at, SvgError::Declarations)),
                    '>' => {
                        self.offset = start + at + 1;
                        return Ok(());
                    }
                    _ => {}
                },
            }
        }
        Err((start, SvgError::Malformed))
    }

    /// Moves past the XML declaration, `<?xml …?>`, the only processing instruction.
    fn instruction(&mut self) -> Scan {
        let target = self.rest().get(2..).unwrap_or_default();
        let end = target
            .find(|c: char| c.is_whitespace() || c == '?')
            .unwrap_or(target.len());
        if target.get(..end) != Some("xml") {
            return Err((self.offset, SvgError::Instruction));
        }
        self.past(2, "?>")
    }

    /// Moves past a start tag, `<name attribute="value" …>` or `<… />`, after checking its name
    /// and each of its attributes.
    fn tag(&mut self) -> Scan {
        let start = self.offset;
        self.offset += 1;
        let name = self.name();
        if !name.starts_with(|c: char| c.is_alphabetic() || c == '_' || c == ':') {
            return Err((start, SvgError::Malformed));
        }
        let local = local(name);
        if ELEMENTS
            .iter()
            .any(|element| local.eq_ignore_ascii_case(element))
        {
            return Err((start, SvgError::Element(name.to_owned())));
        }
        loop {
            self.skip_whitespace();
            let rest = self.rest();
            if rest.is_empty() {
                return Err((start, SvgError::Malformed));
            }
            if rest.starts_with("/>") {
                self.offset += 2;
                return Ok(());
            }
            if rest.starts_with('>') {
                self.offset += 1;
                return Ok(());
            }
            self.attribute()?;
        }
    }

    /// Moves past an attribute, `name="value"` or `name='value'`, after checking it.
    fn attribute(&mut self) -> Scan {
        let start = self.offset;
        let malformed = Err((start, SvgError::Malformed));
        let name = self.name();
        if name.is_empty() {
            return malformed;
        }
        if is_event(name) {
            return Err((start, SvgError::Event(name.to_owned())));
        }
        self.skip_whitespace();
        if !self.rest().starts_with('=') {
            return malformed;
        }
        self.offset += 1;
        self.skip_whitespace();
        let rest = self.rest();
        let Some(quote) = rest.chars().next().filter(|c| matches!(c, '"' | '\'')) else {
            return malformed;
        };
        let Some((value, _)) = rest.get(1..).and_then(|rest| rest.split_once(quote)) else {
            return malformed;
        };
        let value_at = self.offset + 1;
        for (at, _) in value.match_indices('&') {
            let reference = value.get(at..).unwrap_or_default();
            if !REFERENCES.iter().any(|&known| reference.starts_with(known)) {
                return Err((value_at + at, SvgError::Reference));
            }
        }
        if (name == "xmlns" || name.starts_with("xmlns:")) && value.trim() == XHTML {
            return Err((start, SvgError::Html));
        }
        // An animation that sets `onclick`, say, would run its value as a script.
        if local(name).eq_ignore_ascii_case("attributeName") && is_event(value.trim()) {
            return Err((value_at, SvgError::Event(value.trim().to_owned())));
        }
        self.offset = value_at + value.len() + 1;
        Ok(())
    }

    /// Moves past a name at the offset, which ends before a space, `/`, `>`, `=` or a quote.
    fn name(&mut self) -> &'a str {
        let rest = self.rest();
        let end = rest
            .find(|c: char| c.is_whitespace() || "/>=\"'<&".contains(c))
            .unwrap_or(rest.len());
        let name = rest.get(..end).unwrap_or_default();
        self.offset += name.len();
        name
    }

    /// Moves past XML's white space: spaces, tabs and line ends.
    fn skip_whitespace(&mut self) {
        let rest = self.rest();
        let kept = rest.trim_start_matches([' ', '\t', '\r', '\n']);
        self.offset += rest.len() - kept.len();
    }
}

/// The name without its prefix: `script` in `svg:script`.
fn local(name: &str) -> &str {
    name.rsplit_once(':').map_or(name, |(_, local)| local)
}

/// Whether `name`, without its prefix, starts with `on` in any letter case, as `onload` does.
fn is_event(name: &str) -> bool {
    local(name)
        .get(..2)
        .is_some_and(|start| start.eq_ignore_ascii_case("on"))
}

/// The line and the column, in characters, of the byte `offset` of `text`.
fn pos(text: &str, offset: usize) -> Pos {
    let before = text.get(..offset).unwrap_or(text);
    let line_start = before.rfind('\n').map_or(0, |at| at + 1);
    let line = before.matches('\n').count() + 1;
    let col = before.get(line_start..).unwrap_or_default().chars().count() + 1;
    Pos { line, col }
}

impl fmt::Display for SvgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotText => {
                f.write_str("an SVG image is UTF-8 text without NUL characters; save it as UTF-8")
            }
            Self::Malformed => f.write_str(
                "the check cannot read this markup; an SVG image is XML whose tags and comments \
                 close, with each attribute's value in quotes",
            ),
            Self::Element(name) => write!(
                f,
                "`<{name}>` can run a script or show a web page in the image; remove it"
            ),
            Self::Event(name) => write!(f, "`{name}` runs a script; remove it"),
            Self::Javascript => f.write_str("a `javascript:` address runs a script; remove it"),
            Self::Html => f.write_str("the XHTML namespace would show HTML in the image; remove it"),
            Self::Reference => f.write_str(
                "the only references are `&amp;`, `&lt;`, `&gt;`, `&quot;` and `&apos;`; type the \
                 character itself",
            ),
            Self::Instruction => f.write_str(
                "the only processing instruction is the XML declaration, `<?xml …?>`; remove this one",
            ),
            Self::Declarations => f.write_str(
                "a `<!DOCTYPE>` with declarations between `[` and `]` could hide a script from the \
                 check; remove them",
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An image as drawing programs write one, with what could look like a script but is not.
    const IMAGE: &str = r##"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN" "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd">
<!-- Drawn by hand; a <script> in a comment does nothing. -->
<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink='http://www.w3.org/1999/xlink'
     xmlns:dc="http://purl.org/dc/elements/1.1/" viewBox="0 0 10 10">
  <metadata><dc:description>A description &amp; a title, once = twice</dc:description></metadata>
  <style><![CDATA[ rect { fill: #0b7261; } /* <script> */ ]]></style>
  <a href="https://rust-edu.org/" xlink:href="#café"><rect id="café" width="10" height="10"/></a>
  <animate attributeName="opacity" values="0;1" dur="1s"/>
  <text x="1" y="5">Rust Edu &lt;3 JavaScript, online</text>
</svg>
"##;

    fn error(svg: &str) -> Option<(usize, usize, SvgError)> {
        scan(svg)
            .err()
            .map(|(pos, error)| (pos.line, pos.col, error))
    }

    #[test]
    fn an_image_of_shapes_text_links_and_styles_passes() {
        assert_eq!(error(IMAGE), None);
        assert_eq!(error(""), None);
        assert_eq!(error("<svg xmlns=\"http://www.w3.org/2000/svg\"/>"), None);
    }

    #[test]
    fn each_way_to_run_a_script_is_refused_where_it_starts() {
        let element = |name: &str| SvgError::Element(name.to_owned());
        let event = |name: &str| SvgError::Event(name.to_owned());
        let html = format!("<svg xmlns='{XHTML}'>");
        let spaced = format!("<g xmlns:h=' {XHTML} '>");
        let cases = [
            ("<svg><script/>", 1, 6, element("script")),
            ("<svg>\n  <SCRIPT/>", 2, 3, element("SCRIPT")),
            ("<svg><x:script/>", 1, 6, element("x:script")),
            ("<g><foreignObject>", 1, 4, element("foreignObject")),
            ("<svg><iframe/>", 1, 6, element("iframe")),
            ("<svg onload='a'>", 1, 6, event("onload")),
            ("<svg\n OnClick='a'>", 2, 2, event("OnClick")),
            ("<svg ev:onfocusin='a'>", 1, 6, event("ev:onfocusin")),
            ("<set attributeName=' onclick'/>", 1, 21, event("onclick")),
            ("<a href='JavaScript:x'>", 1, 10, SvgError::Javascript),
            ("<!-- javascript: -->", 1, 6, SvgError::Javascript),
            (html.as_str(), 1, 6, SvgError::Html),
            (spaced.as_str(), 1, 4, SvgError::Html),
        ];
        for (svg, line, col, expected) in cases {
            assert_eq!(error(svg), Some((line, col, expected)), "{svg}");
        }
    }

    #[test]
    fn a_reference_that_could_spell_a_script_is_refused() {
        let cases = [
            ("<svg><a href=\"j&#97;vascript:x\">", 1, 16),
            ("<svg><a href='&#x6A;avascript:x'>", 1, 15),
            ("<svg>&nbsp;</svg>", 1, 6),
            ("<svg>a & b</svg>", 1, 8),
            ("<svg a=\"&amp;&b;\">", 1, 14),
        ];
        for (svg, line, col) in cases {
            assert_eq!(error(svg), Some((line, col, SvgError::Reference)), "{svg}");
        }
        let known = "<svg a=\"&amp;&lt;&gt;&quot;&apos;\">&amp;&lt;&gt;&quot;&apos;</svg>";
        assert_eq!(error(known), None);
    }

    #[test]
    fn markup_that_could_hide_a_script_from_the_scan_is_refused() {
        // The entity's text ends a `]>` early for a careless scan, and starts a comment that hides
        // the `onload` from it.
        let hidden = "<!DOCTYPE svg [ <!ENTITY a \"]> <!--\"> ]><svg onload=\"alert(1)\"/><!-- -->";
        let cases = [
            (hidden, 1, 15, SvgError::Declarations),
            ("<?xml-stylesheet href='a'?>", 1, 1, SvgError::Instruction),
            ("<svg/>\n<?php echo 1 ?>", 2, 1, SvgError::Instruction),
            ("<svg>\0</svg>", 1, 6, SvgError::NotText),
        ];
        for (svg, line, col, expected) in cases {
            assert_eq!(error(svg), Some((line, col, expected)), "{svg}");
        }
        let quoted = "<!DOCTYPE svg PUBLIC \"a [b]\" 'c>'><svg/>";
        assert_eq!(error(quoted), None);
    }

    #[test]
    fn markup_that_does_not_close_or_is_not_xml_is_refused_where_it_starts() {
        let cases = [
            ("<svg", 1, 1),
            ("<svg a=\"1", 1, 6),
            ("<svg a=1>", 1, 6),
            ("<svg a>", 1, 6),
            ("<svg/onload=1>", 1, 5),
            ("<svg>\n<!-- no end", 2, 1),
            ("<![CDATA[ a", 1, 1),
            ("<!ENTITY a \"b\">", 1, 1),
            ("< svg>", 1, 1),
            ("<svg></svg", 1, 6),
            ("<!DOCTYPE svg", 1, 1),
        ];
        for (svg, line, col) in cases {
            assert_eq!(error(svg), Some((line, col, SvgError::Malformed)), "{svg}");
        }
    }

    #[test]
    fn a_damaged_image_is_never_a_panic() {
        let sample: Vec<char> = IMAGE.chars().collect();
        let damage = [
            '<', '>', '"', '\'', '&', ';', '!', '?', '-', '[', ']', ':', '=', ' ', 'é',
        ];
        // xorshift64 with a fixed seed: every run tries the same 5,000 images, each with two random
        // characters replaced, inserted or removed.
        let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
        for _ in 0..5_000 {
            let mut text = sample.clone();
            for _ in 0..2 {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                let [low, high, how, what, ..] = state.to_le_bytes();
                let at = usize::from(u16::from_le_bytes([low, high])) % text.len();
                let c = damage[usize::from(what) % damage.len()];
                match how % 3 {
                    0 => text[at] = c,
                    1 => text.insert(at, c),
                    _ => _ = text.remove(at),
                }
            }
            let text: String = text.into_iter().collect();
            let _ = scan(&text);
        }
    }
}
