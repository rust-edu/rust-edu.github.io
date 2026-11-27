# Plan 0006 — Catch disappearing public URLs

Proposed on 2026-11-27. Keep publishing a post as simple as adding one Markdown file. Help reviewers
spot changes that would break a reader's bookmark, including a bookmark nobody on the site links to.

## How publishing works today

`content/news/rust-week-2027.md` publishes at `/news/rust-week-2027/`. A published post also appears
in the news list, sitemap, Atom and RSS. In [urls.yaml](../../urls.yaml), `keep` tells `edu check`
to verify selected page, file or heading addresses against the live build. For example, an old
campus-tour PNG stays accessible to readers even though the post now uses WebP. A `redirects` entry
creates a small HTML page at the old address. The site uses one for `/zulip/`.

Zola also derived URLs from content paths. Authors could override a page's URL with `slug` or `path`,
or put old paths in its `aliases` front matter. This site's Zola news template linked directly to
each post's permalink. Its Zulip redirect was a dedicated content file with a custom template.
[Zola page documentation](https://www.getzola.org/documentation/content/page/) ·
[Former news template](https://github.com/rust-edu/rust-edu.github.io/blob/501c7b6666a156ca05e13731d2e009f54597289f/templates/news.html) ·
[Former Zulip page](https://github.com/rust-edu/rust-edu.github.io/blob/501c7b6666a156ca05e13731d2e009f54597289f/content/zulip.md)

A trial on `0d635ed` added a dated news post in a temporary checkout. It appeared in all five
places above with the original manifest. Renaming the file passed the checks and the old page
disappeared. Adding a redirect restored the old address for browsers. Removing the otherwise
unreferenced `/images/RAACampusTour1.png` failed the check because its address was in `keep`.

## Options

| Approach | What it buys | What it costs |
|---|---|---|
| Keep the curated list and central redirects | Authors already have a one-file publishing workflow. Editors can protect a specific PDF, feed or heading. | An unlisted page can disappear silently once its internal links are gone. |
| Compare live build outputs in PRs | Reviewers see lost page and file paths, including posts added after the Zola migration. | CI needs a second build or a trustworthy output for the exact base revision. |
| Add Zola-style aliases to page headers | When a page moves, its former paths travel with its source. | The generator needs a new front-matter key and checks for conflicts. An alias still needs an author who knows the old path. |

## Recommendation and rollout

1. **Clarify the existing workflow.** The post guide and the manifest header should say that new
   posts need only their Markdown file. Show how to preserve a moved page with a redirect. Prefer a
   title edit to a filename change when the published URL should stay stable. Even with a redirect,
   renaming a post changes its Atom ID and RSS GUID.
2. **Pilot a report if maintainers want wider coverage.** Add a website Make target that compares
   two production output directories. Build the PR base and the proposed merge result from their
   respective revisions at the same base URL, with drafts excluded. Map `index.html` to `/` and
   `a/b/index.html` to `/a/b/`, and retain exact file paths for downloads, feed files and other
   public assets. Exclude build metadata such as `.edu-output` and the Pages `CNAME` configuration.
   Enumerating outputs includes downloads the sitemap omits. Report paths present only in the base.
   If the baseline build is unavailable, say so in the report.
3. **Review the reported removals.** Restore an accidental deletion, add a page redirect for a move,
   retain a file at its old address, or explain an intentional retirement in the PR. Keep the pilot
   advisory and measure the extra build time before considering a required check. Existing `keep`
   and internal-link checks continue to enforce selected anchors and validate redirect targets.

**Acceptance for the pilot.** Publishing one new post needs no manifest edit and reports no lost
path. Renaming an unlisted published post reports the old path. Adding a valid redirect clears it.
Removing a previously shipped PDF reports the file. A title edit or an unpublished draft produces
no removal. Run the same comparison offline from two already-built trees.
