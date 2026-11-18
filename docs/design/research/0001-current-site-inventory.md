# Research 0001 — Current Site Inventory
**Date:** 2026-10-28 · **Status:** for review

The live site is exactly 29 files on upstream `gh-pages` (d63ba44, which Pages serves: [deploy run][pbd]), pushed
2026-05-01 by the [CI run][run] for `main` 2ca5e7c with Zola v0.22.1 (pinned by the action's `master` then: [Dockerfile][df]):
11 HTML routes and a 404, built from 11 Markdown files, 9 Tera templates (one dead) and a 429 KB stylesheet holding Bulma
0.9.3 twice. Content needs only a small CommonMark subset plus 4 lines of raw HTML. Some settings are inert or mis-scoped,
`base_url` sends the CSS, logo, JS and every permalink through a 301 from rust-edu.github.io, and 3 internal links 404.

## 1. Markdown constructs in `content/` (8 files have a body; 3 are front matter only)
| Construct | Count | Example |
|---|---|---|
| ATX headings h1 / h2 / h3 | 2 / 14 / 5 | `content/news/call_for_participation.md:10`, `content/_index.md:5`, `content/workshop/cfp-2022.md:25` |
| Strong `**x**` (3 inside link text) | 43 | `content/news/announcement.md:12`, `content/news/call_for_participation.md:28` |
| Emphasis `*x*` / `_x_` (7 span two lines, one wraps a code span) | 12 / 2 | `content/workshop/cfp-2022.md:62-63`, `content/news/campus_tour.md:61` |
| Inline code (4 inside link text) | 6 | `content/workshop/cfp-2022.md:134`, `content/workshop/paper-format-latex/_index.md:10` |
| Inline links: 14 absolute, 4 `#id`, 8 relative, 1 root-relative | 27 | `content/workshop/cfp-2022.md:7` (`#schedule`), `content/workshop/_index.md:9` (`proceedings.pdf`) |
| Autolinks `<https://…>` / images (`/images/*.png`, alt, no title) | 1 / 4 | `content/news/call_for_participation.md:12`, `content/news/campus_tour.md:29` |
| Bullet items `-` and `*`: depth ≤ 2, tight and loose, multi-paragraph, indented continuations | 64 | `content/workshop/cfp-2022.md:49-94`, `:102-114` |
| Blockquotes (one level) / hard breaks by two trailing spaces (13 more such lines are no-ops) | 2 / 5 | `content/news/campus_tour.md:61-62`, `content/workshop/cfp-2022.md:7` |
| HTML entities / bare `&` to escape | 2 / 1 | `content/news/announcement.md:55` (`&lt;` `&gt;`), `content/workshop/cfp-2022.md:74` |
| Raw HTML, every occurrence | 4 lines | `content/_index.md:13-15` (`<div class="buttons is-centered my-5">`, `<a … class="is">`), `content/_index.md:31` (`<a href="mailto:…">`) |
| Smart-punctuation dependants: straight `"…"` pairs / straight `'` | 2 / 8 | `content/_index.md:27`, `content/news/campus_tour.md:25` |

**Not used (leave out):** setext headings, h4–h6, `__strong__`, fenced or indented code (so no highlighting), ordered
lists, reference links, bare URLs, tables, footnotes, rules, task lists, strikethrough, backslash escapes, `{#id}`, HTML
comments, `@/` links, shortcodes, `:emoji:` codes (📧 🦀 are literal; `7:10:` is a time), `--`/`...` smartening.
**Parser gotchas:** `*Conference Date:*` at line start is emphasis, not a bullet (`content/workshop/cfp-2022.md:7`);
`*   ` items start text at column 4 (`content/news/announcement.md:23-29`); a bullet may interrupt a paragraph
(`content/news/campus_tour.md:19-20`); a marker change starts a new list (D5).

## 2. Front matter (TOML between `+++`; every leaf value is a string except `date`)
| Files | Keys | Read by |
|---|---|---|
| `content/_index.md`, `content/workshop/{_index,cfp-2022,code-of-conduct}.md`, `content/workshop/paper-format-latex/_index.md` | `title` | `templates/index.html:7`, `templates/section.html:3,6`, `templates/page.html:3,6` |
| `content/news/{announcement,call_for_participation,campus_tour}.md` | `title`, `date` (TOML date), `[extra] author` | `templates/page.html:7-8`, `templates/news.html:12,15` |
| `content/news/_index.md` | `template = "news.html"` only; no title (`templates/news.html:3,6` hardcode "News") | Zola |
| `content/zulip.md` | `template = "redirect.html"`, `[extra] link`; no title | `templates/redirect.html:6,11` |
| `content/resources/_index.md` | `title`, `template`, `[[extra.categories]]` → `resources[]` and `subcategories[]` → `resources[]`; resource = `title`, `url`, `description`, optional `author` | `templates/resources.html:15-48`, `templates/macros.html:3-14` |

Resources: 6 categories, 6 subcategories and 16 resources (2 with `author`); Macros, Networking and Web are empty
(`content/resources/_index.md:106-110,120-121`).

## 3. Zola features in use
| Feature | State | Evidence |
|---|---|---|
| Sections, pages, overrides | 5 sections, 6 pages; `index.html` (root), `section.html`, `page.html`; 3 `template =` overrides | `content/news/_index.md:2`, `content/resources/_index.md:3`, `content/zulip.md:2` |
| Not used | `sort_by` (unset = "`none` order, not intended for sorted content"), `paginate_by`, taxonomies, `draft` (`--drafts` on PRs is a no-op), `aliases`, `weight`, `slug`, `path`, `page_template`, feeds | [section docs][zs], `.github/workflows/main.yml:24` |
| Tera | `extends` 7; base blocks `head`, `title`, `content` + 15 overrides; `for` 8; `if` 7 (one `and`); 1 macro, 1 `import`, 2 calls; no `include`, `set`, `else` | `templates/*.html` |
| Filters, functions | `safe` 4, `markdown` 3, `replace` 3, `slugify` 1, `date` 1; `get_url` 3; no `get_page`, `get_section`, `load_data` | `templates/*.html` |
| Heading ids | all 21 headings via `slug::slugify`: transliterate to ASCII, lowercase, each run of non-alphanumerics → `-`, none at either end (`What’s Next?` → `what-s-next`, `Why a Campus Tour?` → `why-a-campus-tour`) | live `/news/campus-tour/`; [Zola][zsrc] `utils/src/slugs.rs:38-40`; [slug docs](https://docs.rs/slug/latest/slug/fn.slugify.html) |
| Link rewriting and checking | relative `x` → `<page permalink>x`, never checked (D3); `#x` → `https://rust-edu.github.io/<page>/#x`, anchor checked at build; there are no `@/` links; external links are checked only by `zola check`, which CI never runs | [Zola v0.22.1 source][zsrc] `markdown/src/markdown.rs:191-205`, `site/src/lib.rs:341-363`; [entrypoint.sh:40-41][ep] |
| Inert settings | `build_search_index` (a top-level key, placed under `[search]`: D1), `render_emoji` (no codes), `minify_html` | `config.toml:11,19-30`, [config docs][zc] |
| `compile_sass`, `smart_punctuation` | `sass/style.scss` → `/style.css` (compressed, 429,277 B); smartening curls the 2 `"…"` pairs and 8 `'` of §1 | `config.toml:6,12` |
| Built-ins, `base_url` | `sitemap.xml` (11 URLs) and `robots.txt` use `base_url` = github.io (D2); `404.html` is 68 B, unstyled; no template links them | `config.toml:2`, [`gh-pages` tree][tree] |

## 4. Output URL map: the 29 files on [`gh-pages`][tree] (Pages 301s slashless directories, e.g. `/news` → `/news/`)
- **Pages (`x/index.html`):** `/`, `/news/`, `/news/announcement/`, `/news/call-for-participation/`,
  `/news/campus-tour/`, `/resources/`, `/workshop/`, `/workshop/cfp-2022/`, `/workshop/code-of-conduct/`,
  `/workshop/paper-format-latex/`, `/zulip/` (meta refresh); default `slugify.paths` made `_` into `-`, `_` URLs 404.
- **Colocated (6):** `/workshop/proceedings.pdf`, `/workshop/statement-template.pdf`,
  `/workshop/paper-format-latex/{rust-edu-workshop.cls,sample.tex,sample.bib,sample.pdf}`.
- **`static/` (7):** `/images/RAACampusTour{1,2,3,4}.png`, `/rust-edu-banner.svg`, `/js/site.js`, `/CNAME`.
- **Generated (5):** `/style.css`, `/404.html`, `/sitemap.xml`, `/robots.txt`, `/.nojekyll` ([entrypoint.sh:96][ep]).
- **Fragments linked from content:** `/workshop/cfp-2022/#schedule`, `/workshop/cfp-2022/#participating`.

## 5. Templates (classes are Bulma unless marked custom; "spacing" = `m*-*`/`p*-*` helpers)
| File | Lines | Used by | Constructs | Data | Classes |
|---|---|---|---|---|---|
| `base.html` | 82 | all | 3 blocks, 3 `for`, `get_url` ×3, `replace` ×3 | `config.base_url`, `extra.navbar`, `extra.external` | `navbar` + 5 `navbar-*`, `columns` (with `is-fluid`, which has no rule there), `column`, `section`, `container`, `footer`, `title is-5`, `content`, `has-text-*`, `is-size-6`, `is-centered`, `is-narrow`, spacing; custom `navbar-logo`, `gray`, `footer-notice`, `main-content` |
| `index.html` | 13 | `/` | `extends`, `safe` | `section.title`, `section.content` | `title is-1`, `subtitle is-4`, `has-text-centered`, `content`, spacing |
| `page.html` | 14 | 5 pages | `extends`, `if … and`, `date`, `markdown \| safe` | `page.title`, `date`, `extra.author`, `content` | `title is-2`, `subtitle is-6`, `content` |
| `section.html` | 10 | 2 sections | `extends`, `markdown \| safe` | `section.title`, `section.content` | `title`, `content` |
| `news.html` | 20 | `/news/` | `extends`, `for` | `section.pages[]`: `permalink`, `title`, `date` | `card`, `card-content`, `block`, `is-flex`, `title`, `subtitle`, spacing |
| `resources.html` | 52 | `/resources/` | `import`, `extends`, 4 `for`, 4 `if`, `slugify`, `markdown \| safe`, 2 macro calls | `section.extra.categories` tree | `notification is-info is-light`, `title is-4`, `subtitle is-5`, `content`, spacing; custom `resources`, `subcategory` |
| `macros.html` | 16 | `resources.html` | 1 macro, 2 `if` | `resource.*` | `is-size-5`, `has-text-weight-semibold`, `is-italic`, `has-text-grey`, spacing; Font Awesome `fa-solid …`; custom `external-link-icon` |
| `redirect.html` | 12 | `/zulip/` | `extends`, 3 blocks | `page.extra.link` | `title is-2` |
| `home.html` | 12 | nothing (dead) | `extends` | `config.title`, `config.description` | `title is-2`, `subtitle is-4`, `has-text-centered` |

## 6. `static/js/site.js`
32 lines from the Bulma navbar docs (`static/js/site.js:2`) that toggle `is-active` on `.navbar-burger` and `#navMenu`,
which Bulma needs to show the menu below 1024 px. Not needed without Bulma: four links fit a wrapping flex row.

## 7. Defects and smells (each verified)
| ID | Finding | Evidence |
|---|---|---|
| D1 | Mis-scoped keys: `build_search_index` under `[search]` (no index, no UI; live `/search_index.en.js` 404); `title`, `description`, `minify_html` under `[link_checker]` | `config.toml:19-30` |
| D2 | `base_url` ≠ domain: CSS, logo, JS, permalinks and `#` anchors 301 via github.io; sitemap and robots advertise github.io URLs | `config.toml:2`, `static/CNAME:1`, live `<link href>` |
| D3 | 3 internal links 404 live: `cfp-2022.html`, `paper-format-latex/`, `code-of-conduct` resolve under the page URL | `content/workshop/_index.md:10`, `content/workshop/cfp-2022.md:123,148` |
| D4 | `email:` scheme instead of `mailto:`, wrapped in `&lt;…&gt;`; two invisible U+200B in a quote | `content/news/announcement.md:55`, `content/news/campus_tour.md:61` |
| D5 | Marker change `*` → `-` splits the GOALS list into two `<ul>` (live HTML) | `content/news/announcement.md:23,31,36` |
| D6 | Two `<h1>` on two pages (template + content) | `templates/page.html:6` + `content/news/call_for_participation.md:10`; `templates/section.html:6` + `content/workshop/paper-format-latex/_index.md:5` |
| D7 | News list is unsorted (live: 2022, 2026, 2025) with raw ISO dates; pages use `%b %e, %Y` | `content/news/_index.md:1-3`, `templates/news.html:15`, `templates/page.html:8` |
| D8 | Markup: `<a role="button">` nested in the logo `<a>`; Font Awesome icon but no Font Awesome loaded (0 `fa-` rules live); resource `<h2>` links to `#slug` without an `id` | `templates/base.html:18-25`, `templates/macros.html:5`, `templates/resources.html:16-17` |
| D9 | 3 empty subcategories render bare headings; Markdown in descriptions prints raw (`` `no_std` ``) | `content/resources/_index.md:89,106-110,120-121`, `templates/macros.html:14` |
| D10 | Bulma imported twice (2 banners in live CSS), lumen variables twice, runtime Google Fonts `@import` from Bulmaswatch; declared Fira Sans never reaches `body` | `sass/_variables.scss:1-2,6`, `sass/style.scss:3,7-8` |
| D11 | Raw HTML with Bulma classes in content; `class="is"` matches no CSS rule | `content/_index.md:13-15,31` |
| D12 | Dead code: `home.html`; `replace('$BASE_URL')` matches nothing; `markdown` re-runs on rendered HTML; `category.description` never set; `main-content`, `subcategory`, `external-link-icon` have no CSS rule | `templates/home.html`, `templates/base.html:30,38,55,63`, `templates/page.html:12`, `templates/section.html:8`, `templates/resources.html:20-24,36`, `templates/macros.html:5` |
| D13 | `/workshop/` has no inbound link; Zulip on two hosts, `zulip.rust-edu.org` 301s to `http://`; two `http://rust-edu.org` links | `config.toml:34-39`, `content/_index.md:32` vs `content/zulip.md:5`, `README.md:3`; `content/workshop/_index.md:5`, `content/workshop/cfp-2022.md:11` |
| D14 | README snippet is invalid TOML (`,,`) and stale; `cargo install zola` hits a yanked "zola installer" crate ([crates.io](https://crates.io/crates/zola); [install docs][zi] say `cargo install --locked --git …`); `ext/` submodules are empty here | `README.md:13,25,29-33,52-57`, `.gitmodules:2-7` |
| D15 | CI: third-party `shalzz/zola-deploy-action@master` unpinned, gets `GITHUB_TOKEN` on PR builds too, force-pushes `gh-pages` ([entrypoint.sh:104][ep]); `actions/checkout@v3.0.0` runs on [node16](https://github.com/actions/checkout/blob/v3.0.0/action.yml) | `.github/workflows/main.yml:17,21,25,32,36` |

## Implications for the plan
- **Markdown subset = the whole parser:** paragraphs; ATX h1–h3 with ids by the §3 rule; `-`/`*` bullets to depth 2
  (tight, loose, multi-paragraph); one-level blockquotes; `**`, `*`/`_`, code spans, inline links, images, `<…>`
  autolinks, two-space breaks; escape `& < >`; pass `&name;` through. Nothing from "Not used"; gotchas become tests.
- **Templates:** escaped `{{ var }}` plus one raw form for rendered HTML, `{% for %}` (news, nav), `{% if %}` (byline)
  and one shared layout instead of `extends`/blocks. No filters: the generator precomputes `url`, `date_display`
  (`Jul 11, 2022`) and heading ids. Nine templates become about four: layout, page, news list, redirect.
- **Front matter (Zola's own keys, which Plan 0002 keeps):** `title`, `date` (`YYYY-MM-DD`, news), `authors` (list;
  was `[extra] author`), `template`, `redirect_to` (was `[extra] link`), `sort_by`; news sorts newest first (D7).
  Site config: `base_url = "https://rust-edu.org"`, `title`, 4 nav and 2 footer links.
- **Resources as plain Markdown** (`## Category`, `### Subcategory`, `- [Title](url) — Author. Description.`) drops
  the 3-level TOML, `macros.html`, `resources.html`, `slugify` and `markdown`, and fixes the D8 anchors and D9's raw Markdown.
- **Asset rule:** copy `static/**` and each non-`.md` file under `content/` to the same relative path, failing on
  collisions; write `x/index.html` per page so Pages keeps the slashless 301s. Official Pages actions ignore `CNAME`;
  the domain is set in repo settings ([docs][cn]); `/CNAME` and `/.nojekyll` (unverified) can go once Pages uses Actions.
- **URL parity:** keep the 11 page URLs, `/404.html`, the 10 content assets, the banner, `/style.css`, and
  `/sitemap.xml` + `/robots.txt` (now on the real domain), plus all 21 heading ids; CI diffs the built file list against
  §4 and checks every relative, root-relative and `#` link (it would catch D3). Drops: `/js/site.js`; `/CNAME`, `/.nojekyll` as above.
- **Content fixes ship with the switch:** D3–D6, D11 and the straight quotes (2 pairs, 8 `'`), so smart punctuation is not needed.

## Open questions

Answered in Plan 0002, except three details for the Part A PRs: the Zulip host and the `/workshop/` link (D13),
and the 3 empty resource subcategories (D9).

[tree]: https://api.github.com/repos/rust-edu/rust-edu.github.io/git/trees/gh-pages?recursive=1
[zs]: https://www.getzola.org/documentation/content/section/
[zc]: https://www.getzola.org/documentation/getting-started/configuration/
[zi]: https://www.getzola.org/documentation/getting-started/installation/
[zsrc]: https://github.com/getzola/zola/tree/v0.22.1/components
[ep]: https://github.com/shalzz/zola-deploy-action/blob/74526e63f38f84b82476349bbcd886b82040ff98/entrypoint.sh
[df]: https://github.com/shalzz/zola-deploy-action/blob/74526e63f38f84b82476349bbcd886b82040ff98/Dockerfile
[run]: https://github.com/rust-edu/rust-edu.github.io/actions/runs/25219955902
[pbd]: https://github.com/rust-edu/rust-edu.github.io/actions/runs/25219977919
[cn]: https://docs.github.com/en/pages/configuring-a-custom-domain-for-your-github-pages-site/managing-a-custom-domain-for-your-github-pages-site
