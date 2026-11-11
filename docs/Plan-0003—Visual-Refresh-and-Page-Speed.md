# Plan 0003 — Visual Refresh and Page Speed

**Date:** 2026-11-09 · **Status:** proposed · **Follows:** [Plan 0002](Plan-0002—Own-Markdown-Site-Generator.md)

**Goal:** a simpler, modern rust-edu.org that loads fast on slow phones and networks: one column of text, a home page
that says what Rust Edu is and what to do next, no added effects, and about a third fewer bytes on a first visit to
each text page. Part A's steps change only CSS, layouts, `site.yaml`, images and their links in one post, the docs and
`ci.yml`, so they add no Rust to the generator.

## Where the site stands

Plan 0002 already did most of the work, before its generator took over: its hand-written stylesheet replaced 429 KB
of CSS, a runtime import of web fonts and a script while Zola still built the site. Live, on a slow 4G phone profile,
the home page of rust-edu.org (still the old build) needs 103 KB in 8 requests and first paints at 1.4 s; that of the
fork where Plan 0002 was built, kenesparta/rust-edu.github.io, needs 19.5 KB in 4 requests and paints at 0.48 s
(medians of 3 runs). 5.4 KB of the fork's figure is GitHub's 404 page for `/favicon.ico`, which only the fork's
address gets, so on rust-edu.org the same build weighs about 14.4 KB. What remains, measured on that build (method
at the end):

| | Today | After steps 1–4 (no Rust) |
|---|---|---|
| Text page, first visit | 13.8–16.1 KB, 4 requests | 8.7–11.1 KB, 4 requests |
| Campus tour post, first visit | 1,738 KB, load 8.6 s | 280 KB, load 1.65 s |
| Campus tour, 400 ms round trip and 400 kbit/s | load 34.8 s | load 6.3 s |
| Layout shift (CLS), campus tour, 1440×1400 window | 0.0754 | 0 |
| First paint (FCP) | 396–464 ms | 412–464 ms: no better (−4 to +16 ms a page, cause not found) |

- **Photos:** the four campus tour photos are 1.7 MB of PNG at 1280×720, shown at most 688 px wide (640 after
  step 4).
- **Logo:** at 24 KB (9.3 KB with gzip), the largest file on every page without photos. Over half of it is path data
  with long coordinates inside nested transforms; editor metadata, unused definitions and inline styles make up most
  of the rest.
- **Favicon:** none is declared, so the first page of a visit also asks for `/favicon.ico` and gets the site's
  68-byte 404 page (at the fork's address, GitHub's 5.4 KB one).
- **Layout shift:** photos have no reserved space, so the text below them jumps when they arrive.
- **Look:** the home page leads with "Rust Edu" and a grey subtitle, the footer repeats the menu, lines average about
  85 characters (a tenth reach 90 or more), and no link reaches `/workshop/`.

## Principles

| Topic | Rule |
|---|---|
| Effects | None: no shadows, gradients, blur, transitions or animation; hairlines only under the header, above the footer, between news rows and beside quotes (warm adds a 3 px top rule and a tinted footer band) |
| JavaScript | None (Plan 0002, decision 3) |
| Fonts | The system's own; no web fonts |
| Colors | Today's values as plain hex (warm adds its paper and the crab's orange), in one light and one dark token block, dark mode by media query; link green `#0b7261` (Plan 0002, decision 2); the brand red and yellow stay only as focus colors, and inline code takes the text color; the spacing and type scales become plain `rem` values |
| Accessibility | WCAG 2.1 AA: 4.5:1 for text and 3:1 for focus outlines in both themes; skip link; reflow at 320 px and at 200% text |
| Older browsers | Only CSS whose absence costs polish or brings back today's behavior (without `aspect-ratio`, photos move the text as they do today): no `light-dark()`, nesting, `@layer`, `oklch()`, `color-mix()`, `:is()` or `:where()`; `:has()` only for spacing; no flex `gap` |
| Generator | Most of Part A works with any generator: steps 1, 3 and 5 change only images, one post, plain CSS and CI, and step 4's stylesheet is plain CSS too; only the layouts and `site.yaml` of steps 2 and 4 are read by this generator alone. Part B is Rust to keep for as long as the generator is kept |
| Zulip | The menu and the home page link to `/zulip/` (Plan 0002) |

## The look

The design below is called **quiet**; **warm** is an alternative to it (decision 1).

