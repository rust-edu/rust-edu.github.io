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
| `title` | every file | The page title, shown as its only top-level heading. Quote it when it contains `: `, as in `title: "Campus Tour: Rust Africa"`. |
| `date` | news posts | `YYYY-MM-DD`; the news list is sorted by it, newest first. |
| `authors` | news posts | One name per line; shown in the byline as "A and B on Mar 1, 2027". |
| `draft` | pages | `draft: true` keeps the page off the live site; pull request builds still include it. |
| `template` | `content/news/_index.md` | The layout of the news list. |
| `sort_by` | `content/news/_index.md` | `date`. |
| `redirect_to` | `content/zulip/_index.md` | The address that `/zulip/` forwards to. |

No other keys are used, because the site's planned generator
([Plan 0002](Plan-0002—Own-Markdown-Site-Generator.md)) will read only these.

## Markdown

Use plain Markdown, never HTML:

- headings `##` and `###` (the title is already the page's `#` heading);
- paragraphs, `**bold**`, `*italic*`, `` `code` ``, and `> quotes`;
- lists with `-` or `*` (one marker per list), nested one level by indenting two spaces;
- links `[text](url)` and images `![description of the image](/images/file.png)`.

## Links

- **Another page or file on this site:** a root-relative URL ending in `/` for pages: `/news/campus-tour/`,
  `/workshop/proceedings.pdf`. Relative links such as `proceedings.pdf` or `../news/` break on some pages.
- **A heading:** `/workshop/cfp-2022/#schedule`, or `#schedule` on the same page. A heading's id is its text in
  lowercase with each run of other characters replaced by `-` (`What’s Next?` → `what-s-next`).
- **Another site:** the full `https://` address.
- **Email:** `[contact@rust-edu.org](mailto:contact@rust-edu.org)`.

## Common changes

- **A news post:** add `content/news/<name>.md` with `title`, `date` and `authors`, as in the header example above.
- **A resource:** add one line under the right heading of `content/resources/_index.md`:
  `- [Title](https://example.org/) — Author. One or two sentences on what it is.` (leave out `Author.` when there is
  none).
- **Menu and footer links:** the `navbar` and `external` lists at the end of `config.toml`.

## Proposing a change on GitHub

1. Open the file on github.com and choose the pencil (*Edit this file*), or open the folder and choose
   *Add file → Create new file*. Without write access, GitHub first asks to *Fork this repository*.
2. Edit, and check the *Preview* tab (GitHub shows the header as a table).
3. Choose *Commit changes… → Propose changes → Create pull request*.

An automatic check builds the site for every pull request; a maintainer reviews and merges it.

## Previewing locally (optional)

With [Zola 0.22.1](https://github.com/getzola/zola/releases/tag/v0.22.1) installed (see the [README](../README.md)):

```bash
zola serve
```

Then open http://127.0.0.1:1111; the page reloads on every saved change.
