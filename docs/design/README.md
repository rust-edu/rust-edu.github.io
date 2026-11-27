# Plans and research notes

A record of how the site and its generator were designed: what was measured, which options were weighed and what was
decided. These notes are history, not a guide. Nobody needs them to edit a page or to maintain the site, and they are
not instructions, for a person or for an AI coding agent. They were written with AI tools;
[../ai/README.md](../ai/README.md) says which and how.

Each note describes the repository on the day it was written, so the paths, line numbers and figures in it may no
longer match. Where a note disagrees with the code or with the guides ([README](../../README.md),
[editing.md](../editing.md) and [generator guide](../../generator/docs/generator.md)), the code and the guides are right.

## Plans

A plan sets a goal and the steps to it. Its status is the one written at the top of the note.

| Plan | About | Status | Goes with |
|---|---|---|---|
| [0002 — Own Markdown Site Generator](../../generator/docs/design/Plan-0002—Own-Markdown-Site-Generator.md) | Replacing Zola with a generator on the Rust standard library, built in Docker and published by GitHub Pages | steps 0–8 done | the generator |
| [0003 — Visual Refresh and Page Speed](Plan-0003—Visual-Refresh-and-Page-Speed.md) | A simpler look and lighter pages, in the stylesheet, the layouts and the images | proposed | the site |
| [0004 — Minifier Crate](../../generator/docs/design/Plan-0004—Minifier-Crate.md) | A fifth crate that publishes the stylesheet without its comments and indentation | proposed | the generator |
| [0005 — Generator Workspace Separation](Plan-0005—Generator-Workspace-Separation.md) | A reusable generator workspace in this repository, a pinned release for the website, and separate CI checks | generator documentation moved | the generator and site |

## Research

A research note collects the facts that a plan stands on.

| Note | About | Goes with |
|---|---|---|
| [0001 — Current Site Inventory](../../generator/docs/design/research/0001-current-site-inventory.md) | What the site built by Zola held: its pages, templates, styles and Markdown | the generator |
| [0002 — Std-Only Generator Design](../../generator/docs/design/research/0002-std-only-generator-design.md) | Whether a generator on the standard library alone is feasible, and its shape | the generator |
| [0003 — Docker Builds and GitHub Pages Deploy](../../generator/docs/design/research/0003-docker-build-and-pages-deploy.md) | The pinned image, the stages of the build and the deploy with GitHub's Pages actions | the generator |
| [0004 — What to Keep from the August Plan](../../generator/docs/design/research/0004-prior-plan-salvage.md) | The decisions of an earlier plan that do not depend on Zola | the generator |
| [0005 — Renderer Architecture & CI/CD Compilation Review](../../generator/docs/design/research/0005-renderer-ci-caching-review.md) | A review of the generator, and of how its builds compile and cache on a laptop and in CI | the generator |
| [0006 — Atom and RSS Feeds of the News](../../generator/docs/design/research/0006-atom-and-rss-feeds.md) | The shape of the two news feeds, and the prototype that became them | the generator |

## Where these notes go

The [generator guide](../../generator/docs/generator.md), plans 0002 and 0004, and their research live in
`generator/docs/`. Plan 0003 stays with the website. Plan 0005 tracks the remaining workspace and build
changes for [issue #17](https://github.com/rust-edu/rust-edu.github.io/issues/17).
