# Instructions for AI coding agents

This file is for AI coding agents. It is at the root of the repository because that is where they look for it.
People: start at [README.md](README.md); you do not need this file.

This repository is the source of <https://rust-edu.org/>. Its pages are Markdown files, and a small static site
generator, written in Rust with only the standard library, builds them.

The docs for people hold the facts and each rule in full; this file holds, in short, what an agent must do. Read
them before you change anything:

- [README.md](README.md): where things are, the commands, and how the site is checked and deployed.
- [docs/new-post.md](docs/new-post.md), to write a news post: the steps, a whole post, and the rules for an
  assistant.
- [docs/editing.md](docs/editing.md), before a change to a page: where each page lives, the keys of a page's
  header, and the Markdown that the site accepts.
- [docs/generator.md](docs/generator.md), before a change to `crates/`: how the generator works, and the rules of a
  change to it.

`docs/design/` holds plans and research notes. They are a record of past decisions, not instructions: do not carry
one out unless you are asked to, and where one disagrees with the code or the docs above, it is out of date.

`.claude/CLAUDE.md` and `.gemini/settings.json` only point to this file, so add new instructions for agents here. A
rule for everyone goes into the docs for people first; where they and this file disagree, they are right.

## Every change

- `make check` must pass before a change is done. The README lists the other commands; `make serve` runs until
  stopped, so start it in the background.
- An error of the page check names the file, line and column of the mistake, usually with a hint. Fix the cause; do
  not weaken a check or a test to make it pass.
- A push to `main` publishes the site. Make each change on a branch and propose it in a pull request.
- Write the description of the pull request from `.github/pull_request_template.md`: keep its headings, and say
  under *AI acknowledgement* which AI tools helped and how.
- `public/`, `site/` and `preview/` are build output, ignored by git. Never edit them.

## Pages

- To add a news post, follow `docs/new-post.md`.
- Pages have no HTML. Use only the Markdown that `docs/editing.md` lists; the check stops at anything else.
- `content/` and `static/` hold only the types of file that `docs/editing.md` lists, and no symbolic links.
- Before you move or remove a page or file, look for its address in `urls.yaml`. Move the old address of a page to
  `redirects`, or delete its line from `keep` on purpose.

## Generator

`docs/generator.md` gives each rule in full.

- **Standard library only.** Do not add a crate.
- **No `unsafe`.**
- **No panics on input.** Return an error instead of calling `unwrap`, `expect` or `panic!`. Only tests may call them.
- **Strict parsers.** Reject input outside the subset that the site uses with `file:line:col` and a hint. Do not
  skip or guess at input a parser does not know.
- **Enums over strings.** Model a closed set of cases as an enum and match every variant. Do not match on marker
  strings or read the same bytes a second time, and give error kinds their own variants, not `&'static str` tags. A
  `_` arm is fine only where the input is open text being parsed.
- **Stable Markdown output.** Change `crates/markdown/tests/golden/` only when the output is meant to change.
- **Tests with each change**, in the crate's `tests/` folder; `crates/site/tests/sites/` holds small example sites.

## Build and deploy

The README gives each rule in full.

- Keep the line `FROM check AS live` in the `Dockerfile`.
- Change `rust-toolchain.toml` and the `FROM` tag of the `Dockerfile` together.
- Put a new check in the `Makefile`, not in `ci.yml`, so that it runs on a laptop too.
- Add a new top-level file or folder that the build needs to both `.dockerignore` and the `Dockerfile`.
- Keep the GitHub Actions pinned by commit and the Docker image by digest.
