# Plan 0002 — Own Markdown Site Generator

**Date:** 2026-10-28 · **Status:** steps 0–7 done · **Supersedes:** Spec 0001 and Plan 0001 (archived at tag
[`archive/initial-project-plan`](https://github.com/kenesparta/rust-edu.github.io/tree/archive/initial-project-plan/docs)
on the fork kenesparta/rust-edu.github.io) · **Research:**
[0001 inventory](../research/0001-current-site-inventory.md) ·
[0002 generator](../research/0002-std-only-generator-design.md) ·
[0003 Docker and Pages](../research/0003-docker-build-and-pages-deploy.md) ·
[0004 August plan](../research/0004-prior-plan-salvage.md)

**Goal:** anyone can update rust-edu.org by editing Markdown on GitHub. Our own small Rust generator builds the site
(standard library only: no crates, submodules, Sass or Zola) inside Docker, and GitHub Pages serves it.

## Decisions taken

| Topic | Decision |
|---|---|
| Generator | A Cargo workspace on the Rust standard library only; we own the Markdown, YAML-subset and template code |
| Content | Markdown with a short YAML header; no HTML; unsupported syntax fails the build with `file:line` and a hint |
| Design | One hand-written `static/style.css` and a few HTML templates in `layouts/`; changing them needs no Rust |
| Build | One Docker image for laptops and CI; plain `cargo` works too |
| Hosting | GitHub Pages, published by GitHub's own actions pinned by commit SHA |
| Transition | Zola keeps deploying until step 8 through `shalzz/zola-deploy-action` pinned by SHA, the one temporary exception to GitHub-only actions; every step is a small PR that leaves the site working |
| Workflow | Every step is a reviewed PR into `main` of the fork kenesparta/rust-edu.github.io; the finished work goes upstream as one PR from the fork's `main` to upstream `main` |
| Resources page | A Markdown list (decision 1 below); the 3 empty subcategories go |
| Zulip | The menu and the home page link to `/zulip/`, which redirects to `https://rust-edu.zulip.cs.pdx.edu` |
| Link color | Brand green `#0b7261` (decision 2) |
| JavaScript | None; the menu wraps on small screens (decision 3) |
| Names | Binary `edu`; crates `edu-yaml`, `edu-markdown`, `edu-template` and `edu-site` in `crates/{yaml,markdown,template,site}`; templates in `layouts/` (decision 5, 2026-11-01) |
| Code license | CC0 1.0 like the rest of the repository, `license = "CC0-1.0"` (decision 6, 2026-11-01) |
| Smart quotes | None: quotes and apostrophes render as typed, in Zola too (decision 4, 2026-11-03) |
| External links | `rel="external"` on every link to an `http:` or `https:` URL, as Zola writes it (2026-11-05) |
| Character references | Only `&amp;`, `&lt;`, `&gt;` and `&quot;`; any other fails the build with a hint to type the character itself (2026-11-05) |

## Editing with only Markdown

This is the workflow at the end of the plan; most of it already works after step 2. Every page is one `.md` file
under `content/`. A news post, for example `content/news/rust-week-2027.md`:

```markdown
---
title: Rust Week 2027
date: 2027-03-01
authors:
  - Jane Doe
---
Plain **Markdown**: headings, lists, quotes, [links](https://www.rust-lang.org/) and images.
```

- **On github.com:** open the folder and choose *Add file* (or the pencil on an existing file; without write access
  GitHub first asks you to *Fork this repository*), edit, check *Preview* (GitHub shows the header as a table), then
  *Commit changes… → Propose changes → Create pull request*. An automatic check builds the site and checks every
  internal link; a maintainer merges, and the site updates a few minutes later.
- **Also Markdown:** the resources page is a Markdown list; menu and footer links are a few lines of `site.yaml`.
- **Preview locally (optional):** `docker compose up`, then open http://127.0.0.1:1111; with Rust installed,
  `cargo run -p edu-site -- serve`.
- **Stable format:** headers use only keys Zola also understands (`title`, `template`, `draft`; pages also `date`,
  `authors`; sections also `sort_by`, `redirect_to`), and links between pages are root-relative (`/news/`), the one
  form Zola and our generator resolve alike. So content stops changing after step 2, and a rollback to Zola is a plain
  revert.

## Steps

One branch and one PR per step, from the fork's `main`, named `<type>/<kebab-description>` with Conventional Commit
types (for example `fix/pin-zola-deploy-action`). Each PR has one concern, keeps the site building and deploying, and
stays near 500 changed lines (600 at most), not counting content moves and deletions; bigger steps split as noted.
Each PR merges into the fork's `main` after review; the upstream PR comes after step 8 is rehearsed on the fork.

**Part A — on Zola, useful on its own**

0. *Done (fork PR #3).* `docs/markdown-site-plan` — this plan and `research/`. *Done when:* merged into the fork's
   `main` after step 1.
1. *Done (fork PR #2).* `fix/pin-zola-deploy-action` (~7 lines) — pin `shalzz/zola-deploy-action` to v0.22.1
   (`74526e6`) and `actions/checkout` to v7.0.1 by commit SHA, with per-job token permissions; the README names Zola
   0.22.1 for local builds. `@master` now installs Zola 0.23.6, which stops at `templates/macros.html:1`
   (`Unknown tag`), so every PR build and the next deploy fail. *Done when:* the PR build and the deploy that follows
   the merge are green.
2. *Done (fork PR #4).* `refactor/markdown-only-content` (~250, mostly moves) — YAML headers in all 11 files; the
   resources page becomes a Markdown list (the 3-level TOML data, `resources.html` and `macros.html` go; the 3 empty
   subcategories go; the entries of upstream PRs #3 and #9 are ported with `Co-authored-by` credit, and the upstream PR
   notes that it supersedes them); remove the 4 raw-HTML lines; fix 3 broken links (as root-relative URLs) and an
   `email:` link; news sorted newest first; `/zulip/` via `redirect_to`, with `content/zulip.md` moved to
   `content/zulip/_index.md` (Zola ignores the key on a page), and the menu and home page linking there; kebab-case file
   names (URLs unchanged); add `docs/editing.md`. *Done when:* Zola builds; the 11 page URLs are unchanged, `/zulip/`
   still redirects, `/news/` lists the posts newest first and bylines still show; `content/` has no HTML tags; the PR
   shows before/after screenshots of `/`, `/news/` and `/resources/`.
3. *Done (fork PR #5).* `refactor/plain-css` (~600 + deletions) — one hand-written `static/style.css` at the same URL
   (≤ 50 KB; today 429 KB): design tokens, dark mode by media query, classless styles for Markdown output, WCAG AA
   contrast; Bulma-free templates with a skip link and visible focus, no JavaScript; delete `ext/`, `.gitmodules`,
   `sass/` and `static/js/site.js`; fix `config.toml` (`base_url` → `https://rust-edu.org`, `compile_sass = false`,
   mis-scoped keys); README without the submodule step. *Done when:* a fresh clone builds with no submodule step; the PR
   shows before/after screenshots.

**Part B — our generator, built next to Zola and not deployed yet**

Code follows our Rust rules (no `unwrap`, `expect` or `panic!` on input paths, no `mod.rs`, the workspace lints of
research 0002 §1) with two std-only exceptions: hand-written error enums instead of `thiserror`/`anyhow`, and a seeded
random-input test instead of a property-testing crate. Part B is about 8,400 lines including tests in about
twenty-two PRs; research 0002 §9 sizes a broader subset at ~4,650.

4. *Done (fork PRs #6, #7 and #8).* `feat/edu-workspace` (~1,250, three PRs: workspace, Docker and CI; YAML maps,
   scalars and errors; YAML lists, page headers and `edu check`) — a workspace with zero dependencies,
   `rust-toolchain.toml` (1.99.1) and workspace lints, where each crate of `crates/{yaml,markdown,template,site}`
   arrives with its first code (`edu-site` and the `edu` binary first, `edu-yaml` in the second PR, the other two in
   steps 5 and 6); a `Dockerfile` whose base image is pinned by digest; `ci.yml` runs fmt, clippy, the tests and an
   offline build inside Docker; the YAML subset; a first `edu check` that parses every header in `content/`.
   *Done when:* CI is green, `Cargo.lock` lists only our own crates, and a CI check fails when the Dockerfile's `FROM`
   tag and `rust-toolchain.toml` name different Rust versions.
5. *Done (fork PRs #9 to #14).* `feat/edu-markdown` (~2,500, six PRs: paragraphs and headings; fenced code, quotes
   and lists; code spans, autolinks, escapes and line breaks; links and images; emphasis and errors; `edu check` on
   page bodies and golden tests) — the Markdown the content uses (research 0001 §1) plus fenced code and numbered
   lists, with heading ids as Zola makes them for headings whose letters and digits are ASCII (all 30 today);
   anything else is an error.
   *Done when:* one golden test per content file passes, its HTML diffed against Zola 0.22.1's output with each
   difference explained in the PR; rejected syntax has error tests; the seeded random-input test finds no panic.
6. *Done (fork PRs #15 to #20).* `feat/edu-template` (~2,050, six PRs: values, `{{ }}` and comments; `if`, `else`
   and `for`; the ported layouts and `site.yaml`; the values of each page; the header's keys; rendering every page)
   — `{{ x }}` (escaped), `{{ x | safe }}`, `if`/`else`, `for` and one layout; the step-3 templates ported to
   `layouts/` (Zola keeps `templates/`), menu and footer links in `site.yaml`; the header keys of the stable format
   above, each checked, and `sort_by: date`. *Done when:* every page renders and template errors name the file and
   line.
7. *Done (fork PRs #21 to #27).* `feat/edu-build` (~2,600, seven PRs: build and assets; redirects, sitemap,
   robots and 404; drafts and `--base-url`; check; the site in memory; serve; `compose.yaml` and the preview of each
   PR) — `edu build` (pretty URLs, assets, `redirect_to` pages, drafts only with `--drafts`, `sitemap.xml`,
   `robots.txt`, `404.html`, `--base-url` for fork previews), `edu check` (internal links, anchors, and the
   URL-parity list in research 0001, as `urls.txt`), `edu serve` (std::net, rebuilds on request); `compose.yaml`;
   `ci.yml` attaches the `--drafts` build to each PR. *Done when:* the checks pass on real content and two builds are
   byte-identical.

**Part C — switch over**

8. `ci/pages-deploy` (~200 + deletions, two PRs) — `deploy.yml` on GitHub's Pages actions, building in Docker and
   rehearsed on the fork's own Pages site (with `--base-url`); the second PR deletes `config.toml`, `templates/` and
   `main.yml`; the README and `docs/editing.md` switch from Zola to `docker compose up` and `edu build`, with the badge
   on the new workflow; Dependabot for `github-actions`, and for `docker` plus `rust-toolchain` in one
   multi-ecosystem group, so a Rust release bumps the image and `rust-toolchain.toml` in one PR. *Done when:* the
   fork's Pages site serves the new build.
9. **The upstream PR** — the fork's `main` to upstream `main`; an upstream admin sets Settings → Pages → Source to
   *GitHub Actions* (checklist in the PR) and merges it in the same sitting. *Done when:* rust-edu.org serves the new
   build. Rollback: revert and flip the source back; `gh-pages` stays for two weeks.

## Decisions for review (default first)

1. *Decided 2026-10-28: a Markdown list.* Resources page: **a Markdown list** (anchors and GitHub preview for free),
   or keep structured YAML data (nested lists in the header plus a dedicated template).
2. *Decided 2026-10-28: brand green.* Link color: **brand green `#0b7261`** (today's `$green`; 5.84:1 on white), or
   darker blue `#276f94` (5.54:1).
3. *Decided 2026-10-28: none.* JavaScript: **none** (the menu wraps on small screens), or a small disclosure menu.
4. *Decided 2026-11-03: dropped.* Smart quotes: **dropped** (about 12 lines render as typed once resources are
   Markdown), or implemented (~60 lines).
5. *Decided 2026-11-01: as proposed.* Names: **binary `edu`; crates `edu-yaml`, `edu-markdown`, `edu-template`,
   `edu-site`; templates in `layouts/`**.
6. *Decided 2026-11-01: CC0.* Code license: **CC0 like the rest of the repo**, or MIT/Apache-2.0 for `crates/`.
7. Pages source: **GitHub Actions, flipped once by an upstream admin**, or a plain-git push to `gh-pages` (no admin).

## Not in this plan

Atom feed (`/atom.xml`, upstream issue #6; the first follow-up), syntax highlighting, search, live reload, scheduled
posts, and external link checks (std has no TLS).

## Risks

- **Upstream may not want a custom generator.** Everything is built and rehearsed on the fork first, so upstream
  reviews a working result; Part A stands on its own and can go upstream alone if Part B is declined.
- **Parser bugs.** Strict subsets: unsupported input is an error, never a guess; golden tests use real content and are
  diffed against Zola's output.
- **Maintenance and bus factor.** Four small crates, a README that documents build, check and serve, tests as the
  spec; no dependency updates ever, only action pins, the base image and the Rust toolchain (bumped together).
- **The Pages flip needs an admin.** Name that admin before step 9; if nobody agrees, decision 7 takes the
  `gh-pages` push (no settings change).
