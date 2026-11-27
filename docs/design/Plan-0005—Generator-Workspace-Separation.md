# Plan 0005 — Separate the generator from the website content

**Date:** 2026-11-26 · **Status:** proposed · **Issue:**
[#17](https://github.com/rust-edu/rust-edu.github.io/issues/17)

An editor should be able to fix a news post and preview it with the same generator that builds the
live site. A Rust developer should be able to install `edu`, build another site, and contribute a
parser change without publishing rust-edu.org. Both can work in this repository.

The existing `edu-site` package already provides the `edu` binary and the library entry point
`edu_site::run`. Its three library dependencies are `edu-markdown`, `edu-template` and `edu-yaml`.
The work is to give those four crates their own workspace directory and release process, make
their tests independent of rust-edu.org, and let the website choose which release to use.

The first decision for maintainers is whether to adopt the same-repository layout. If they accept
it, align the title and remaining checklist in issue #17 with that decision and open the move PR.

## 1. Move the workspace in one pull request

- Move `Cargo.toml`, `Cargo.lock`, `clippy.toml`, `rustfmt.toml`, the four crates and their tests
  to `generator/`. Moving `clippy.toml` and `rustfmt.toml` with the workspace keeps test lint
  allowances (`allow-unwrap-in-tests`, `allow-expect-in-tests`, `allow-panic-in-tests`), MSRV and
  formatting identical whether Cargo runs inside `generator/`, via `--manifest-path`, or from an
  isolated build context. Include a copy of `LICENSE` (`CC0-1.0`) in `generator/` so standalone
  exports carry the license declared in `generator/Cargo.toml`.
- Give `generator/` a `README.md` covering installation (`cargo install --path generator/crates/site --locked`),
  the `edu_site::run` library entry point, and the site directory contract (`edu check`,
  `edu build` and `edu serve` always read the current working directory `.` and take no root-path
  flag). Add an allowlist `generator/.dockerignore` (`*`, `!Cargo.toml`, `!Cargo.lock`,
  `!clippy.toml`, `!rustfmt.toml`, `!crates`) so passing `generator/` as a Docker build context
  never uploads host `generator/target/` artifacts.
- Move [generator.md](../generator.md), [Plan 0002](Plan-0002—Own-Markdown-Site-Generator.md),
  [Plan 0004](Plan-0004—Minifier-Crate.md) and the six [research notes](README.md#research) into
  `generator/docs/`. Fix relative links across the boundary (including `docs/design/README.md` and
  `Plan 0004`'s links to [Plan 0003](Plan-0003—Visual-Refresh-and-Page-Speed.md)), update human
  guides, and keep root [`AGENTS.md`](../../AGENTS.md) (imported by `.claude/CLAUDE.md` and
  `.gemini/settings.json`) aligned while placing generator-specific rules where standalone
  contributors also see them. The site keeps `content/`, `layouts/`, `static/`, `site.yaml`,
  `urls.yaml`, the editing guides, [Plan 0003](Plan-0003—Visual-Refresh-and-Page-Speed.md) and
  this plan.
- Update the root `Makefile`, `Dockerfile`, `.dockerignore` and `.gitignore` (`/generator/target/`
  in addition to `/target`). Because `edu` reads `.`, root `Makefile` commands must invoke Cargo
  from the repository root with `--manifest-path generator/Cargo.toml` (not `cd generator && cargo run`),
  preserving repo-root-relative `file:line:col` paths for PR annotations. Update `guard-lock` to
  read `generator/Cargo.lock` and report `generator/Cargo.lock:<line>:1: error: …`. Keep the shared
  `rust-toolchain.toml` at the root (where native builds and Dependabot's `rust` group read it),
  the four-package zero-source lockfile guard, the digest-pinned base image and the
  `FROM check AS live` gate.
- Decouple the production feed test in `crates/site/src/build.rs`, which currently walks two
  directories up (`ancestors().nth(2)`) to build rust-edu.org. Assert canonical feed and entry
  `<id>` / `<guid>` preservation alongside rebased `<link rel="alternate">` URLs on the existing
  `feed` fixture inside `generator/`. Add an explicit site check (run by both `make check` and the
  Docker `check` stage) that builds the real site with `--drafts --base-url https://example.github.io/rust-edu/`
  into a temporary folder and verifies that `atom.xml` IDs and `rss.xml` GUIDs keep
  `https://rust-edu.org` while alternate links rebase under `/rust-edu`.

**Ready to merge when** `make check` and `make ci` pass, the generator suite (`fmt`, `clippy`,
`test`) runs in a temporary copy containing only `generator/` and `rust-toolchain.toml` (no website
files), and the installed CLI checks and builds a copy of the `valid` fixture outside the
repository. Run the old and new generators on the same site inputs and compare the output trees
byte for byte for a live build, a draft preview and a fork served under a path. An isolated trial
on `b822727` passed 160 tests and all three output comparisons. Repeat those comparisons against
the merge base of the implementation PR.

## 2. Release the generator and pin the website to it

1. **Tag the initial release and verify external consumption.**
   - Ensure `[workspace.package] version` in `generator/Cargo.toml` matches the intended tag
     (`edu --version` prints `CARGO_PKG_VERSION`) and `generator/Cargo.lock` is in sync before
     tagging, otherwise `--locked` installs fail.
   - Tag the merged commit from step 1 as `generator-v0.1.0`. Git tags work directly with the
     workspace's `publish = false` setting, and Cargo discovers workspace members inside
     `generator/` automatically.
   - Document and test both external forms outside this repository:
     `cargo install --git https://github.com/rust-edu/rust-edu.github.io --tag generator-v0.1.0 --locked edu-site`
     and a separate Rust crate depending on `edu-markdown` (or `edu-site`) via `git` + `tag`.

2. **Pin the website via `generator.rev` and materialize the pinned context safely.**
   - Record the full 40-character lowercase hex commit SHA in `generator.rev` at the repository
     root.
   - Add a `guard-rev` target to `make guards` that validates `generator.rev` as a single line
     matching `^[0-9a-f]{40}$` (emitting `generator.rev:1:1: error: …` on failure, rejecting short
     SHAs, ref names, comments or leading `-` flags).
   - Resolve the pin with a single shared script or Make recipe used by local Make targets,
     Compose, PR previews and Pages deployments:
     1. Check `git cat-file -e "<rev>^{commit}"` first so repeated local runs work offline.
     2. Only when missing (such as `actions/checkout`'s default `fetch-depth: 1` shallow clone with
        `persist-credentials: false`), run `git fetch --depth=1 origin <rev>`. GitHub's upload-pack
        only serves raw 40-char OIDs reachable from an advertised ref, so every `generator.rev` on
        `main` must be an ancestor of `main` (or a pushed `generator-v*` tag); keeping it on `main`
        ensures forks created with GitHub's default *"Copy the `main` branch only"* option can
        fetch `<rev>` from their own `origin`.
     3. Extract `generator/` into an ignored cache path (for example `.cache/generator/<rev>/`) via
        `git archive "<rev>:generator"`, failing with a clear diagnostic if the revision is
        unreachable or predates `generator/`.
     4. Run the four-package zero-source `Cargo.lock` guard on **both** `generator/Cargo.lock` and
        the extracted pinned `Cargo.lock`, so `generator.rev` cannot point to an unguarded commit
        carrying external dependencies.

3. **Keep website builds immune to future toolchain lints on historical pins.**
   - Dependabot updates the single root `Dockerfile` (`FROM rust:<ver>-…`) and
     `rust-toolchain.toml` on new Rust releases. Running `cargo clippy -- -D warnings` or
     `cargo fmt --check` against an immutable historical `generator.rev` during website builds
     would allow a new compiler lint to break content PRs and Pages deployments.
   - Keep **one** root `Dockerfile` (preserving the single `guard-rust` pin) with distinct stages:
     - `check` (website): receives the extracted pin via `--build-context generator=<pinned-dir>`
       (with `!generator` removed from root `.dockerignore` so working-tree `generator/` edits
       cannot leak into or bust cache for pinned site builds), compiles the CLI with
       `--network=none cargo build --release --locked --offline -p edu-site`, and runs `edu check`,
       the fork-`BASE_URL` feed-ID check and the double-build reproducibility `diff` with
       `--network=none`. `live` remains `FROM check AS live` (`guard-live`).
     - `generator-check` (candidate workspace): receives `--build-context generator=generator` and
       runs `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --locked --offline -- -D warnings`
       and `cargo test --workspace --locked --offline` with `--network=none`.
   - Keep `rust-toolchain.toml`'s channel at or above `rust-version` in both the pinned and
     working-tree `generator/Cargo.toml`.

4. **Align local Docker Compose, native previews, upgrade order and security triage.**
   - Plain `docker compose up` cannot run `git fetch` or `git archive` before building
     `compose.yaml`. Either wire `compose.yaml`'s `additional_contexts.generator` to the path
     materialized by Make (failing fast via `${GENERATOR_CONTEXT:?…}` with a clear hint when unset)
     and update [README.md](../../README.md), [editing.md](../editing.md) and
     [new-post.md](../new-post.md) together if the local preview command changes, or provide a
     Make target that materializes the pin before starting `docker compose up`.
   - Keep `make serve` on the pinned generator and add explicit candidate targets (for example
     `make check-generator` and `make serve-candidate`) so generator contributors can test working
     tree changes against rust-edu.org without touching `generator.rev`.
   - **Upgrade sequencing:** a PR cannot place its own future merge commit into `generator.rev`.
     Site changes that rely on a *new* generator feature must land in two steps: merge and tag the
     generator PR first, then bump `generator.rev` in the site PR alongside (or before) the content
     change. Conversely, *tightening* a generator check requires fixing existing site content
     before or with the generator PR so the candidate compatibility check passes.
   - Update [`SECURITY.md`](../../SECURITY.md) so maintainer triage notes that fixing a generator
     issue affecting rust-edu.org requires both landing/tagging the `generator/` fix and bumping
     `generator.rev` to deploy the patched binary.

**Ready to merge when** a change to the working copy of the renderer is visible in a candidate
build while the pinned site still reproduces its previous output. Exercise native and Compose
previews (including from a fresh shallow clone), draft inclusion, offline re-runs, invalid or
missing `generator.rev` diagnostics, and the Pages `BASE_URL` override. To upgrade or roll back,
change `generator.rev` to a reviewed release commit in a site PR and inspect its preview and
generated diffs.

## 3. Route reviews and CI by the files that changed

| PR changes | Checks to run |
|---|---|
| Website content, layouts, assets, site settings, site docs or `generator.rev` | Run `make guards`, then use the pinned CLI to check pages and links, verify fork-`BASE_URL` feed IDs, build twice and compare, and attach the draft `site-preview` artifact. |
| `generator/**` source, manifests or generator docs | Run `make guards`, format, lint and test the candidate workspace in Docker (`--network=none`), then install/build its CLI and run the full website check, fork-`BASE_URL` feed check and double-build diff against the current site as a compatibility check. |
| Both sets of inputs, shared Make, Docker, Compose, toolchain or workflow files, or any unrecognized path | Run both jobs. Include both the old and new paths of renamed files and the paths of deleted files when classifying changes. |

- **Git-only path detection (no third-party actions):** in `ci.yml`, check out the PR merge commit
  (`refs/pull/<n>/merge`) with `fetch-depth: 2` and classify `git diff --name-only HEAD^1 HEAD`
  (which naturally lists deleted paths and both sides of renames). Fall back to running both jobs
  on `workflow_dispatch` or if merge parents are unavailable, and classify **fail-closed** so any
  unrecognized top-level file runs both jobs rather than skipping CI.
- **Single required PR status gate:** add a final aggregator job with `if: always()` depending on
  the classifier, site and generator jobs. Require every dependency to be `success` or `skipped`
  (explicitly failing on `cancelled` as well as `failure`) and verify that at least one check job
  ran, avoiding GitHub's perpetual-Pending state on skipped path-filtered workflows.
- **Keep annotations and local parity:** preserve `ci.yml`'s `sed -uE` `file:line:col: error:` to
  `::error` annotation filter on `make guards` and the site job. Keep `make check` and `make ci` as
  the full local checks, and expose individual Make targets (`make ci-site`, `make ci-generator`)
  matching each CI job.
- **Fail-safe Pages deployment triggers:** in `deploy.yml`, prefer `paths-ignore` on `push` to
  `main` (ignoring `generator/**`, `docs/**`, `README.md`, `AGENTS.md`, `SECURITY.md`, `.claude/**`,
  `.gemini/**`, `.github/workflows/ci.yml`, `.github/pull_request_template.md` and
  `.github/dependabot.yml`) over a narrow `paths` allowlist. That skips automatic deployments on
  generator-only and docs-only merges while guaranteeing that changes to `content/**`, `layouts/**`,
  `static/**`, `site.yaml`, `urls.yaml`, `generator.rev`, `Dockerfile`, `.dockerignore`,
  `Makefile`, `rust-toolchain.toml`, `deploy.yml` or any new website path still publish, with
  `workflow_dispatch` retained for manual deploys.

**Ready to merge when** a content-only change gets a pinned preview, a generator-only change runs
the candidate suite and website compatibility check without triggering a Pages deployment on merge,
a docs-only change runs guards/checks cleanly without hanging required status checks, and a mixed
or shared-tooling change runs both jobs. A broken site link must fail the site job with a PR line
annotation, and a failing parser test must fail the generator job.

## Repository history and a possible future split

Use ordinary move commits on a branch. This preserves existing commit, issue, PR and security
advisory references and gives future generator changes a clean path (`git log -- generator/` after
the move, and `git log --follow -- generator/<file>` across the move). Commit `cd5d5e0` (the squash
merge of #14) did not only add `crates/`: it also updated 11 `content/` files (including ported
community resource PRs #3 and #9), added `layouts/`, `site.yaml`, `urls.yaml`, `static/style.css`,
`static/favicon.svg` and the WebP images (27 website files in all), and removed Zola's templates,
Sass and submodule. Dropping or reverting `cd5d5e0` would lose the site migration itself; stripping
generator history requires path-based tree filtering across `cd5d5e0..main`.

If maintainers later want a dedicated `rust-edu` repository, they can export `generator/` (with or
without filtered `generator/` history) at a reviewed tag, verify `cargo test`, `cargo install` and
library git dependencies there, and switch the website's fetch source and `generator.rev` pin in
one PR (still fetching on the host/runner before Docker's `--network=none` stages and verifying the
pinned `Cargo.lock` guard).

An archive and rewrite of website `main` history is a separate maintainer decision. Before any
rewrite:
1. Preserve the pre-rewrite `main` tip under a named archive branch or tag (not an unreferenced
   SHA, which Git garbage collection can prune) so historical PR and advisory links stay reachable.
2. Rehearse the path-by-path filter in a disposable clone and verify that the resulting tree and
   rendered site (`live`, `--drafts`, and fork `--base-url`) match byte for byte.
3. Freeze merges briefly, coordinate with authors of open PRs and forks so they `git rebase --onto`
   the rewritten `main` instead of merging pre-rewrite commits back in, and re-verify GitHub Pages
   custom-domain (`static/CNAME`) and branch ruleset settings immediately after cutover.
