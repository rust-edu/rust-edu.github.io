# Instructions for AI coding agents

This repository is the source of <https://rust-edu.org/>. Its pages are Markdown files, and a small static site
generator, written in Rust with only the standard library, builds them.

Read these before you change anything:

- [README.md](README.md): how to run the site and how the generator works.
- [docs/editing.md](docs/editing.md): where each page lives, the keys of a page's header, and the Markdown that the
  site accepts.

`CLAUDE.md` and `.gemini/settings.json` only point to this file, so add new instructions here.

## Commands

| Command | What it does |
|---|---|
| `make check` | Checks format and lint, runs the tests, and checks every page and link. It must pass before a change is done. |
| `make ci` | Runs the same checks in CI's pinned Docker image, and builds the site twice to see that both builds are the same. It needs Docker. |
| `make serve` | Serves the site with its drafts at <http://127.0.0.1:1111>, and builds it again on a reload after a change. It runs until stopped, so start it in the background. |

An error of the page check names the file, line and column of the mistake, usually with a hint. Fix the cause; do not
weaken a check or a test to make it pass.

## Where things are

| Path | What it holds |
|---|---|
| `content/` | The pages, one Markdown file each, and the files they link to, such as PDFs |
| `layouts/` | The HTML layout of each kind of page |
| `static/` | `style.css` and the images |
| `site.yaml` | The site's address, title and description, and the menu, next-step (`hero`) and footer links |
| `urls.txt` | The addresses that other sites may link to |
| `crates/` | The generator: `yaml`, `markdown`, `template` and `site`, which has the `edu` binary |
| `docs/`, `research/` | The editing guide, and the generator's plan and research notes |
| `public/`, `site/`, `preview/` | Build output, ignored by git. Never edit it. |

## Pages

- Pages have no HTML. Use only the Markdown that `docs/editing.md` lists; the check stops at anything else.
- `content/` and `static/` hold only the types of file that `docs/editing.md` lists, and no symbolic links.
- Before you move or remove a page or file, look for its address in `urls.txt`. Keep the old address of a page with
  an `_index.md` that has `redirect_to`, or delete its line on purpose.

## Generator

- **Standard library only.** Do not add a crate. CI fails when `Cargo.lock` lists a crate from a registry or git.
- **No `unsafe`.** The workspace forbids it.
- **No panics on input.** Return an error instead of calling `unwrap`, `expect` or `panic!`. Only tests may call them.
- **Strict parsers.** Each parser accepts only the subset of YAML, Markdown or the template language that the site
  uses, and rejects anything else with `file:line:col` and a hint. Do not skip or guess at input it does not know.
- **Enums over strings.** Model a closed set of cases as an enum and match every variant. Do not match on marker
  strings or read the same bytes a second time, and give error kinds their own variants, not `&'static str` tags. A
  `_` arm is fine only where the input is open text being parsed.
- **Stable Markdown output.** `crates/markdown/tests/golden/` holds the expected HTML of the site's pages. Change it
  only when the output is meant to change.
- **Tests with each change.** Each crate has a `tests/` folder, and `crates/site/tests/sites/` holds small example
  sites.

## Build and deploy

- A push to `main` publishes the site. Make each change on a branch and propose it in a pull request.
- `rust-toolchain.toml` and the `FROM` tag of the `Dockerfile` name the same Rust. Change them together; CI fails
  when they differ.
- `.dockerignore` is an allowlist and the `Dockerfile` copies paths by name, so a new top-level file or folder that
  the build needs has to be added to both.
- The GitHub Actions are pinned by commit and the Docker image by digest. Keep them pinned.
