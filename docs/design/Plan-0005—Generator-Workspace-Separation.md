# Plan 0005 — Separate the generator from the website content

Proposed on 2026-11-26 for [issue #17](https://github.com/rust-edu/rust-edu.github.io/issues/17).

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

- Move the Cargo workspace manifest, lockfile, four crates and their tests to `generator/`.
  Give the workspace a README covering installation, the library entry point and the site directory
  that `edu check`, `edu build` and `edu serve` operate on.
- Move the generator guide, design plans 0002 and 0004, and the six research notes alongside the
  crates. Fix links and contributor instructions in the same PR. The site owns `content/`,
  `layouts/`, `static/`, `site.yaml`, `urls.yaml`, the editing guides and visual plan 0003.
- Update the root Makefile, Docker stages and Docker context allowlist in the move. Keep the shared
  Rust toolchain pin at the root, where native builds and the existing Dependabot configuration use it.
  Keep the four-package lockfile guard, digest-pinned image and `FROM check AS live` gate.
- The feed test in `crates/site/src/build.rs` currently walks two directories up to find the
  production site. Exercise its canonical-ID assertions on the existing feed fixture. Check the
  real site's Atom IDs and RSS GUIDs in the site build at a fork base URL.

**Ready to merge when** `make check` and `make ci` pass, the generator suite runs with only the
workspace and toolchain available, and the installed CLI checks and builds a copy of the valid
fixture outside the repository. Run the old and new generators on the same site inputs and compare
the output trees byte for byte for a live build, a draft preview and a fork served under a path.
An isolated trial on `b822727` passed 160 tests and all three output comparisons. Repeat those
comparisons against the merge base of the implementation PR.

## 2. Release the generator and pin the website to it

After the move lands, tag its merged commit with a generator-specific version such as
`generator-v0.1.0` matching the workspace version. Document installation from the tagged repository
and use from another Cargo project. Test both a fresh CLI install and a separate Rust package that
depends on one library. Git releases work with the workspace's current `publish = false` setting.

Record the full commit ID in a single website file, `generator.rev`. For a site build, read that
pin, obtain the corresponding `generator/` sources from this repository, and pass them to Docker
as a local build context. A fresh checkout may need to fetch the pinned commit. Once the sources
and toolchain are present, compile with `--locked --offline` and run the content, link and
reproducibility checks with networking disabled. Local Make targets, Compose, PR previews and
Pages builds must all resolve the same pin. A missing revision must fail with a diagnostic.

**Ready to merge when** a change to the working copy of the renderer is visible in a candidate
build while the pinned site still reproduces its previous output. Exercise native and Compose
previews, draft inclusion, and the Pages `BASE_URL` override. To upgrade or roll back, change the
pin to a reviewed release commit in a site PR and inspect its preview and generated diffs.

## 3. Route reviews and CI by the files that changed

| PR changes | Checks to run |
|---|---|
| Website content, layouts, assets, site settings or release pin | Use the pinned CLI to check pages and links, build twice and compare, and attach the draft preview. |
| Generator source or manifests | Format, lint and test the candidate workspace. Install its CLI and build the real website with it as a compatibility check. |
| Both sets of inputs, or shared Make, Docker, toolchain or workflow files | Run both sets of checks. Include the path that held a renamed or deleted file when deciding which jobs to run. |

Keep `make check` and `make ci` as the full local checks. Provide Make targets for each CI job and
a required PR status that reports the result of every applicable job. Trigger automatic Pages
deployment for website inputs, the release pin and relevant build tooling. Keep manual deployment.

**Ready to merge when** a content-only change gets a pinned preview, a generator-only change runs
the candidate suite without triggering a Pages deployment, and a mixed change runs both jobs.
A broken site link must fail the site job and a failing parser test must fail the generator job.

## Repository history and a possible future split

Use ordinary move commits on a branch. This preserves existing commit and PR references and gives
reviewers a path for future generator changes (`git log -- generator/`). The commit that introduced
the generator, `cd5d5e0`, also touched 11 content files. Dropping that commit would lose site edits.

If maintainers later want a dedicated `rust-edu` repository, they can export the workspace at a
reviewed tag, verify the CLI and libraries there, and switch the site's source and pin in one PR.
An archive and rewrite of website history is a separate maintainer decision. Before any rewrite,
preserve the old `main` tip under an archive ref, rehearse the path-by-path split in a disposable
clone, compare both trees and site output, and agree on a rollback and contributor cutover.
