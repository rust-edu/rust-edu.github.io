# Research 0004 — What to Keep from the August Plan
**Date:** 2026-10-28 · **Status:** for review

Most of the August plan (commit 25d26ff) does not depend on Zola: its brand, accessibility, budget, URL, feed and
CI/deploy decisions carry over; everything tied to Zola, Sass, Tera or `.zola-version` is superseded. Four facts need
fixing first: no search index since June 2025, one token pair fails AA, stale action pins, and a broken next deploy (§3).

Key: PRD, P1, F1, F7 = Spec-0001, Plan-0001, FR-0001 and FR-0007 `spec.md` under `docs/` at tag
[`archive/initial-project-plan`](https://github.com/kenesparta/rust-edu.github.io/tree/archive/initial-project-plan/docs)
(commit 25d26ff, on the fork kenesparta/rust-edu.github.io); repo = `main` @ 2ca5e7c.

## 1. Carry forward

| Old ID | Decision | New plan |
|---|---|---|
| AC-0001.04, F1 R3/D-03 | Rust palette `#2a3439` `#ffc832` `#a72145` `#0b7261` `#2e2459`, banner logo, system font stack | CSS tokens (§4) |
| NFR-0004, F1 R9 | CSS ≤ 51 200 B and first-party JS ≤ 10 240 B, uncompressed | Build fails over budget |
| NFR-0002/0008, AC-0002.05, F1 D-08 | WCAG 2.1 AA in both themes (4.5:1 text; 3:1 large text, UI); skip link, landmarks, 2 px `:focus-visible`; no JS needed | Templates |
| NFR-0001, AC-0001.01/.02 | No third-party requests (CDN, fonts, analytics); no submodules or vendored CSS | Delete `.gitmodules`, `ext/`, `sass/` |
| AC-0001.03, F1 D-05 | 320–1920 px, one 768 px breakpoint, disclosure menu that stays open without JS | Breakpoint kept; JS optional (Plan 0002, decision 3) |
| NFR-0010, AC-0001.06, AC-0005.04 | Every current URL keeps resolving; a rename ships a redirect | URL snapshot test |
| US-0003, FR-0004 | Atom (RFC 4287) news feed, autodiscovery on all pages, Subscribe link, author fallback "Rust-Edu", no drafts, W3C-valid | `/atom.xml` (#6) |
| AC-0006.01–.04 | Post = 1 Markdown file (title, date; author optional); resource ≤ 10 lines (title, URL); no markup; bad file names file + cause | Same, YAML-style |
| AC-0008.01, NFR-0007, F7 D-01/D-08 | `actions/*` only, exact tags; CI read-only, no secrets, no `pull_request_target`; deploy `pages`/`id-token: write` | Re-pin (§2) |
| AC-0008.03, F7 D-05/D-07 | PRs: full build with drafts + blocking internal link check; external links warn-only; dispatchable sweep | Our own link checker |
| AC-0008.04/.05, F7 D-06/D-11 | Failed build/deploy keeps the old site; newest wins, never cancel in-flight; `workflow_dispatch`; `ubuntu-24.04`; ≤ 10 min | Unchanged |
| AC-0007.01–.03, AC-0008.02 | Fresh clone to running site in ≤ 3 commands, no submodule step; README matches the real steps; one pinned toolchain source; local build = CI build | One command; `rust-toolchain.toml` + image digest |
| NFR-0003/0005/0009/0011/0012, IC-0008.01/.02, PRD §9 | Last 2 evergreen browsers; Lighthouse ≥ 95; CC0 footer; small PRs that each deploy; titles, descriptions, sitemap; recipes mirror real files; non-goals | Unchanged |

## 2. Superseded

- **Zola pin** (NFR-0006, F7 D-02/D-03): `.zola-version`, `.zola-version.sha256` and the checksum download give way
  to `rust-toolchain.toml` and a digest-pinned Docker base image.
- **Sass** (F1 D-01/D-02/D-06): `_tokens.scss`, `_base.scss`, `_layout.scss`, the Sass breakpoint and dark construct.
  Plain CSS has custom properties; only the breakpoint stays a literal `768px` (media queries cannot read `var()`).
- **Tera and Zola specifics** (F1 §4.1/D-09, F7 D-05/D-14): `title`/`head`/`content` blocks, `get_url`, `$BASE_URL`,
  `slugify`, `zola build --drafts`, `zola check`, `[link_checker]`. Kept as ideas: nav from config, a `--drafts` flag.
- **Transition shims** (F1 D-12, F7 D-09, runbook step 8): inert Bulma classes and `submodules: true` are moot.
- **Pins** (F7:177-178): `checkout@v4.2.2`, `configure-pages@v5.0.0`, `upload-pages-artifact@v3.0.1`, `deploy-pages@v4.0.5`;
  latest today v7.0.1, v6.0.0, v5.0.0, v5.0.1 (`https://api.github.com/repos/actions/<name>/releases/latest`).
- **Delivery** (P1:33-81): the 9-PR series, M1–M3 and the 11–15.5-day LOE assumed Zola stays; void now.
- **Deferred**: search (FR-0005, built on Zola's index) and Rust highlighting (FR-0002); content has no fenced code.

## 3. Cited defects and upstream items (live site checked 2026-10-28)

| Item | Evidence | Still matters? |
|---|---|---|
| Uninitialized submodules break a fresh build | `.gitmodules:2-7`, `sass/style.scss:3,7-8`, `README.md:29-33,44-48` | Yes; removed with Sass |
| Unpinned third-party deploy, `gh-pages` push | `.github/workflows/main.yml:17,21,36,38`; PR #13's CI failed on 2026-03-31 until its third config fix (eb4cb8b, 6c216a5, 25cf3df) under the action's Zola 0.22.1; `@master` now bundles 0.23.6 (Tera 2), which fails locally at `templates/macros.html:1` | Yes; the next deploy fails; new workflows |
| "Search index built but unused" (PRD:49) | Not built since bad5fe7 (merged 2025-06-02; inferred, as a June 2025 deploy went live): 4 top-level keys sit in tables (`config.toml:19-30`), and Zola's `[search]`/`[link_checker]` ignore unknown keys | Strict parser |
| `base_url` github.io vs `CNAME` (F7 D-13) | `config.toml:2`, `static/CNAME:1`; the live sitemap lists 11 github.io URLs; assets load via 301 | Yes; fix in config |
| Template debris (F1 R8, pattern 9) | Font Awesome markup without its CSS (`templates/macros.html:5`), dead `home.html`, nested `<a>` (`base.html:18-25`) | No; replaced |
| Link `#4299bf` (3.21:1); "Fira Sans" never shipped: live text is Source Sans Pro from a Google Fonts `@import` | `sass/_variables.scss:6,18`; first rule of the live `/style.css` | Yes; see §4, NFR-0001 |
| New: CFP 2022 link is 404 | `content/workshop/_index.md:10` links `cfp-2022.html`; the page is `/workshop/cfp-2022/` | Yes; check relative links |
| New: resource anchors have no target | `templates/resources.html:16-18`; live `href="#books"` etc. with no matching `id` | Yes; emit ids |
| #1 Bootstrap? (open, 2022); PR #13 CFP (open; P1:99 says merged) | No framework answers #1; #13 is fully on `main`: identical `config.toml` (25cf3df = 1e7a7f5's config patch), post rewritten via `cfp-2026` (2ca5e7c) | No; close both |
| PR #3 book, PR #9 course, #5 resource gaps (open) | Content and curation only | Yes; credited entries (P1 §5) |
| #6 "Provide a way to follow progress" (open) | `/atom.xml` is 404 | Yes; the Atom feed |

## 4. CSS approach, restated for plain CSS

- One `:root` token block (F1 §4.5): `--brand-*`; `--color-*` (bg, surface, text, text-muted, link, link-hover, border,
  focus, footer-*); `--space-1…8`; `--text-sm…3xl`; `--leading-*`; font stacks; `--radius`; `--container-max`.
- Dark mode in CSS only: re-declare `--color-*` in `@media (prefers-color-scheme: dark)` with F1's dark values.
- Classless prose: style what the Markdown renderer emits (headings, p, lists, blockquote, pre/code, table, img) under
  `main`, plus a few `site-*` shell classes (F1 D-04); content never needs a class (AC-0006.03).
- CI checks: no raw hex outside the token block (F1 T-S04) and the byte budget (F1 T-S05); `/style.css` keeps its URL.
- F1 §4.6 recomputed: all pairs match except dark focus (9.58:1, not 10.3:1; still ≥ 3:1). Gap: light link `#2b7a9e`
  on surface `#f5f6f7` is 4.43:1; `#276f94` (5.54 / 5.12:1) or brand green `#0b7261` (5.84 / 5.40:1) pass both.
- Underline inline links: light candidates are 2.2–2.7:1 against body text (dark link 1.85:1; footer yellow vs white
  1.55:1), below the 3:1 color-only links need ([G183](https://www.w3.org/WAI/WCAG21/Techniques/general/G183)); today
  only resource links are always underlined (`style.scss:55-58`), prose and footer links on hover (`:24-26`, `:68-71`).

## 5. Pages flip runbook (from F7 §4.5; an upstream admin runs it after a fork rehearsal; the fork has no Pages site yet)

Superseded by research 0003 §8, which Plan 0002 step 8 follows.

## 6. July 2026 prototype (the author's private notes; never pushed, so not verifiable here)

- July's commits were never pushed. CSS ≈ 14.8 KB vs ≈ 419 KB Bulma baseline (F1:43 expected 8–12 KB); a fresh clone
  ran in 2 steps without submodules. The baseline still matches: live `/style.css` is 429 277 B (419 KiB) today.
- The author approved a July design that turned links from blue to green (shade unrecorded); August D-07 chose a
  darker blue (Plan 0002, decision 2).
- Overridden in August: third-party JamesIves action, Atom+RSS 2.0, "clean academic" look, self-hosted Fira Sans (the author may revisit).

## Implications for the plan

- Step 0 (docs only): freeze the URL inventory as a test fixture: 11 sitemap URLs (incl. `/zulip/`), `static/` and
  workshop files, `404.html`, ids `/#who-are-we`, `/workshop/cfp-2022/#schedule`; `/resources/#books` lacks a target.
- The generator must cover what Zola did: Zola-style file and heading slugs (`campus_tour.md` → `/news/campus-tour/`),
  colocated assets (`content/workshop/`), sitemap, `404.html`, redirects, drafts; plus Atom and a relative-link check.
- Strict config and front-matter parsing: a misplaced key is an error with `path:line` ([Zola's top-level keys](https://www.getzola.org/documentation/getting-started/configuration/)).
- The resource catalog is three levels deep (`content/resources/_index.md:5-122`); its format sets our YAML subset.
- One Docker image (digest-pinned base + `rust-toolchain.toml`) builds locally and in CI through plain `docker` steps;
  `docker/*` actions are Docker Inc.'s; the Pages artifact may hold [no symlinks or hard links](https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages).
- std has no TLS ([`std::net`](https://doc.rust-lang.org/std/net/)): external link checks need `curl` in the image, or are dropped (Plan 0002: not in this plan).
- Keep wall-clock time out of the output (feed `updated` = newest post date) so local and CI builds match.
- Content fixes (#3/#9 entries, CFP link, `content/_index.md:13-15` markup, workshop nav) ship as small PRs (deploy: Plan 0002 step 1).

## Open questions

All answered in Plan 0002.