- **One column.** Header, text and footer share one 40rem column with one left edge at every width, and the footer
  sits at the bottom of short pages. The page always shows its scrollbar, so the column does not move between short
  and long pages.
- **The home page leads with its purpose.** Its `h1`, the statement, is the site's description, "Rust education in
  academia (and beyond)", set large (2.5rem on phones, 3rem from 40rem, weight 600). Under it, one row of next
  steps: a green *Join us on Zulip* button and a plain *Find resources* link, from a new `hero` list in `site.yaml`
  (decision 7). The Markdown sections follow, set apart by space alone.
- **Menu:** *News*, *Resources* and *Zulip chat*, in the text color with an underline on hover. *Home* goes, since
  the logo links home.
- **Footer:** a hairline and two short rows: "Content licensed CC0. Source on GitHub." and the `footer` links, which
  gain *2022 Workshop*.
- **News:** flat rows, with the date in muted tabular figures in a narrow column left of the title from 40rem up, and
  above it on phones. The byline keeps its sentence, "A and B on Mar 1, 2027".
- **Type:** system fonts with "Segoe UI" named first; body text 17 px on phones and 18 px from 40rem, line height
  1.6, about 73 characters a line; headings in weight 600, and the menu, news titles and next steps in 500.
- **Images:** each image in a page fills the column, with small corners, and holds a 16:9 space while it loads. A
  bold line right above a photo sits close to it, as its label.
- **Head:** a `color-scheme` meta tag, an SVG favicon (527 B, a crab in a cap, drawn new for this plan), and the home
  page's title "Rust Edu: Rust education in academia (and beyond)" instead of "Home - Rust Edu".

`style.css` becomes 6,547 B (today 7,632 B), but 2,327 B with gzip against today's 2,178 B: step 4 is for the look,
and the bytes saved on each page come from the logo (step 3). A few values are tuned to one case, which the file's
short comments do not always name: the cap on `h1` (decision 5); the logo's `margin-left: -11px`, the drawing's
blank margin at the 150 px width set in `layout.html`; the 46rem query, from which the gutter outside the column
holds a list marker; and "Segoe UI" before `system-ui`, which keeps Segoe UI for Latin text on Windows in a Chinese
or Japanese locale.

**Warm** adds identity by flat means: warm paper (`#fbf8f3`, in dark `#1b1815`), a 3 px top rule in the crab's
orange, a tinted footer band across the window, the CC0 crab beside the statement on the home page, and 8 px
corners. On top of quiet it is a token block, one marked block of CSS that moves the column from `body` to its
children, and one line of `home.html`: 64 lines and a one-line `static/crab.svg`. It costs one more request (1.9 KB)
on the home page and about 0.2 KB more stylesheet on the first page of a visit.

## Steps

One branch and one PR per step, named as in Plan 0002, near 500 changed lines (600 at most), each passing
`make check`. No test or check shows how a page looks (see *Risks*), so each PR that changes the look shows before
and after screenshots of `/`, `/news/` and `/news/campus-tour/` at 390 and 1280 px, in light and dark. Each PR says
which AI tools helped and how, as a maintainer asked on the upstream PR, rust-edu/rust-edu.github.io#14. Steps 1, 2
and 3 do not depend on each other; step 4 comes after steps 1 and 2, whose lines it rewrites, and its budget needs
step 3; step 5 comes after step 1, whose photo note it extends.

**Part A — no Rust**

0. `docs/visual-refresh-plan` — this plan. Its PR shows the screenshots named above, of today, quiet and warm, from
   which decisions 1, 5 and 7 are taken. *Done when:* merged, with decisions 1–5 and 7 answered and recorded as in
   Plan 0002 ("*Decided …*"); decision 6 waits for the maintainers' answer on the generator.
