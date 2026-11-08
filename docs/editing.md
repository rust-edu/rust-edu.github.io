# Editing rust-edu.org

Every page of the site is one Markdown file under `content/`. A change merged into `main` is live a few minutes
later; no HTML, templates or tools are needed for content.

## Where each page lives

| URL | File |
|---|---|
| `/` | `content/_index.md` |
| `/news/` | `content/news/_index.md` (the list itself; it has no text) |
| `/news/<name>/` | `content/news/<name>.md` |
| `/resources/` | `content/resources/_index.md` |
| `/workshop/` and its pages | `content/workshop/` |
| `/zulip/` | `content/zulip/_index.md` (only a redirect to the Zulip chat) |

The file name becomes the URL: `content/news/rust-week-2027.md` is served at `/news/rust-week-2027/`. Name files in
lowercase kebab-case. Images go in `static/images/` and are linked as `/images/<file>`; other files (PDFs, LaTeX)
sit next to the page that links them.

## The header

Each file starts with a short YAML header between two `---` lines:

```markdown
---
title: Rust Week 2027
date: 2027-03-01
authors:
  - Jane Doe
---
The text of the page starts here.
```

| Key | Used in | Meaning |
|---|---|---|
| `title` | every file but a redirect | The page title, shown as its only top-level heading. Quote it when it contains `: `, as in `title: "Campus Tour: Rust Africa"`. |
| `date` | news posts | `YYYY-MM-DD`, a day of the calendar; the news list is sorted by it, newest first, so every news post needs one. |
| `authors` | news posts | One name per line; shown in the byline as "A and B on Mar 1, 2027". |
| `draft` | every file but the home page | `draft: true` keeps the page off the live site, and in an `_index.md` everything in its folder too; pull request builds still include it. |
| `template` | `content/news/_index.md` | The layout of the news list, a file of `layouts/`. |
| `sort_by` | `content/news/_index.md` | `date`, the only value. |
| `redirect_to` | `content/zulip/_index.md` | The full address, starting with `https://`, that `/zulip/` forwards to. |

`title`, `template` and `draft` go in any file, `date` and `authors` only in pages, and `sort_by` and `redirect_to`
only in `_index.md` files. The automatic check (see below) stops at any other key, because the site's generator
reads only these.

## Markdown

Use plain Markdown, never HTML:

- headings `##` and `###` (the title is already the page's `#` heading);
- paragraphs, `**bold**`, `*italic*`, `` `code` ``, and `> quotes`;
- lists with `-` or `*`, or numbered `1.`, one marker per list; an item's further lines and nested lists are
  indented to the item's text;
- links `[text](url)`, images `![description of the image](/images/file.png)`, and a bare address as
  `<https://example.org>`;
- code on lines of its own, between two lines of three backticks (```` ``` ````);
- a line break inside a paragraph: two spaces or a `\` at the end of the line.

Quotes and apostrophes show as typed: `"` and `'` stay straight, and `“`, `”` and `’` are curly. Type other
characters as they are too, such as `é` or `—`. To show a character that Markdown reads as syntax, put a `\` before
it: `\*`, `\_`, `\[`, `\<`, `\~`; `&amp;`, `&lt;`, `&gt;` and `&quot;` work as well.

The automatic check (see below) stops at what GitHub's preview and the site would show differently, with its line,
column and a hint: HTML, tables, footnotes, task lists, `~~strikethrough~~`, `***` (write bold italic as
`**_text_**`) and other `*` or `_` that do not pair up, `{#id}` after a heading, horizontal rules, headings
underlined with `===`, indented code, and web or email addresses not written as `<…>` or as a link.

## Links

- **Another page or file on this site:** a root-relative URL ending in `/` for pages: `/news/campus-tour/`,
  `/workshop/proceedings.pdf`. Relative links such as `proceedings.pdf` or `../news/` break on some pages.
- **A heading:** `/workshop/cfp-2022/#schedule`, or `#schedule` on the same page. A heading's id is its text in
  lowercase, letters of any language included, with each run of other characters replaced by `-`
  (`What’s Next?` → `what-s-next`, `Taller en Perú` → `taller-en-perú`).
- **Another site:** the full `https://` address.
- **Email:** `[contact@rust-edu.org](mailto:contact@rust-edu.org)`.

The automatic check follows every link to a page or file of the site, the menu's too, and stops at one that goes
nowhere: a page, file or id that the site does not have, a page's URL without its `/` at the end, a relative link,
a link from a page to a draft, or an address that starts with something other than `https:`, `http:` or `mailto:`.
It does not check links to other sites.

## Common changes

- **A news post:** add `content/news/<name>.md` with `title`, `date` and `authors`, as in the header example above.
- **A resource:** add one line under the right heading of `content/resources/_index.md`:
  `- [Title](https://example.org/) — Author. One or two sentences on what it is.` (leave out `Author.` when there is
  none).
- **Menu and footer links:** the `menu` and `footer` lists of `site.yaml`.
- **Moving or removing a page or file:** other sites may link to it, so its address is in `urls.txt`, and the check
  stops while that address goes nowhere. Delete its line there on purpose, or keep the old address of a page with an
  `_index.md` at it that has `redirect_to`.

## Proposing a change on GitHub

1. Open the file on github.com and choose the pencil (*Edit this file*), or open the folder and choose
   *Add file → Create new file*. Without write access, GitHub first asks to *Fork this repository*.
2. Edit, and check the *Preview* tab (GitHub shows the header as a table).
3. Choose *Commit changes… → Propose changes → Create pull request*.

An automatic check builds the site for every pull request and names the file, line and column of any mistake in a
page's header or text; a maintainer reviews and merges it.

## Previewing locally (optional)

With [Docker](https://docs.docker.com/get-started/get-docker/) installed (see the [README](../README.md)):

```bash
docker compose up
```

Then open http://127.0.0.1:1111, where drafts show too. After saving a change, reload the page: it shows the site
built again or, when a page has a mistake, the message of the automatic check, with its file, line and column.
