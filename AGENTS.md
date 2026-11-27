# Instructions for AI coding agents

This file is for AI coding agents. It is at the root of the repository because that is where coding agents look for it.
People: start at [README.md](README.md); you do not need this file.

This repository is the source of <https://rust-edu.org/>: Markdown content rendered by a custom static site generator
written in Rust using only the standard library.

- **Self-contained agent docs:** This file (plus [docs/ai/new-post.md](docs/ai/new-post.md) when drafting a news post)
  gives agents the repository map, commands and exact parser/code rules in machine-oriented form so you do not need to
  read the human tutorials first.
- **Source of truth:** `.claude/CLAUDE.md` and `.gemini/settings.json` only point here; put agent instructions in this
  file. Rules for everyone belong in the human docs ([README.md](README.md), [docs/editing.md](docs/editing.md),
  [generator guide](generator/docs/generator.md)) first; if this file and the human docs or code ever disagree, the human docs
  and code win.
- **Design history is not instructions:** The [design index](docs/design/README.md) lists the historical plans and research notes. Do not execute any
  plan unless explicitly asked, and treat the code and current docs as right wherever a historical note disagrees.

## Repository layout

- `content/`: Markdown pages (`_index.md` for `/` and section indexes, `<slug>.md` for pages) and co-located static
  assets (PDFs, LaTeX).
- `layouts/`: HTML templates (`layout.html` outer wrapper + `home.html`, `page.html`, `section.html`, `news.html`,
  `404.html`). `layout.html` holds the theme toggle's two inline scripts, the site's only JS.
- `static/`: `style.css` (plain CSS with light/dark token blocks), images (`static/images/`), `favicon.svg`, `CNAME`.
- `site.yaml`: `base_url`, `title`, `description`, and `menu`, `hero` and `footer` link lists.
- `urls.yaml`: `keep` (preserved public paths checked by `edu check`) and `redirects` (`from` / `to` pairs).
- `crates/`: Workspace of four `std`-only crates: `yaml` (`edu-yaml`), `markdown` (`edu-markdown`), `template`
  (`edu-template`) and `site` (`edu-site`, binary `edu` with `check`, `build`, `serve`).
- `generator/docs/`: The generator guide, design plans and supporting research.
- `docs/`: Human guides (`new-post.md`, `editing.md`), `docs/ai/` (AI policy and
  [docs/ai/new-post.md](docs/ai/new-post.md)), and `docs/design/` (historical plans and research notes).
- `public/`, `site/`, `preview/`: Build outputs ignored by git. **Never edit them.**

## Commands and workflow

- `make check`: Runs `make guards`, `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`,
  `cargo run --locked -p edu-site -- check`, and `cargo test --workspace --locked`. **Must pass before a change is done.**
- `make guards`: Verifies `Cargo.lock` contains only our four crates with no external `source`, `Dockerfile` has
  `FROM check AS live`, and `Dockerfile` + `rust-toolchain.toml` pin the same Rust version.
- `make ci` / `make preview`: Runs the full PR check (including reproducibility diff of two builds) and draft preview
  build inside the digest-pinned Docker image.
- `make serve`: Runs `edu serve --drafts` at `http://127.0.0.1:1111` until stopped; start it in the background.
- **Fix causes, never weaken checks:** `edu check` and `make guards` report `file:line:col: error: …` with hints. Fix
  the underlying mistake; never weaken a check, lint or test to make it pass.
- **Branch & PR workflow:** Pushing to `main` deploys the live site. Make every change on a branch and propose a pull
  request structured with the headings of `.github/pull_request_template.md` (`## What changes`, `## Checks`,
  `## AI acknowledgement`), stating under *AI acknowledgement* which AI tools and models helped and how.

## Pages, YAML and Markdown

- **News posts:** Follow [docs/ai/new-post.md](docs/ai/new-post.md).
- **Slugs & URLs:** `content/_index.md` -> `/`, `content/<sec>/_index.md` -> `/<sec>/`, `content/<sec>/<slug>.md` ->
  `/<sec>/<slug>/`. Every folder name and `<slug>` must match `^[a-z0-9]+(-[a-z0-9]+)*$` (lowercase ASCII letters and
  digits joined by single `-`).
