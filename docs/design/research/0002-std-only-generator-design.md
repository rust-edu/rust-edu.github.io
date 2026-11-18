# Research 0002 — Std-Only Generator Design
**Date:** 2026-10-28 · **Status:** for review

A Rust-std-only generator is feasible: ~3,000 lines of code and ~1,500 of tests in five crates, because the inputs are
small and regular (11 Markdown files, 231 template lines, no code blocks or tables). One rule keeps it small and safe:
**subset, not dialect** — what we accept renders as CommonMark 0.31.2 / YAML 1.2 and GitHub's preview would, anything
else fails the build with `file:line` and a fix hint. It ships as a pin PR, 15 PRs of <= 400 lines, then one
cutover PR; Plan 0002 regroups them into its steps 1–8, so "PR n" here is not a plan step.

## Calibration (repo and live site, checked 2026-10-28)
| Finding | Evidence | Consequence |
|---|---|---|
| Markdown in use: ATX `#`–`###`, paragraphs, nested loose lists, `>`, `*` `_` `**`, code spans, inline links and images, `<https://…>`, `&lt;`, 2-space breaks. Unused: fences, tables, ordered lists, setext, reference links, tabs | grep of `content/**/*.md` | the §2 subset covers 100% |
| Raw HTML in 2 places | `content/_index.md:13-15`, `:31` | rewrite as Markdown, then reject HTML |
| Tera use: 3 blocks, 5 filters, 1 macro, `get_url()`; `home.html` is dead; `replace` is a no-op (no `$BASE_URL`) | `templates/base.html:8-12,30,42`, `page.html:7-12`, `resources.html:17,29` | layout + slot; fields computed in Rust; the hard-coded titles `Home`/`News`/`Redirecting...` (`index.html:3`, `news.html:3`, `redirect.html:3`) move to front matter |
| Zola slugifies file names | `/news/call-for-participation/` 200, `/news/call_for_participation/` 404 | file name = slug; rename 2 files |
| 3 relative links are 404 live; 6 `#…` anchors on /resources/ have no target `id` | `workshop/_index.md:10`, `workshop/cfp-2022.md:123,148`; `resources.html:16-17` | resolve links against the source dir; anchor check |
| `email:` link scheme; sitemap, robots, `get_url()` assets (CSS, JS, logo) and news permalinks use `rust-edu.github.io` (one 301 hop each), CNAME is `rust-edu.org` | `news/announcement.md:55`; `config.toml:2`, `static/CNAME:1`; live /news/ HTML | scheme allowlist; `base_url: https://rust-edu.org` |
| News list is not date-sorted (2022, 2026, 2025) | live /news/; no `sort_by` in `content/news/_index.md` | newest first |
| A plain slug reproduces 21/21 live heading ids | live HTML vs the §2 algorithm | `/workshop/cfp-2022/#schedule` links survive |
| Smart quotes alter 9 lines (on since 6c216a5); emoji and highlighting unused; no search index is built at all (key mis-scoped, §3; live 404) | `config.toml:10-20`, `workshop/cfp-2022.md:103` | drop all four |

## 1. Workspace layout
| Crate (`crates/…`) | Responsibility | Depends on |
|---|---|---|
| `edu-yaml` | YAML subset → `Node` tree; front-matter split | — |
| `edu-markdown` | Markdown subset → HTML + heading list; link-resolver callback | — |
| `edu-template` | template parse and render with its own `Value` | — |
| `edu-site` (lib) | content model, URLs, render, copy, sitemap/robots/404/feed, link check; maps `Node` → `Value` | the three leaves |
| `edu-cli` (bin `edu`; Plan 0002 folds it into `edu-site`, so four crates) | `edu build`, `edu check`, `edu serve`; alias `cargo site …` in `.cargo/config.toml` | `edu-site` |

- Virtual root manifest: `resolver = "3"` (set in `[workspace]`, Rust 1.84+); `[workspace.package]` `edition = "2024"`
  (stable since 1.85), `rust-version = "1.99.1"` (the toolchain pin), `publish = false`;
  `[workspace.lints.rust] unsafe_code = "forbid"`; clippy `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`,
  `dbg_macro`, `print_stdout`, `mod_module_files`, `wildcard_imports`, `enum_glob_use` and the three `cast_*` lints at
  warn (`clippy.toml` allows unwrap, expect and panic in tests), no `pedantic` or `nursery`, CI `-D warnings`; members
  `[lints] workspace = true` (1.74+).
- `rust-toolchain.toml`: `channel = "1.99.1"` (stable manifest of 2026-10-15), `profile = "minimal"`, clippy, rustfmt.
  Image `rust:1.99.1-slim-trixie` exists but installs `--profile minimal`; the Dockerfile adds both components.
