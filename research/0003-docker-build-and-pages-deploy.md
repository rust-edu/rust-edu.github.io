# Research 0003 — Docker Builds and GitHub Pages Deploy
**Date:** 2026-10-28 · **Status:** for review

Build in one image, `rust:1.99.1-slim-trixie` pinned by tag and digest, both locally and in CI:
`docker build --target site -o public .` runs the tests and the link check, then exports only the built site via a `scratch` stage.
Deploy with GitHub's own Pages actions pinned by SHA, after an upstream admin or maintainer flips the Pages source once
(the author works from the fork kenesparta/rust-edu.github.io and has pull-only access upstream). The honest Docker
cost is a ~330 MB pull per CI job, with no layer cache.

## 1. Toolchain and base image
| Item | Verified 2026-10-28 |
|---|---|
| Rust stable | 1.99.1 (2026-10-15), 1.99.0 (2026-10-01) — [Rust blog](https://blog.rust-lang.org/releases/); 1.100.0 due 2026-11-12 ([releases.rs](https://releases.rs/)) |
| Official tags | `1.99.1-slim-trixie` (= `slim`), `1.99.1-trixie` (= `latest`), `…-slim-bookworm`, `…-alpine3.21`–`3.24` (`alpine` = 3.24) — [tags](https://github.com/docker-library/docs/blob/master/rust/README.md) |
| Compressed size amd64/arm64 | slim-trixie 327/287 MB · alpine3.24 352/351 MB · trixie 602/557 MB — [Hub API](https://hub.docker.com/v2/namespaces/library/repositories/rust/tags/1.99.1-slim-trixie) |
| Multi-arch index digest | `rust:1.99.1-slim-trixie@sha256:b8e062b1c3a80183df83d9ba187cadd2b5ab84bc78d453dbfeb2fccd5f982247` |
- Pin tag + index digest in one literal `FROM` line: tags are mutable (this one was re-pushed 2026-10-28 10:44 UTC), and
  Dependabot's `docker` ecosystem bumps tag and digest by PR [pin]. One index digest serves arm64 Macs and amd64 CI.
- musl + `scratch`: `x86_64-unknown-linux-musl` links statically by default [link], but only the built site ships: skip it.
- `rust-toolchain.toml` with `channel = "1.99.1"` keeps host `cargo` equal to the image; rustup auto-installs it [rustup].
## 2. Dockerfile sketch (names as in Plan 0002: binary `edu`, `site.yaml`, `crates/`, `layouts/`)
```dockerfile
FROM rust:1.99.1-slim-trixie@sha256:b8e062b1c3a80183df83d9ba187cadd2b5ab84bc78d453dbfeb2fccd5f982247 AS bin
WORKDIR /src
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates/ crates/
RUN rustup component add clippy rustfmt
RUN --network=none cargo test --locked --offline && cargo build --release --locked --offline \
 && install -m 0755 target/release/edu /usr/local/bin/edu
FROM bin AS build
COPY . .
ARG DRAFTS=false
RUN edu build --out /out --drafts="$DRAFTS"   # non-zero exit on broken internal links
FROM scratch AS site
COPY --from=build /out /
FROM bin AS dev
CMD ["edu", "serve", "--bind", "0.0.0.0:1111", "--out", "/tmp/public"]
```
- `docker build --target site -o public .`: `-o <dir>` = local exporter: it writes on the client, owned by the current
  user, and merges into existing files [bx]; use a fresh dir, or `-o type=local,dest=public,mode=delete` (buildx ≥ 0.35).
- Preview: `docker build --target dev -t site-dev .` + `docker run --rm --init -p 127.0.0.1:1111:1111 -v "$PWD":/site -w /site site-dev`
  (`--init`, or `init: true` in `compose.yaml`: as PID 1 a std-only server ignores Ctrl-C and `docker stop`).
- An allowlist `.dockerignore` (`*`, then `!Cargo.toml !Cargo.lock !rust-toolchain.toml !crates !content !layouts
  !static !site.yaml`) makes `COPY . .` safe: `.git`, `target/`, `public/`, `.idea/` and `ext/` stay out of the context.
- `serve` binds `0.0.0.0` (container loopback is unreachable via `-p`), writes to `/tmp`, not the mount (no root-owned
  files on Linux), and polls mtimes to reload (std-only; works over Docker Desktop mounts).
- Cache: zero crates, so only layer order matters (code before content). BuildKit's `gha` cache needs the containerd store
  or another driver, plus tokens `docker/build-push-action` fills in [gha]: skip it. Estimates: cold CI ≈ 1 min; output ≈ 10 MB.
## 3. GitHub Pages with official actions
| Action (GitHub-owned) | Latest | Pin (full commit SHA, from `git ls-remote`) |
|---|---|---|
| `actions/checkout` | [v7.0.1](https://github.com/actions/checkout/releases/tag/v7.0.1) 2026-07-20 | `3d3c42e5aac5ba805825da76410c181273ba90b1` |
| `actions/configure-pages` | [v6.0.0](https://github.com/actions/configure-pages/releases/tag/v6.0.0) 2026-03-25 | `45bfe0192ca1faeb007ade9deae92b16b8254a0d` |
| `actions/upload-pages-artifact` | [v5.0.0](https://github.com/actions/upload-pages-artifact/releases/tag/v5.0.0) 2026-04-10 | `fc324d3547104276b827a68afc52ff2a11cc49c9` |
| `actions/deploy-pages` | [v5.0.1](https://github.com/actions/deploy-pages/releases/tag/v5.0.1) 2026-09-01 | `368f82528645a54fb793d4d04e342629a3f51346` |
| `actions/upload-artifact` (PRs) | [v7.0.2](https://github.com/actions/upload-artifact/releases/tag/v7.0.2) 2026-10-07 | `cf430e030ddbb5b0abf93d22962f4752f3646cd9` |
- Write `uses: actions/checkout@<sha> # v7.0.1`; Dependabot bumps the SHA and the same-line comment [sec], and admins can
  enforce SHA pins [aset]. The starter lags (`checkout@v4`) [start]; upload-pages-artifact SHA-pins upload-artifact [upa].
- Shape: `contents: read`; `pages: write` + `id-token: write` on the deploy job only ([start] grants all three workflow-wide);
  it targets `environment: github-pages` (exists upstream, no rules; see §8 [src]); concurrency `pages`, no cancelling.
- Gotchas: the Pages artifact keeps `retention-days: 1` and drops dotfiles by default [upa]. `configure-pages`
  `enablement` only creates a missing site (`src/api-client.js`), never flips a `legacy` one, and needs a non-`GITHUB_TOKEN` token [cfg].

| | A: Source = GitHub Actions (official flow) | B: `git push` built site to `gh-pages` in a `run:` step |
|---|---|---|
| Settings | One-time flip by admin/maintainer ([docs](https://docs.github.com/en/pages/getting-started-with-github-pages/configuring-a-publishing-source-for-your-github-pages-site)) | None: upstream is `legacy`, `gh-pages /` today |
| Token | `pages: write`, `id-token: write` | `contents: write`; upstream `main` has no protection or rulesets |
| Domain | `CNAME` file ignored; the Pages setting already holds `rust-edu.org` | `CNAME` must survive every force push ([docs](https://docs.github.com/en/pages/configuring-a-custom-domain-for-your-github-pages-site/troubleshooting-custom-domains-and-github-pages)); add `.nojekyll` |
| Limits | Exempt from the 10 builds/h soft limit ([limits](https://docs.github.com/en/pages/getting-started-with-github-pages/github-pages-limits)) | Soft limit applies, plus a second "pages build and deployment" run |
| Rollback | Re-run all jobs of an older run: same SHA, ≤ 30 days ([re-run](https://docs.github.com/en/actions/how-tos/manage-workflow-runs/re-run-workflows-and-jobs)) | Revert on `main`; resetting `gh-pages` needs kept history |

**Recommendation: A for upstream.** GitHub recommends a workflow for non-Jekyll builds [src]. The deploy job never holds
`contents: write` on an unprotected repo; no generated branch, CNAME rule or Jekyll quirk to keep. Cost: ~2 admin minutes.
B is the no-admin fallback and is proven today: a `GITHUB_TOKEN` push of `d63ba44` triggered [run 25219977919][run].
## 4. Custom domain
- Verbatim: "If you are publishing from a custom GitHub Actions workflow, no `CNAME` file is created, and any existing
  `CNAME` file is ignored and is not required." Only an admin sets the domain, in Settings or via the API [cname], [src].
- Upstream now (`gh api repos/rust-edu/rust-edu.github.io/pages`): `build_type: legacy`, `cname: rust-edu.org`,
  `https_enforced: true`. Re-check both after the flip; keep `static/CNAME:1`, which B and the flip-back path need.
- `config.toml:2` still says `base_url = "https://rust-edu.github.io"`; the new config should say `https://rust-edu.org`.
## 5. PR validation (fork-safe)
- `on: pull_request` (never `pull_request_target`), `permissions: contents: read`, no secrets; fork PRs get a read-only
  token and no secrets anyway [ev]. Steps: checkout (`persist-credentials: false`) → `docker build --progress=plain
  --target site --build-arg DRAFTS=true -o public .` → `upload-artifact` (`site-preview`, 7 days); cf. `main.yml:20-24`.
- First-time contributors' runs wait for a maintainer with write access to approve them (the default) [ev], [aset]. No
  ruleset exists upstream, so a red check blocks nothing until a rule requires it.
## 6. Docker in CI vs cargo on the runner
| | Docker build (chosen) | `cargo` on the runner |
|---|---|---|
| Toolchain | Pinned image, identical to laptops | Rust 1.99.0 + rustup 1.29.1 preinstalled on [24.04](https://github.com/actions/runner-images/blob/main/images/ubuntu/Ubuntu2404-Readme.md) and [26.04](https://github.com/actions/runner-images/blob/main/images/ubuntu/Ubuntu2604-Readme.md); auto-installs 1.99.1 |
| Overhead per job (estimate) | ~330 MB pull ≈ 10–30 s; no layer cache | ≈ 10–20 s toolchain install; `actions/cache` usable |
| Docker Hub limits | [Not applied](https://docs.github.com/en/actions/reference/limits) to hosted runners pulling public images | n/a |
| Local requirement | Docker; Desktop is free for personal, education, OSS and small-org use; paid for orgs ≥ 250 staff or ≥ $10M revenue and government ([license](https://docs.docker.com/subscription/desktop-license/)) | rustup |
| Runner drift | `ubuntu-latest` is moving 24.04 → 26.04 now (rollout 2026-10-19 to 11-19, [#14748](https://github.com/actions/runner-images/issues/14748); Docker 28.0.4 → 29.4.2); the image hides the OS change | Toolchain pinned; the OS is not |
- Honest cost: ≈ +15–30 s per run (estimate) and Docker for local previewers; Markdown-only editors need nothing.
## 7. "Edit with only Markdown" flow
1. github.com: open the `.md` → pencil → "Fork this repository" (non-writers) → edit → Preview → "Propose changes"
   → PR [edit]. Or github.dev (`.` key) for multi-file edits. It has no terminal or compute, so CI is the build [dev].
2. PR check + `site-preview` artifact → maintainer merges → the deploy workflow publishes.
- github.com renders YAML front matter as a key/value table [yaml] (re-checked today on a github/docs file); TOML `+++`
  (`content/news/announcement.md:1-7`) renders as raw paragraph text, so YAML-style metadata previews cleanly.
- Local preview: `docker compose up` (optional `compose.yaml`) or the §2 commands; with Rust: `cargo run -p edu-site -- serve`.
## 8. Cutover and rollback
Facts: `gh-pages` is one parentless commit (`d63ba44`, 2026-05-01). shalzz does `set -e` → `git init` → `git push --force`
[shz], so there is no history to revert to, and a failed build never pushes: the old site stays up.
1. PR: add `ci.yml` (Docker PR check); `main.yml` keeps deploying.
2. PR: add `deploy.yml` (flow A), `workflow_dispatch` only: inert until the flip (a pre-flip run from `main` should be refused [dpl]).
3. Admin: flip Source, then ensure a default-branch-only rule; run `deploy.yml`; check domain, HTTPS, key URLs. Until 4
   lands, merges publish only by dispatch. Rollback: relax the rule (it overrides the source branch [dpl]); flip to `gh-pages /`.
4. PR: add `push: main`, delete `main.yml` (`main.yml:21,36`), fix its `README.md:2` badge; land it no later than the PR that stops Zola.
5. After ~2 weeks of good deploys, delete `gh-pages` (optionally tag `d63ba44` first).
## Implications for the plan
- Small PRs: Dockerfile + `.dockerignore` → `ci.yml` → `deploy.yml` (dispatch-only) → push trigger + delete `main.yml`
  → `.github/dependabot.yml` (`docker`, `github-actions`). Admin steps go into PR descriptions as checklists.
- Generator CLI contract: `build --out --drafts --base-url`, `serve --bind --out` (polling); non-zero exit on broken
  internal links. Keep `404.html`, `sitemap.xml`, `robots.txt` and today's URL paths (see the `gh-pages` root).
- Rehearse A on the fork first: a project site under `/rust-edu.github.io/`, so feed `configure-pages`' `base_url` output to `--base-url`.
## Open questions

Answered in Plan 0002; step 4's PR picks the runner image (§6).

[pin]: https://docs.docker.com/build/building/best-practices/#pin-base-image-versions
[link]: https://doc.rust-lang.org/reference/linkage.html#static-and-dynamic-c-runtimes
[rustup]: https://blog.rust-lang.org/2025/03/04/Rustup-1.28.1/
[bx]: https://docs.docker.com/reference/cli/docker/buildx/build/
[gha]: https://docs.docker.com/build/cache/backends/gha/
[sec]: https://docs.github.com/en/actions/reference/security/secure-use
[start]: https://github.com/actions/starter-workflows/blob/main/pages/static.yml
[upa]: https://github.com/actions/upload-pages-artifact/blob/v5.0.0/action.yml
[src]: https://docs.github.com/en/pages/getting-started-with-github-pages/configuring-a-publishing-source-for-your-github-pages-site
[cfg]: https://github.com/actions/configure-pages/blob/v6.0.0/action.yml
[aset]: https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/enabling-features-for-your-repository/managing-github-actions-settings-for-a-repository
[run]: https://github.com/rust-edu/rust-edu.github.io/actions/runs/25219977919
[cname]: https://docs.github.com/en/pages/configuring-a-custom-domain-for-your-github-pages-site/managing-a-custom-domain-for-your-github-pages-site
[ev]: https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows#pull_request
[edit]: https://docs.github.com/en/repositories/working-with-files/managing-files/editing-files
[dev]: https://docs.github.com/en/codespaces/the-githubdev-web-based-editor
[yaml]: https://github.blog/news-insights/product-news/viewing-yaml-metadata-in-your-documents/
[shz]: https://github.com/shalzz/zola-deploy-action/blob/master/entrypoint.sh
[dpl]: https://github.com/actions/deploy-pages/blob/v5.0.1/README.md
