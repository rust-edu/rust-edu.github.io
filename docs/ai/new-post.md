# LLM specification: drafting a news post for rust-edu.org

This file is written for language models and AI coding agents drafting a news post for <https://rust-edu.org/>.
Human contributors: read [../new-post.md](../new-post.md) instead.

## Output requirements

1. **Single post file:** Create `content/news/<slug>.md`, where `<slug>` matches `^[a-z0-9]+(-[a-z0-9]+)*$` (lowercase
   ASCII letters `a`–`z` and digits `0`–`9` separated by single `-`; no accents, underscores, spaces, or leading or
   trailing `-`). Do not modify any other file, except to add a supplied photo under `static/images/`. Never rename an
   existing published post, because feed readers identify posts by URL (`/news/<slug>/`).
2. **Facts only:** Use only facts provided by the author. Never invent names, dates, locations, quotes, numbers or
   URLs. If `title`, `date`, `authors` or required details are missing from the notes, ask the author before drafting.
3. **Verification:**
   - If you have shell access in this repository, run `make check` and fix every `file:line:col` error until it passes.
   - If you do not have shell access, check every line of the output against the YAML, Markdown and link constraints
     below before returning it.
4. **Delivery:** Output the target path (`content/news/<slug>.md`) and the complete file contents in a single code
   block, and remind the author to declare which AI tool helped and how under *AI acknowledgement* in the pull request.

## YAML header specification

The file must begin on line 1 with `---` and close the header with `---`:

```markdown
---
title: Rust Week 2027
date: 2027-03-01
authors:
  - Jane Doe
---
```

- **Allowed keys** (every other key is rejected by `edu check`):
  - `title` (required): non-blank single-line scalar; rendered as the page's `<h1>`.
  - `date` (required): valid calendar date >= `1900-01-01` formatted strictly as `YYYY-MM-DD`.
  - `authors` (required): block list of non-blank author names, one per line indented with two spaces and `- `.
  - `draft` (optional): `true` (or omit the key when publishing).
- **YAML subset & quoting rules (`edu-yaml`):**
  - Indent with spaces only; tabs are errors.
  - Wrap any `title` or author name in double quotes (`"..."`) if it contains `: ` (colon + space), contains ` #`
    (space + `#`, which starts a YAML comment and truncates unquoted text), ends with `:`, or starts with any YAML
    indicator or reserved character (`&`, `*`, `!`, `[`, `{`, `|`, `>`, `?`, `-`, `%`, `@`, `` ` ``, `,`, `:`, `]`,
    `}`, `'`, `"`).
  - Inside double quotes, only `\"`, `\\`, `\n` and `\t` are valid escapes. Do not use block scalars (`|`, `>`) or
    flow lists (`[...]`).

## Markdown body specification

Write the body immediately after the closing `---` using only the accepted CommonMark subset (`edu-markdown`):

- **Allowed blocks** (separate blocks with a blank line):
  - Headings starting at `##` (`##` and `###`; never `#`, because `title` is `<h1>`). Every heading must contain at
    least one letter or digit so an anchor `id` can be generated.
  - Paragraphs (plain UTF-8 text; write characters like `é`, `—`, `“`, `”`, `’` directly).
  - Lists using `- ` (unordered) or `1. ` (ordered), with a single marker style per list, non-empty text after the
    marker, continuation lines and nested lists indented to the item text's starting column, and nesting depth <= 8.
  - Blockquotes with `> ` (every continuation line inside a quote must keep `>`).
  - Fenced code blocks between lines of ```` ``` ```` (or `~~~`).
- **Allowed inline syntax:**
  - `*italic*` or `_italic_`, `**bold**` or `__bold__`, and `**_bold italic_**` (never `***` or `___`; avoid placing
    `*` or `_` directly against non-ASCII characters). Every delimiter must pair cleanly in the same paragraph or link.
  - Inline code `` `code` ``.
  - Inline links `[text](url)` and images `![alt text](/images/<file>.webp)` (no link titles such as
    `[text](url "title")`, no spaces in `url`, no nested links).
  - Autolinks `<https://example.org/>` and `<user@example.org>`.
  - Backslash escapes (`\*`, `\_`, `\[`, `\<`, `\~`) and only the four XML character references `&amp;`, `&lt;`, `&gt;`
    and `&quot;`.
  - Hard line breaks via two trailing spaces or a trailing `\` before a newline.
- **Rejected syntax (`edu check` fails on all of these):**
  - Raw HTML tags or HTML comments (`<!-- ... -->`).
  - GFM tables, footnotes (`[^1]`), task lists (`- [ ]`), and `~~strikethrough~~`.
  - Horizontal rules (`---`, `***`, `___`), setext headings underlined with `===` or `---`, 4-space indented code
    blocks, and `{#id}` heading attributes.
  - Reference-style links (`[text][1]` or `[1]: https://...`).
  - Bare URLs or bare email addresses outside `<...>` or `[...](...)`.
  - HTML named or numeric character references other than `&amp;`, `&lt;`, `&gt;` and `&quot;` (e.g. `&mdash;`,
    `&nbsp;`, `&#8212;` are errors).
  - Tabs and Unicode noncharacters.

## Link and image constraints

- **Internal page links:** Must be root-relative starting and ending with `/` (e.g. `/news/campus-tour/`,
  `/resources/`, `/workshop/`), and when running in the repository must point to an existing non-draft page under
  `content/`. Never write relative paths (`../news/` or `./post/`).
- **Internal file links:** Root-relative with extension (e.g. `/workshop/proceedings.pdf`).
- **Heading anchors:** `#id` or `/path/page/#id`, where `id` is the heading text in lowercase (retaining letters and
  digits of any script) with runs of other characters collapsed to `-`.
- **External & email links:** Must start with `https://`, `http://` or `mailto:`.
- **Photos:** Stored in `static/images/<name>.webp` (960 px wide, 16:9 aspect ratio, WebP <= 90 KB) and referenced as
  `![description](/images/<name>.webp)`.

## Reference example (`content/news/rust-week-2027.md`)

```markdown
---
title: Rust Week 2027
date: 2027-03-01
authors:
  - Jane Doe
---
Rust Week 2027 takes place in **Lima** from March 1 to 5, with talks for teachers and a workshop for students.

## What to expect

- Talks on teaching Rust in a first course
- A workshop, for which you need only a laptop

Read the [call for participation](/news/call-for-participation/), and write to
[contact@rust-edu.org](mailto:contact@rust-edu.org) with any question.
```
