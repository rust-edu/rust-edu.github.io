# Rust-edu
[![Deploy](https://github.com/rust-edu/rust-edu.github.io/actions/workflows/deploy.yml/badge.svg)](https://github.com/rust-edu/rust-edu.github.io/actions/workflows/deploy.yml)
[![Zulip](https://img.shields.io/badge/chat-on%20Zulip-9146FF?logo=zulip&logoColor=white)](https://rust-edu.zulip.cs.pdx.edu/)


Rust Edu is an organization dedicated to supporting Rust education in the academic environment. This repository is
the source of its website, <https://rust-edu.org/>: the pages are Markdown files, and the site's own small static site
generator, written in Rust with only the standard library, builds them.

## 📚 Documentation
These are written for people:

| To | Read |
|---|---|
| Write a news post, the site's blog | [docs/new-post.md](docs/new-post.md) |
| Change another page, or add a resource | [docs/editing.md](docs/editing.md) |
| Run, check, preview or deploy the site | this file |
| Understand or change the generator in `crates/` | [docs/generator.md](docs/generator.md) |
| See which AI tools were used here, and how | [docs/ai.md](docs/ai.md) |
| Look up why the site is built this way | [docs/design/](docs/design/README.md): plans and research notes, a record and not a guide |

[AGENTS.md](AGENTS.md) is written for AI coding agents. A person does not need it: each of its rules is also in the
docs above, in full.

## 🚀 Setting Up on Local Machine
To contribute or run this website locally, follow the steps below.

To change only the content, see [docs/editing.md](docs/editing.md): every page is a Markdown file under `content/`.

### ✅ Requirements
- [Docker](https://docs.docker.com/get-started/get-docker/) with Compose, which runs the generator in the image that
  CI and the deploy use
- or [Rust](https://www.rust-lang.org/tools/install), with the toolchain pinned in `rust-toolchain.toml`

## 📦 Installation Steps
Clone the repository:
```bash
git clone https://github.com/rust-edu/rust-edu.github.io.git
cd rust-edu.github.io
```

## 💻 Run Locally
Start a local server:
```bash
docker compose up
```
Your site will be live at: http://127.0.0.1:1111, drafts too.

After a change, reloading the page builds the site again; while the site has errors, the page shows them instead.
With Rust installed, `make serve` does the same.

## 📁 Where Things Are
| Path | What it holds |
|---|---|
| `content/` | The pages, one Markdown file each, and the files they link to, such as PDFs |
| `layouts/` | The HTML layout of each kind of page; `layout.html` also holds the two short scripts of the theme button, the site's only JavaScript (without it, the site follows the system's theme) |
| `static/` | The images and `style.css`, the whole stylesheet: plain CSS with its colors in token blocks at the top, light and dark |
| `site.yaml` | The site's address, title and description, and the menu, next-step (`hero`) and footer links |
| `urls.yaml` | The addresses that other sites may link to (`keep`) and the addresses that redirect elsewhere (`redirects`) |
| `crates/` | The generator: `yaml`, `markdown`, `template` and `site`, which has the `edu` binary |
| `docs/` | The guides of the table above, and the plans and research notes in `docs/design/` |
| `public/`, `site/`, `preview/` | Build output, ignored by git. Never edit it. |

## 🦀 The Generator
A small generator in `crates/`, four crates on the Rust standard library alone, checks every page and link and
builds the site. [docs/generator.md](docs/generator.md) explains how it works and gives the rules of a change to it.

## 🔧 Checking and Deploying

### Check locally
Each job is one command of the `Makefile`:

| Command | What it does |
|---|---|
| `make check` | Runs the guards, checks format and lint, checks every page and link, then runs the tests. It needs Rust, and it must pass before a change is done. |
| `make guards` | Checks three rules that the build cannot see: `Cargo.lock` lists only our crates, the live stage is `FROM check`, and the `Dockerfile` and `rust-toolchain.toml` name the same Rust. |
| `make ci` | Runs what CI runs on a pull request: the guards, then the same checks in CI's pinned Docker image, where the site is built twice to see that both builds are the same. It needs Docker. |
| `make preview` | Builds the site with its drafts into `preview/`, as CI does for the reviewers of a pull request. It needs Docker. |
| `make serve` | Serves the site with its drafts at <http://127.0.0.1:1111>, and builds it again on a reload after a change. It needs Rust, and runs until stopped. |

An error of the page check names the file, line and column of the mistake, usually with a hint. Fix the cause; do not
weaken a check or a test to make it pass.

With [Rust](https://www.rust-lang.org/tools/install) installed, `rust-toolchain.toml` selects the pinned toolchain,
and the generator's commands run by themselves too:
```bash
cargo run -p edu-site -- check
cargo test
```
`edu build` writes the site into `public/`, for the `base_url` of `site.yaml` or for the address of `--base-url`:
```bash
cargo run -p edu-site -- build
```

### Preview of a pull request
The checks of each pull request attach the site with its drafts as `site-preview`; its links start at the top of a
site, so serve the unpacked folder at the root of an address, as with `python3 -m http.server -d site-preview 1111`,
rather than opening its files. `make preview` writes the same site into `preview/`.

### Deploy
`.github/workflows/deploy.yml` runs on each push to `main`: it checks and builds the site in Docker and publishes it
with GitHub's own Pages actions, at the address that Pages gives the site, which the build takes as `--base-url`. It
needs *Settings → Pages → Source* set to *GitHub Actions*; a fork with that setting gets the site at its own Pages
address. *Actions → Deploy to GitHub Pages → Run workflow* runs it again by hand. Locally,
`docker build --target site -o site .` writes the same site, for rust-edu.org, into `site/`.

A push to `main` publishes the site, so make each change on a branch and propose it in a pull request, whose
description starts from `.github/pull_request_template.md`. These keep a bad change from going live:

- `FROM check AS live` in the `Dockerfile` builds the live site on the stage that ran the checks, which is what
  stops a bad push from publishing. Keep it; the guards fail when it changes.
- `rust-toolchain.toml` and the `FROM` tag of the `Dockerfile` name the same Rust. Change them together; the guards
  fail when they differ.
- CI runs `make ci` and `make preview`. `ci.yml` adds only what belongs to GitHub: when the checks run, what they
  may read, the errors shown on the lines of a pull request, and the preview as a download. Put a new check in the
  `Makefile`, so that it runs on a laptop too.
- `.dockerignore` is an allowlist and the `Dockerfile` copies paths by name, so a new top-level file or folder that
  the build needs has to be added to both.
- The GitHub Actions are pinned by commit and the Docker image by digest. Keep them pinned: Dependabot proposes
  updates of the actions, and of the Docker image together with `rust-toolchain.toml`, so that a Rust release bumps
  both in one pull request.

## 🤖 AI Acknowledgement
Most of the generator in `crates/`, its tests, the layouts, the stylesheet and the docs were written with Claude Code
(Anthropic), at the direction of Ken Esparta, who read each change in its pull request. [docs/ai.md](docs/ai.md) has
the whole acknowledgement, with the models and what each tool did, and says what is asked of a contributor who uses an
AI tool.

## 📬 Get Involved
Interested in contributing or learning more?
* 🌐 Visit: [https://rust-edu.org/](https://rust-edu.org/)
* 📰 Follow the news in a feed reader: [Atom](https://rust-edu.org/atom.xml) or [RSS](https://rust-edu.org/rss.xml)
* 📧 Email: [contact@rust-edu.org](mailto:contact@rust-edu.org)
Let’s build the future of Rust education together!