- Zero deps: `Cargo.lock` lists only the workspace members (four in Plan 0002; no `source`/`checksum` lines); commit
  it, fail CI on `^source = ` and compile with `RUN --network=none cargo build --locked --offline` (Dockerfile 1.3+) as
  proof, dev-deps included. This guard stands in for a `deny.toml`: there is no third-party source to allow.

## 2. Markdown subset — `edu-markdown`, ~1,000 code + ~450 tests
| | Supported (output as CommonMark / GitHub) | Rejected with `file:line:col` + hint |
|---|---|---|
| Blocks | paragraphs; ATX `#`–`######`; fenced code with backticks or `~~~` (info → `class="language-x"`); `>` quotes; `-` `*` `+` and `1.` lists nested by content column, loose/tight, new list on marker change; `---` | HTML blocks, setext, indented code, tables, task lists, lazy continuation, leading tabs, `[x]: url` |
| Inlines | code spans; `*`/`_` em; `**`/`__` strong; `[t](dest)` with balanced parens; `![alt](src)`; `<https://…>` `<mailto:…>` `<a@b.c>`; `\` escapes; entity references kept (same rendering, but cmark decodes them, spec ex. 25, so not byte-exact); hard break (2 spaces or `\`) | inline HTML (`<tag`, `</tag`, `<!--`), link titles, `[a][b]`, runs of 3+, unmatched flanking `*`/`_`, `~~`, bare URLs and emails (GitHub autolinks them, CommonMark does not) |

- Algorithm: pass 1 is line-based — open containers (quote, list item) match per line by marker/indent; a line that
  matches none closes them (no laziness, no backtracking). Pass 2 scans each paragraph's joined lines: code spans and
  autolinks, links (recursive), then `*`/`_` runs on a delimiter stack; a closer takes the nearest same-length opener.
- Escape `& < > "` (plus `'` in attributes); `&name;` and `&#…;` pass through; raw HTML is never emitted, so `Vec<T>`
  outside backticks is an error. Heading ids: lowercase, keep alphanumerics, other runs → `-`, trim `-`, dedupe `-1`, `-2`.
- Pitfalls avoided: emphasis (17 rules incl. rule of 3, 132 spec examples) → same-length pairs, `_` never intraword;
  lazy continuation → error; nested lists → content-column indent, no tabs; `(…)` in URLs → paren counter, not regex.
- Tests (std harness only): golden `tests/golden/*.md` → `.html`, byte-exact, `UPDATE_GOLDEN=1` re-blesses; vendored
  `spec.txt` 0.31.2 (652 examples, CC-BY-SA 4.0: keep notice; repo is CC0), ~200 allowlisted (estimate) + error cases.
  Plan 0002 skips the vendored spec and diffs each golden against Zola 0.22.1's output instead.

## 3. Metadata and config — `edu-yaml`, ~300 code + ~250 tests
- Grammar: `block(n) = map(n) | list(n)`; a map is lines at indent `n` of `key: scalar` or `key:` + a deeper block; a
  list is `- scalar` or `- key: …` (list of maps, sibling keys at `n+2`); keys `[A-Za-z0-9_-]+`; scalars plain,
  `'single'` (`''`) or `"double"` (`\" \\ \n \t`); `#` comments at line start or after a space, outside quotes.
