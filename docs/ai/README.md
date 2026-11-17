# AI and this repository

For people: which AI tools helped to make this repository, what is asked of a contributor who uses one, and which
files are written for AI tools rather than for human readers. Nothing in this folder is needed to edit a page or to
maintain the site.

## Acknowledgement

Most of the generator in `crates/`, its tests, the layouts, the stylesheet and the docs were written with Claude Code
(Anthropic, model Claude Opus 5.5) from October 2026, at the direction of Ken Esparta, who read each change in its
[pull request](https://github.com/kenesparta/rust-edu.github.io/pulls?q=is%3Apr+is%3Amerged) on the fork
kenesparta/rust-edu.github.io. The docs include the plans and research notes of [design/](../design/README.md). The
changes to the logo that keep its colors in the dark theme, and the tab icon drawn from it, were made with Claude Code
and the model Claude Fable 5.2, as were the initial split of the docs, the guide to a news post and the template of a
pull request. The white lettering and page-colored hover of the feed buttons, `urls.yaml`, the 404 page, and the review
and separation of the human and LLM documentation in #19 were made with Antigravity (Google DeepMind). An earlier plan
from August 2026, which Plan 0002 replaced, was drafted with Claude Fable 5; the fork keeps it at the tag
`archive/initial-project-plan`. Gemini CLI was used only for reviews and questions, and none of its output is in the
repository.

## When you use an AI tool

Say in the description of your pull request which tools helped and how; its template has the heading *AI
acknowledgement* for this. The checks and the review are the same for every change, whoever or whatever wrote it.

### Drafting a news post with an assistant

An assistant can draft a news post from your notes; you are still the author, so read what it wrote and check each
name, date and link. Give it [new-post.md](new-post.md) (the LLM specification in this folder, separate from the human
guide [../new-post.md](../new-post.md)) and your notes:

```text
Read https://raw.githubusercontent.com/rust-edu/rust-edu.github.io/main/docs/ai/new-post.md and follow it to write a
news post for rust-edu.org from my notes. Give me the file path and the complete file in one code block. Ask me for
anything missing; do not make anything up.

My notes: …
```

## The files for AI tools

| File | Read by | What it holds |
|---|---|---|
| [`AGENTS.md`](../../AGENTS.md) | AI coding agents | Self-contained repository specification and rules for coding agents working in this repository. |
| [`docs/ai/new-post.md`](new-post.md) | LLMs / AI assistants | Standalone prompt and format specification for drafting a news post in `content/news/`. |
| `.claude/CLAUDE.md` | Claude Code | One line (`@../AGENTS.md`) that imports `AGENTS.md`. |
| `.gemini/settings.json` | Gemini CLI | One setting that names `AGENTS.md` as the context file. |

`AGENTS.md` is at the root of the repository because that is where coding agents look for it, as <https://agents.md/>
describes; `.claude/CLAUDE.md` and `.gemini/settings.json` sit in the folders of their tools, and the remaining AI
documentation lives here under `docs/ai/`.

People do not need the agent files: the human documentation — [README.md](../../README.md),
[docs/new-post.md](../new-post.md), [docs/editing.md](../editing.md) and [docs/generator.md](../generator.md) — covers
every workflow and rule for human readers, while `AGENTS.md` and `docs/ai/new-post.md` present those same rules in a
compact, machine-oriented form for LLMs. A new rule goes into the human docs first, and into `AGENTS.md` when an agent
must follow it too; where the two disagree, the human docs and the code are right. When the generator moves to its own
repository ([issue #17](https://github.com/rust-edu/rust-edu.github.io/issues/17)), the *Generator* section of
`AGENTS.md` moves there with `crates/` and [docs/generator.md](../generator.md).

## The page for language models

`/llms.txt` is the site in brief for language models, in the Markdown that <https://llmstxt.org/> proposes: the
`title` and `description` of `site.yaml`, a link to each page with its title, a link to each news post with its date
and authors, newest first, and the two feeds, which hold the whole text of every post. It leaves out the redirects and
the drafts. The build writes it again each time, so a new page or post needs nothing more than its file.

It is part of the published site, for models that read rust-edu.org, and is separate from the files above, which are
for tools used while editing or maintaining this repository.
