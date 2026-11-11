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
| `/zulip/` | `urls.yaml` (`redirects`, forwarding to the Zulip chat) |

The file name becomes the URL: `content/news/rust-week-2027.md` is served at `/news/rust-week-2027/`. Name files in
lowercase kebab-case. Images go in `static/images/` and are linked as `/images/<file>`; other files (PDFs, LaTeX)
sit next to the page that links them.

Besides its pages, the site publishes images (`.png`, `.jpg`, `.jpeg`, `.gif`, `.webp`, `.avif`, `.ico` and
`.svg`), `.pdf` and `.txt` files, LaTeX files (`.tex`, `.cls` and `.bib`), stylesheets (`.css`) and
`static/CNAME`. The automatic check stops at any other file, such as `.html` or `.js`, since a browser would run
the scripts in it as part of the site. It also stops at an SVG image with a script, an attribute such as
`onload`, a `javascript:` address or HTML in it, and at one not saved as UTF-8.

The automatic check stops at a symbolic link, which could bring a file from outside the site into it. Files and
folders whose names start with `.`, such as `.DS_Store`, are not part of the site.

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
| `title` | every file but a redirect | The page title, shown as its only top-level heading; the home page's heading is the words of the `description` of `site.yaml`, styled in `layouts/home.html`, instead. Quote it when it contains `: `, as in `title: "Campus Tour: Rust Africa"`. It cannot be empty or only spaces. |
| `date` | news posts | `YYYY-MM-DD`, a day of the calendar from 1900 on; the news list and the feeds are sorted by it, newest first, so every news post needs one. The feeds give it as the post's time, at midnight UTC. |
| `authors` | news posts | One name per line, none of them empty or only spaces; shown in the byline as "A and B on Mar 1, 2027". |
| `draft` | every file but the home page | `draft: true` keeps the page off the live site, and in an `_index.md` everything in its folder too; pull request builds still include it at its address, but the news list leaves it out. |
| `template` | `content/news/_index.md` | The layout of the news list, a file of `layouts/`. |
| `sort_by` | `content/news/_index.md` | `date`, the only value. The pages of a section with it that are not drafts are the posts of the feeds, `/atom.xml` and `/rss.xml`. |
| `redirect_to` | an `_index.md` | The full address, starting with `https://`, that the section forwards to (prefer `redirects` in `urls.yaml`, which needs no folder in `content/`). |

`title`, `template` and `draft` go in any file, `date` and `authors` only in pages, and `sort_by` and `redirect_to`
only in `_index.md` files. The automatic check (see below) stops at any other key, because the site's generator
reads only these.

## Markdown

Use plain Markdown, never HTML:

- headings `##` and `###` (the title, or on the home page the site's description, is already the `#` heading);
- paragraphs, `**bold**`, `*italic*`, `` `code` ``, and `> quotes`;
- lists with `-` or `*`, or numbered `1.`, one marker per list; an item's further lines and nested lists are
  indented to the item's text;
- links `[text](url)`, images `![description of the image](/images/file.webp)`, and a bare address as
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
- **Another site:** the full `https://` address; links to other sites (and redirects to them, such as `/zulip/`)
  open in a new tab.
- **Email:** `[contact@rust-edu.org](mailto:contact@rust-edu.org)`.

The automatic check follows every link to a page or file of the site, `site.yaml`'s too, and stops at one that goes
nowhere: a page, file or id that the site does not have, a page's URL without its `/` at the end, a relative link,
a link from a page to a draft, or an address that starts with something other than `https:`, `http:` or `mailto:`.
It does not check links to other sites.

## The news feeds

`/atom.xml` and `/rss.xml` are feeds of the news posts for feed readers, in the Atom and the RSS format: the same
posts, newest first, each with its title, date, authors and whole text. Their name and description are the `title`
and `description` of `site.yaml`, which a site with news posts needs. Every page with the site's layout names both
for feed readers, the Atom feed first, and has an orange button for each feed at the end of its menu. On a narrow
screen one button with the feed symbol takes their place and opens a short menu with both. `/news/` links to both
too. The build writes the feeds again each time, from the posts, so a new post needs nothing more than its file.
Links in a post need nothing special: the feeds turn the links to the site and to ids into full addresses. A feed
reader knows a post by its address, so keep the file name of a post once it is published.

## The page for language models

`/llms.txt` is the site in brief for language models, in the Markdown that <https://llmstxt.org/> proposes: the
`title` and `description` of `site.yaml`, a link to each page with its title, a link to each news post with its date
and authors, newest first, and the two feeds, which hold the whole text of every post. It leaves out the redirects and
the drafts. The build writes it again each time, so a new page or post needs nothing more than its file.

## Common changes

- **A news post:** add `content/news/<name>.md` with `title`, `date` and `authors`, as in the header example above;
  it goes into the feeds too.
- **A photo:** save it in `static/images/`, 960 pixels wide, in the 16:9 shape, as WebP of at most 90 KB. It shows at
  the full width of the text, so a narrower image is stretched, and another shape moves the text below it when it
  loads, a portrait photo by a third of a phone's screen or more.
- **A resource:** add one line under the right heading of `content/resources/_index.md`:
  `- [Title](https://example.org/) — Author. One or two sentences on what it is.` (leave out `Author.` when there is
  none).
- **Menu, next steps and footer links:** the `menu`, `hero` and `footer` lists of `site.yaml`; the first link of
  `hero` is the button on the home page.
- **The site's description:** the `description` of `site.yaml`, which the home page's title, search engines, the
  feeds and `/llms.txt` show, and the same words in the heading of `layouts/home.html`, whose three parts `static/style.css` styles.
- **Moving or removing a page or file:** other sites may link to it, so its address is under `keep` in `urls.yaml`,
  and the check stops while that address goes nowhere. Move the old address of a page to `redirects` in `urls.yaml`
  (with `from` and `to`), or delete its line from `keep` on purpose.

## Proposing a change on GitHub

1. Open the file on github.com and choose the pencil (*Edit this file*), or open the folder and choose
   *Add file → Create new file*. Without write access, GitHub first asks to *Fork this repository*.
2. Edit, and check the *Preview* tab (GitHub shows the header as a table).
3. Choose *Commit changes… → Propose changes → Create pull request*.

An automatic check builds the site for every pull request and names the file, line and column of any mistake in a
page's header or text, which the pull request's *Files changed* tab also shows on that line; a maintainer reviews
and merges it. On the first pull request of someone new to the repository, the check waits until a maintainer
lets it run.

## Previewing locally (optional)

With [Docker](https://docs.docker.com/get-started/get-docker/) installed (see the [README](../README.md)):

```bash
docker compose up
```

Then open http://127.0.0.1:1111, where drafts show too, each at its own address; the news list leaves them out. After
saving a change, reload the page: it shows the site built again or, when a page has a mistake, the message of the
automatic check, with its file, line and column.