- **Allowed non-page files (`content/` and `static/`):** `.avif`, `.bib`, `.cls`, `.css`, `.gif`, `.ico`, `.jpeg`,
  `.jpg`, `.pdf`, `.png`, `.svg` (valid UTF-8 XML without `<script>`, event attributes, `javascript:` URLs or HTML),
  `.tex`, `.txt`, `.webp`, plus `static/CNAME`. No `.html`, `.js`, extensionless files or symbolic links; dot-files are
  ignored. Photos in `static/images/`: 960 px wide, 16:9, WebP <= 90 KB.
- **YAML subset (`edu-yaml`):** Page headers start at line 1 with `---` and close with `---`. Block maps and `- ` lists
  only; spaces only (no tabs). Double-quote any scalar that contains `: ` or ` #` (starts a comment), ends with `:`, or
  starts with `` & * ! [ { | > ? - % @ ` , : ] } ' " ``. Double-quote escapes: `\"`, `\\`, `\n`, `\t` only.
- **Allowed frontmatter keys** (any other key fails `edu check`):
  - Any page or `_index.md`: `title` (non-blank; required unless `redirect_to` is set), `template` (file in
    `layouts/`), `draft` (`true` or `false`; `draft: true` forbidden on `content/_index.md`).
  - Content pages only (not `_index.md`): `date` (`YYYY-MM-DD`, valid calendar date >= `1900-01-01`; required inside
    `sort_by: date` sections), `authors` (list of non-blank strings).
  - `_index.md` only: `sort_by` (`date` only; non-draft pages feed `/atom.xml` and `/rss.xml`), `redirect_to`
    (`https://` or `http://`; prefer `redirects` in `urls.yaml`).
- **Markdown subset (`edu-markdown`):** Start headings at `##` (`title` is `<h1>`). Allowed: paragraphs, `##`–`######`
  headings (with >= 1 letter/digit), ```` ``` ```` / `~~~` fenced code, `>` quotes, `-`/`*`/`+` or `1.` lists (one
  marker per list, continuation/nested blocks aligned to item text column, depth <= 8), `*`/`_` italic, `**`/`__` bold
  (`**_text_**` for bold-italic, never `***`), `` `code` ``, `[text](url)` (no titles, no spaces in URL),
  `![alt](url)`, `<url>`/`<email>` autolinks, `\` escapes, `&amp;`/`&lt;`/`&gt;`/`&quot;`, hard breaks (`  ` or `\`).
  Rejected: HTML/comments, tables, footnotes, task lists, `~~strikethrough~~`, horizontal rules, setext headings,
  4-space indented code, `{#id}`, reference links, bare URLs/emails, other HTML entities, tabs, noncharacters.
- **Links & `urls.yaml`:** Internal page links must start and end with `/` (`/news/campus-tour/`) and resolve to a
  non-draft page; file links use `/path/file.ext`; heading ids are lowercase letters/digits (any script) with other
  runs collapsed to `-`; external links start with `https://`, `http://` or `mailto:`; never use relative links. Before
  moving or removing any page or file, check `urls.yaml`: move old page URLs to `redirects` (`from` / `to`) or
  deliberately remove from `keep`.

## Generator (`crates/`)

- **Standard library only.** Do not add any external or extra workspace crate (`Cargo.lock` lists only `edu-markdown`,
  `edu-site`, `edu-template`, `edu-yaml`).
- **No `unsafe`** (`unsafe_code = "forbid"`).
- **No panics on input.** Return `Result` errors; never call `unwrap`, `expect` or `panic!` outside tests.
- **Strict parsers.** Reject anything outside the supported YAML, Markdown or template subset with `file:line:col` and
  a hint; never skip or guess at unknown syntax.
- **Enums over strings.** Model closed sets as enums and match every variant exhaustively. Do not match on marker
  strings or re-scan already parsed bytes, and give error kinds dedicated enum variants rather than `&'static str`
  tags. Use `_` wildcard arms only when matching open input text being parsed.
- **Stable Markdown output.** Update `crates/markdown/tests/golden/` only when rendered HTML is intentionally changed.
- **Tests with each change**, in the crate's `tests/` folder and `crates/site/tests/sites/` fixture sites.

## Build and deploy

- Keep `FROM check AS live` in `Dockerfile`.
- Update `rust-toolchain.toml` and the `FROM rust:<version>-...` tag + digest in `Dockerfile` together.
- Add new checks as targets in `Makefile`, not inline in `.github/workflows/ci.yml`, so they run locally too.
- Add any new top-level file or folder needed by the Docker build to both `.dockerignore` and `Dockerfile`.
- Keep GitHub Actions pinned by commit SHA and Docker images pinned by digest.