- Every scalar is a string typed by its consumer (`date` = `YYYY-MM-DD`, `draft` = `true|false`, integers): no
  implicit typing ([StrictYAML](https://hitchdev.com/strictyaml/)'s "Norway problem").
- Errors with line:col: tabs, duplicate keys, `&` anchors, `*` aliases, `!` tags, flow `[ ] { }`, `| >` block scalars,
  multi-line scalars, `? ` keys, `<<` merges, plain values containing `: ` (hint: quote; 7 today, `cfp-2022.md:2`).

| Option | Covers today | Parser | GitHub web view | Verdict |
|---|---|---|---|---|
| **YAML subset** | nav lists of maps; resources 3 levels deep | ~300 | key/value table, nested lists and maps too (verified) | **recommended** |
| TOML subset | everything, no migration | ~450 (dotted and `[[…]]` headers, inline tables) | `+++` block as garbled text (verified) | scoping bit us: since bad5fe7 neither `title` (now in `[link_checker]`, `config.toml:22-30`) nor `build_search_index` (now in `[search]`, `config.toml:19-20`) is top-level |
| Flat `key: value` + structure in Markdown | no nav lists | ~80 | table | too weak for `site.yaml` |

- `site.yaml` replaces `config.toml`; front matter uses `---`, which Zola also reads, so content converts before cutover.

## 4. Template mini-language — `edu-template`, ~450 code + ~250 tests
| Syntax | Meaning |
|---|---|
| `{{ page.title }}` | dotted path, HTML-escaped; a missing path is an error |
| `{{ page.content \| safe }}` | raw output; `safe` is the only modifier (Tera spelling, `templates/index.html:11`) |
| `{% if path %}…{% else %}…{% endif %}` | true = non-empty string/list/map or `true`; a missing path is false |
| `{% for item in path %}…{% endfor %}` | iterate a list |
| `{% include "resource.html" %}` | partial sharing the current scope; replaces the `macros.html` macro; `{# … #}` comments (Plan 0002 drops `include`: resources become Markdown) |

- **Layout + slot**: every template's output is wrapped by `layouts/layout.html` at `{{ content | safe }}` (as
  [Jekyll](https://jekyllrb.com/docs/layouts/) does). The 3 blocks become data — `title` → `page.title`, `head` (only
  `redirect.html:5-7`) → `{% if page.redirect_to %}` — so no block tables, overrides or `super()`; one place for chrome.
- Values: `Str | Bool | List | Map` (ordered). Rust precomputes `page.url`, `page.date_display` ("Jul 11, 2022", was
  `date(format=…)`), `page.pages`; `and` (`page.html:7`) becomes nested `if`; `get_url()` becomes `/style.css`.
- Errors name template and line (`layouts/page.html:7:4: unknown variable page.titel`, unclosed `if` from line N,
  unknown tag, missing include, depth > 8). Templates load at build time, so design changes need no recompile.

## 5. Build pipeline — `edu-site`
- `a/b.md` → `/a/b/index.html`, `a/_index.md` → `/a/index.html` (a section: `page.pages` = siblings, newest first);
  `.md` names match `[a-z0-9-]+`; template = `template:` key, else `home.html` (root), `section.html` or `page.html`.
- Relative links and images resolve against the source file's directory, as GitHub's file view does, and are rewritten
  to root-relative URLs (`x.md` → its page URL); unknown targets fail. This removes the cause of the 3 live 404s.
  While Zola still deploys, content links between pages stay root-relative, the one form both resolve alike (Plan 0002).
- Other files in `content/` are copied beside their page (PDF, `.tex`, `.cls`); `static/` (plain CSS, JS, CNAME) is
  copied to the root untouched. `draft: true` is excluded everywhere unless `--drafts` (PR builds, `main.yml:24`).
- Render into `public.tmp/` and swap only on success; delete only a non-source directory inside the repo that holds
  our `.edu-output` marker. Deterministic (sorted walks, ordered maps, no clock); CI builds twice and diffs. Errors
  are collected as `path:line:col: error: …` plus `::error file=…,line=…::` annotations under GitHub Actions; exit 1.

## 6. Dev server — `edu serve`, std::net only, ~300 code + ~120 tests
- `TcpListener`, a thread per connection, `Connection: close`, 5 s read timeout, 8 KiB header cap; GET/HEAD, else 405.
- Resolution as GitHub Pages does it (checked live): `/news` → 301 `/news/`, `dir/` → `index.html`, `/404` →
  `404.html`, missing → `404.html` with status 404; ~15 MIME types, default `application/octet-stream`.
- Traversal: split the raw path on `/`, percent-decode each segment, reject `..`, `.`, `\` and NUL, push segments one
  by one (`PathBuf::push` of an absolute path replaces the base), then `canonicalize` and require `starts_with(root)`.
- Rebuild on request when the sources' (file count, max mtime) changed, under a `Mutex`; build errors render as a 500
  page. Optional auto-reload: served HTML polls `/__edu/version` (WebSockets would need SHA-1, which std lacks).
- Bind `127.0.0.1:1111` (Zola's port, `README.md:39`); in Docker `--bind 0.0.0.0:1111` with `-p 127.0.0.1:1111:1111`.

## 7. Checks and outputs
- Link check after every build over generated HTML: each `href`/`src` must resolve under the server's rules; schemes
  allowlisted (`https`, `http`, `mailto`), so `email:` and `javascript:` fail; `#frag` must match an `id` in the target.
  On today's site it flags the 3 bad links, 6 dead anchors and the `email:` link. External links: no TLS in std, skip.
- `sitemap.xml`, `robots.txt`, `404.html` (parity) and later Atom `/atom.xml` (RFC 4287) are strings through one escape
  fn (`& < > " '`); dates are `YYYY-MM-DD` + leap-year check; feed `updated` = newest post date, never build time.

## 8. Std-only trade-offs
| Gain | Cost we take on | Mitigation |
|---|---|---|
| No supply chain beyond the toolchain image and official actions | Markdown/YAML correctness is ours | closed subsets, reject the rest, golden + spec tests |
| No upstream breakage: 3 of the last 9 commits (eb4cb8b, 6c216a5, 25cf3df) only changed `config.toml`, right after 3 failed PR builds (2026-03-31); 6c216a5 and 25cf3df rework the highlighting keys Zola 0.22 moved to `[markdown.highlighting]`, which arrived via unpinned `shalzz/zola-deploy-action@master` (`main.yml:21,36`), as Zola 0.23 (Tera 2) has since | ~3k lines to maintain | five small crates; new syntax needs an issue + tests |
| A readable all-Rust codebase that doubles as teaching material for a Rust-education org | bus factor | `docs/` architecture note, tests as the spec, crates small enough to read in one sitting |
| Fast, offline builds | no highlighting, search, emoji, smart quotes, external link check | only smart quotes are used today (9 lines); explicit non-goals |

## 9. Size and PR split (estimates ±30%; fixtures and vendored `spec.txt` excluded)
| PR | Scope | Code + tests |
|---|---|---|
| 0 | pin `shalzz/zola-deploy-action@v0.22.1` (= `74526e6`, last green deploy 2026-05-01) at `main.yml:21,36`: every PR runs Zola (`main.yml:9`) and `@master` now ships 0.23.6, whose Tera 2 drops the macros `resources.html:1,29,42` uses, so the next build fails (reproduced: Zola 0.23.6 stops with `Unknown tag` at `templates/macros.html:1:4`); consider bumping `actions/checkout@v3.0.0` too (GitHub announced Node 20 removal from runners for 2026-09-16) | 2 |
| 1 | workspace, toolchain, lints, CI (fmt, clippy, test, zero-dep guard) | ~150 |
| 2–3 | `edu-yaml`: scalars, maps, errors · lists, lists of maps, front-matter split | 300 + 250 |
| 4–7 | `edu-markdown`: paragraphs, headings, fences · quotes, lists · code, links, autolinks · emphasis, breaks, spec allowlist | 1,000 + 450 |
| 8–9 | `edu-template`: lexer, parser, output, errors · if, for, include, layout, loader | 450 + 250 |
| 10–12 | `edu-site`: walk, URLs, render, copy, swap · sections, drafts, sitemap/robots/404, errors · link rewriting, link and anchor check | 800 + 350 |
| 13–14 | `edu-cli`: build and check · dev server | 400 + 150 |
| 15 | Dockerfile; CI builds and tests inside the image with `--network=none` | ~100 |
| 16 | cutover: `site.yaml`, template port, workflow on official Pages actions; delete `config.toml`, `sass/`, `ext/`, `.gitmodules` | ~400 + deletions |
| later | Atom feed; auto-reload | ~200 each |

- Prior art: [smu](https://github.com/Gottox/smu) is one 677-line C file; [linemd](https://github.com/90-008/linemd)
  is a zero-dependency Rust parser + renderer in ~610 lines; [md4c](https://github.com/mity/md4c) passes all CommonMark
  0.31.2 tests with a 7,464-line parser (`md4c.c`, GFM extensions included) plus a 2,186-line entity table (upper
  bound); [Djot](https://github.com/jgm/djot) makes similar cuts (no indented code blocks, raw HTML only when marked).

## Implications for the plan
- Acceptance rule for every parser PR: golden/spec tests for what it accepts, an error test for what it rejects.
- Order: PR 0 → leaves → site → cli → Docker → cutover. After PR 0, PRs 1–15 never touch `main.yml`, so Zola keeps deploying.
- Live-safe Zola PRs after PR 0: `base_url = "https://rust-edu.org"`; fix 3 links, `email:` and 2 raw-HTML spots; rename the
  2 underscore files (same URLs); front matter → YAML `---` (Zola verifies it), after which CI can `edu check` real content.
- Cutover gates: the 11 sitemap URLs and every static/colocated file exist; link and anchor checks clean; heading ids
  unchanged; news newest first; sitemap on `rust-edu.org`. Rollback: revert (and the Pages source, if switched).
- `actions/upload-pages-artifact` v5 + `actions/deploy-pages` v5 need Pages source "GitHub Actions", an upstream setting
  (admin or maintainer) to change before PR 16. Keep the CSS rewrite a separate track (can land on Zola first).

## Open questions

Answered in Plan 0002; scheduled (future-dated) posts are not part of it.