1. `perf/photos` (about 17 lines and four WebP files, 267 KB in all) — the campus tour photos at 960×540 as WebP
   (`cwebp -m 6 -sharp_yuv -q 75 -f 30 -metadata none -resize 960 540`), linked from
   `content/news/campus-tour.md`; in today's `style.css`, `main img { width: 100%; aspect-ratio: auto 16 / 9; }`,
   so that each photo holds its space while it loads; and under *Common changes* in `docs/editing.md`: "**A
   photo:** it shows at the full width of the text, so save it 960 pixels wide, in the 16:9 shape, as WebP of at
   most 90 KB. A narrower image is stretched, and another shape moves the text below it when it loads, a tall photo
   by most of a phone's screen." The PNGs stay at their addresses (decision 4). *Done when:* the four WebP files are
   960×540 and none is over 90 KB (`file` and `ls -l`), so the post's first visit is about 285 KB on rust-edu.org
   (290 KB at the fork's address, 280 KB once step 3 lands); the post shows no layout shift in a 1440×1400 window
   (today 0.0754); and the photos look the same as the PNGs at 1×, and a little softer at 2× (960 px files shown
   688 px wide).
2. `fix/head-and-links` (14 lines) — the `<head>` in the order charset, viewport, `color-scheme`, title,
   description, icon, stylesheet; the home page's title; `static/favicon.svg`; the menu *News*, *Resources*, *Zulip
   chat*, and *2022 Workshop* in the footer links (decision 3). On rust-edu.org the icon costs about 0.3 KB more
   than today's 404, and gives the tab an icon. *Done when:* in the PR's `site-preview` (the built site that CI
   attaches to each PR), `grep -rL 'rel="icon"' --include='*.html' .` lists only `404.html` and `zulip/index.html`,
   which the generator writes itself and which still ask for `/favicon.ico` (Safari before 26 shows no SVG icon);
   and the home page's title reads "Rust Edu: Rust education in academia (and beyond)".
3. `perf/clean-logo` (one file, +1 −331) — the same drawing at the same address, without editor metadata, unused
   definitions, inline styles and nested transforms, its coordinates rounded to two decimals: 24,295 B → 8,544 B,
   and 9.3 KB → 3.9 KB with gzip. The `<desc>` URL is kept word for word, now as the drawing's description beside a
   new `<title>`. *Done when:* screenshots of the logo in the header of `/`, before and after, in a 1280 px window
   at 1× and 2×, in light and dark, differ in no pixel at 1× and in at most 10 edge pixels at 2×, counted with a
   pixel diff such as ImageMagick's `compare -metric AE` (measured: 0, and 3 in light and 4 in dark).
4. `feat/simple-modern` (438 lines, and about 8 lines of docs not yet written) — the look above:
   `static/style.css` (+185 −206), the footer of `layouts/layout.html`, the statement and next steps in
   `layouts/home.html`, the date first in `layouts/news.html`, the `hero` list in `site.yaml`. `README.md`,
   `AGENTS.md` and `docs/editing.md` name the `hero` list, and `docs/editing.md` says that the home page shows the
   site's description as its heading. *Done when:* the page budget holds; in both token blocks, text, muted, link
   and hover colors reach 4.5:1 on the page and on the code background (`--color-surface`), the button's text 4.5:1
   on its fill, and the focus color 3:1 on the page; `/`, `/news/`, `/news/campus-tour/` and `/resources/` do not
   scroll sideways in a 320 px window, nor at 200% text (browser font size 32 px) at 320, 390 and 1280 px, in light
   and dark. If decision 1 takes warm, a later PR `feat/warm-look` (65 lines) adds it; *Done when:* the page budget
   holds, with 5 requests on the home page, and warm's token blocks meet the same contrast.
5. `ci/image-size-limit` (11 lines: 8 in `ci.yml`, and 3 that add to step 1's photo note the 500 KB stop and a way
   to shrink a photo; optional) — a step that CI runs (`make check` does not) and that fails when an `.avif`,
   `.gif`, `.jpeg`, `.jpg`, `.png` or `.webp` file in `content/` or `static/` is over 500 KB, so a photo straight
   from a phone cannot land by mistake. It does not check PDFs (the kept `/workshop/proceedings.pdf` is 7.9 MB) or
   SVG images, and it does not hold the budget's 90 KB. *Done when:* a PR that adds a 3 MB JPEG fails CI with the
   file's name.

**Part B — optional Rust, once the maintainers decide to keep the generator, and after the Atom feed**

Each step below is independent of the others in its code and comes after step 4; its size comes from a patch that
was built and passes `make check` on top of step 4. Each will need a rebase onto the Atom feed, which is likely to
change the same lines of `build.rs` and the `<head>`. Together they add about 100 lines of Rust and 120 of tests.

6. `feat/lazy-images` (35 lines: 6 Rust, 27 of tests and golden HTML, 2 of docs) — `loading="lazy"` on each image
   from Markdown. Before scrolling, the campus tour's first visit falls from 280 KB to 87 KB, and its load event from
   1.65 s to 0.43 s; a reader who scrolls to the end still fetches all 280 KB, and first paint stays near 450 ms.
   It changes a golden file (`crates/markdown/tests/golden/`, the expected HTML of each page) and ends the README's
   claim of output byte for byte equal to Zola's. It needs step 1's photo rule. *Done when:* before scrolling, the
   campus tour's first visit in a 412 px phone view is under 100 KB from a server that gzips, and in a 1440×1400
   window outside device mode the layout does not shift.
7. `fix/404-page` (20 lines: 9 Rust, 11 of tests) — today's 68-byte `404.html` becomes a 280-byte page with
   `lang`, viewport, title, the stylesheet and a link home. It has no icon link (about 2 more lines), so it still
   asks for `/favicon.ico`. Its markup and text stay a constant in `crates/site/src/build.rs`, as today, so changing
   them needs Rust; a page from `content/404.md` and the layouts is the alternative (61 lines, 25 of them Rust).
   *Done when:* a missing address answers with a page that loads `/style.css` and links home.
8. `feat/menu-current-page` (91 lines, and about 10 more to make its two `aria-current` values, `page` and `true`,
   an enum, as AGENTS.md asks) — `aria-current` on the menu item of the page's section, shown in the link color and
   underlined. It puts the shape of `site.yaml`'s menu into Rust. *Done when:* on each page of a menu section, that
   section's menu item carries `aria-current`.
9. `perf/inline-css` (76 lines) — the stylesheet inside each page's `<style>`, which `edu check` refuses if it
   holds a `<`. The only step that moves first paint: −164 to −220 ms on slow 4G (against step 6), and at a 400 ms
   round trip, on the home page, −456 ms on a first visit and −428 ms on a repeat visit (against step 4). There,
   with a real connection's setup added, a text page first paints at about 2.2 s without it and 1.7 s with it. Its
   price: about 2.3 KB more on every later page of a visit (2.4 KB with warm); the HTML of the campus tour and of
   `/workshop/cfp-2022/` at 7.0 and 5.4 KB with gzip, past the budget's guide of 5 KB; and a shift of the footer on
   `/news/` (CLS 0.008 in 11 of 21 runs; other short pages not measured), over the budget's 0. So it comes last,
   and only if wanted. *Done when:* no page built from the layouts links `style.css`, which stays at its address
   for `urls.txt` and step 7's 404 page; first paint falls by at least 150 ms on the budget's profile; and `/news/`
   shows no layout shift, or the same PR changes the budget's CLS line.

## Page budget

| Item | Limit | After step 4 |
|---|---|---|
| Each photo a page shows | ≤ 90 KB, ≤ 960 px wide, WebP | 48–87 KB at 960×540 |
| Logo | ≤ 4 KB with gzip | 3.9 KB |
| `style.css` | ≤ 7,632 B (today's size), and ≤ 2.5 KB with gzip | 6,547 B and 2,327 B |
| HTML of one page (a guide: a long post may pass it) | ≤ 5 KB with gzip | 0.9–4.8 KB |
| First visit to a page without photos | ≤ 15 KB, ≤ 4 requests (5 on the home page with warm) | 8.7–11.1 KB, 4 requests |
| First visit to the photo page, before scrolling | ≤ 15 KB + 90 KB per photo that loads | 280 KB: all four photos load until step 6 (87 KB after) |
| First paint (FCP), slow 4G profile | for steps 1–4, no more than 20 ms later than today's build; after step 4, than the build before the PR | 412–464 ms (today 396–464 ms) |
| First paint (FCP), 400 ms round trip and 400 kbit/s, home and campus tour | ≤ 1.2 s | 0.94–1.16 s (today 0.93–1.14 s) |
| Layout shift (CLS), counting every shift | 0 with 16:9 photos | 0 |

KB is 1,000 B. The site meets every line once step 4 has landed; before then the logo is over its line until step
3, and `style.css` is over its own from step 1 (7,769 B with the photo rule) until step 4. Later PRs are held to the
lines in review.

Sizes with gzip come from `gzip -9 -n -c <file> | wc -c`; GitHub Pages' own gzip is up to 1.8% larger, which would
serve warm's stylesheet at about 2,530 B. A first visit is the sum of the gzip sizes of the page, `style.css`, the
logo and the icon, plus about 0.3 KB of headers a request. First paint needs Chrome, a server that gzips as GitHub
Pages does, and the throttling profile of *How the numbers were taken*, set by hand (DevTools' "Slow 4G" preset adds
562.5 ms of latency and 10% less throughput instead); `make serve` sends no gzip. Layout shift needs the same
profile, so that photos arrive after the first paint, and is measured outside device mode, which leaves early shifts
out. Each is the median of 7 cold loads, with the two builds measured in one session and their runs interleaved;
medians of the same page in two builds differed by up to 8 ms. The repository has no such server, so each PR's
author takes these lines and gives the medians in the PR.

## Decisions for review (default first)

1. **Look** (before step 4): **quiet**, as described; or quiet with **warm** as a later PR. Warm has more
   character, and its band closes short pages and wide windows, where quiet leaves one column on white. But its
   `style.css` sits 15 B under the 2.5 KB limit by `gzip -9` and about 30 B over it as GitHub Pages serves it
   (quiet: 173 B under); with step 8 it reaches 2,501 B. Its home page takes 5 requests; in dark mode its logo's
   crab is paler than the crab beside its statement; and its statement breaks inside words from 175% text on phones
   and at 200% on windows up to 430 px wide (quiet: at 200%, up to 390).
2. **Where the steps merge** (before step 0): **into the fork's `main`, as in Plan 0002**. The upstream PR (#14)
   comes from that branch, so each merge joins it and redeploys the fork's site; #14 grows while it is under review,
   as it did with fork PR #33 (the instructions for AI coding agents and the `Makefile`, merged after #14 opened). If
   the maintainers move the fork into place instead of merging #14, the steps are already there. Or an integration
   branch `next` on the fork, which keeps #14 as it stands: each step is a PR into it; `main` comes in by a PR from
   `main` into `next`, since CI checks pull requests into any branch but pushes only to `main`; nothing deploys from
   it; and it goes upstream after the Atom feed, once #14 has landed (rebased first if #14 is squashed) or the fork
   has moved.
3. **Navigation** (before step 2): **drop *Home* from the menu, rename *Zulip* to *Zulip chat* and move it last, and
   link *2022 Workshop* from the footer**. Each part can be declined alone.
4. **The old PNG addresses** (before step 1): **keep the four PNGs** (`urls.txt` lines 8–11; 1.7 MB in each build,
   and in git history either way); or delete them with their lines, on purpose.
5. **Width of page titles** (before step 4): **`max-width: 12.1em` on `h1`** (387 px at 2rem, and 581 px for the
   statement), so that Chrome's balanced wrapping (`text-wrap: balance`, which step 4 gives headings) cannot break
   "Rust-Edu" at its hyphen in the newest post's title, "Announcing the 2026 Rust-Edu Refresh and CFP". Its price:
   titles wrap sooner ("Rust Education Workshop 2022" takes two lines at 1280 px; 55 characters take three where two
   fit), and it guards that one title. Or no cap, and a non-breaking hyphen (U+2011) typed in that title: the same
   two lines from 430 px up in Chrome, in the system font, Segoe UI and Roboto, at the price of a content edit that
   find-in-page for "Rust-Edu" may miss. Or neither: Chrome then breaks that title as "Rust- / Edu" in every window
   from about 588 px wide (476 px in Roboto).
6. **Rust** (before Part B): **none until the maintainers decide to keep the generator, and the Atom feed first;
   then steps 6 and 7**, and steps 8 and 9 only if the maintainers want them (step 9 is the one that serves slow
   networks).
7. **Home page** (before step 4): **the description as its heading, and a row of next steps from a `hero` list in
   `site.yaml`**, the first drawn as a button (a page's header cannot hold the links, since `edu check` refuses keys
   it does not know); or keep "Rust Edu" with the description under it, as today, and write the two links in
   `content/_index.md`.

## Not in this plan

- The Atom feed (`/atom.xml`), which answers upstream issue #6, "Provide a way to follow progress"; the site has no
  feed today. It needs Rust and gets its own plan; as the upstream PR says, it is the first follow-up once the
  generator is upstream.
- Image sizes written by the generator (`width` and `height` on each `<img>`): 435 lines, and as written its test
  fails beside step 6's. Not needed while photos are 16:9.
- A separate dark logo through `<picture>`: the generator neither checks nor rebases `srcset` (`links.rs` reads only
  `href` and `src`; a 15-line fix), and printing would show its light lettering on white paper. The CSS filter that
  already turns the logo light in dark mode does the job.
- The newest post on the home page, a description for each page, a 404 page written in Markdown (step 7 names it).
- Speculation rules, view transitions, AVIF, `srcset`, minified CSS or HTML (on step 4's files, about 0.8 KB once
  for the stylesheet and 0.06 KB a page), `theme-color`, web fonts, a CDN.
- Where `/zulip/` leads (Zulip's log-in page, which also offers *Sign up*).

## Risks

- **Keeping the generator.** The maintainers may not keep it; the *Generator* principle says what holds without it,
  and Part B waits for their answer.
- **No test sees the look.** `make check` parses the layouts and follows their links, but passes with a page that
  renders wrong or a broken stylesheet. Screenshots in each PR, and a look at its `site-preview`, stand in for tests.
- **Images of other shapes.** The photo rule holds a 16:9 box. A portrait photo still moves the text below it when
  it arrives, by most of a phone's screen (CLS 0.11–0.16), a small image is stretched to the column, and `edu check`
  warns of neither. `max-height: 90vh; object-fit: contain; object-position: left` on `main img` keeps a tall photo
  inside one window for about 28 B of gzip; it is the first rule to add when such a photo appears.
- **Fonts.** Weight 500 falls back to regular where a system font has no such face (likely Segoe UI). Segoe UI and
  Roboto were checked only as web fonts on a Mac.
- **The logo's sources.** The crab is the CC0 Rustacean. The file names no license or author for the rest, and its
  only source note, a `<desc>` URL to a "desktop keyboard" clip-art page that now returns 404, may or may not
  concern the cap. Ask the author of commit 7bfbba4 before using the cap anywhere new; warm's crab has no cap, and
  the favicon's cap is drawn new.
- **Taste.** The maintainers may prefer today's look. Steps 1, 2 and 3 stand alone and need no decision on it.

## Known limits

- With classic scrollbars, as on Windows, short pages show an empty scrollbar track: the price of a column that
  does not move between pages.
- Below 46rem a three-digit list marker can lose part of its first digit: "1xx." in some Linux fonts, markers from
  200 up by a sliver in any font, and "120." in any font under WCAG's text-spacing test; with classic scrollbars the
  same happens at 736–744 px, where markers hang. No page has such a list. `padding-left: 2rem` on lists fits every
  three-digit marker in the fonts measured, at no cost in size, with lists 8 px further in on phones; it does not
  fit "120." under the text-spacing test (about 54 px needed) or the hanging markers at 736–744 px (the 46rem query
  would move to 47rem).
- A browser without `text-wrap: balance` breaks the oldest post's title as "Announcement: Rust- / Edu" in windows of
  about 342–406 px (today 358–432), whichever way decision 5 goes. WebKit breaks the newest post's row on `/news/`
  the same way in windows of 320–337 px.
- At 200% text the statement breaks inside words in windows up to 390 px wide.
- The photo label rule also matches a sentence with one bold phrase right above a photo. No page has one; the
  campus tour's labels could become `###` headings instead.

## Not tested

Real phones; networks in the places the campus tour visited (the live check reached both sites through one CDN
edge, in Lima); browsers older than Chrome 152 and Firefox 159; Safari itself (WebKit ran as macOS's WKWebView,
without a real Tab walk); timings in Firefox or Safari, at a device pixel ratio above 1, or in the dark theme;
Windows, Android and Linux system fonts; Android's text-size setting; Windows contrast themes (forced colors were
emulated); screen readers; Lighthouse; WebP served by GitHub Pages; `make ci` in Docker for any step, the Part B
patches included.

## How the numbers were taken

Chrome 152 headless on one laptop, through a local HTTP/2 server that sends GitHub Pages' headers (gzip,
`cache-control: max-age=600`). Phone view 412×823 at 1×, light theme, with a slow 4G profile: 150 ms round trip,
1.6 Mbit/s down, 750 kbit/s up, CPU slowed 4×; the high-latency rows use 400 ms and 400 kbit/s both ways. Medians of
7 cold runs; sizes are bytes on the wire with headers. A real connection adds its handshake, about 0.45 s at a 150 ms
round trip and 1.2 s at 400 ms, so the times are floors. The live figures are the home pages only, 3 cold runs each
against rust-edu.org and the fork. The per-page tables and the scripts are not in the repository; a research note
can hold them.
