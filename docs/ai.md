# AI and this repository

For people: which AI tools helped to make this repository, what is asked of a contributor who uses one, and which
files are written for the tools and not for you. Nothing on this page is needed to edit a page or to maintain the
site.

## Acknowledgement

Most of the generator in `crates/`, its tests, the layouts, the stylesheet and the docs were written with Claude Code
(Anthropic, model Claude Opus 5.5) from October 2026, at the direction of Ken Esparta, who read each change in its
[pull request](https://github.com/kenesparta/rust-edu.github.io/pulls?q=is%3Apr+is%3Amerged) on the fork
kenesparta/rust-edu.github.io. The docs include the plans and research notes of [design/](design/README.md). The
changes to the logo that keep its colors in the dark theme, and the tab icon drawn from it, were made with Claude Code
and the model Claude Fable 5.2, as was the sorting of the docs into those for people, those for AI coding agents and
the design notes. An earlier plan from August 2026, which Plan 0002 replaced, was drafted with Claude Fable 5; the fork
keeps it at the tag `archive/initial-project-plan`. Gemini CLI was used only for reviews and questions, and none of
its output is in the repository.

## When you use an AI tool

Say in the description of your pull request which tools helped and how. The checks and the review are the same for
every change, whoever or whatever wrote it.

## The files for AI coding agents

| File | Read by | What it holds |
|---|---|---|
| [`AGENTS.md`](../AGENTS.md) | AI coding agents | The instructions for an agent that works in this repository: what to read first and the rules of a change, in short. |
| `.claude/CLAUDE.md` | Claude Code | One line that points to `AGENTS.md`. |
| `.gemini/settings.json` | Gemini CLI | One setting that names `AGENTS.md` as the file of instructions. |

`AGENTS.md` is at the root of the repository because that is where the tools look for it, as <https://agents.md/>
describes; the other two are in the folders of their tools. A person does not need any of them: each rule of
`AGENTS.md` is also in the [README](../README.md), [editing.md](editing.md) or [generator.md](generator.md), in
full.

A new rule goes into those docs for people, and into `AGENTS.md` in short when an agent has to follow it too. Where
the two disagree, the docs for people are right.

## The page for language models

`/llms.txt` is the site in brief for language models, in the Markdown that <https://llmstxt.org/> proposes: the
`title` and `description` of `site.yaml`, a link to each page with its title, a link to each news post with its date
and authors, newest first, and the two feeds, which hold the whole text of every post. It leaves out the redirects and
the drafts. The build writes it again each time, so a new page or post needs nothing more than its file.

It is part of the published site, for the models that read rust-edu.org, and has nothing to do with the files above,
which are for the agents that work in this repository.
