# Plan 0004 — Minifier Crate

**Date:** 2026-11-11 · **Status:** proposed · **Follows:** [Plan 0002](Plan-0002—Own-Markdown-Site-Generator.md) and
[Plan 0003](Plan-0003—Visual-Refresh-and-Page-Speed.md)

**Goal:** the site publishes its stylesheet without comments and indentation, and its two inline scripts without
their comments. A fifth crate, `edu-minify`, on the standard library alone, does the first (steps 2 to 4); three
lines of one layout do the second (step 1). A first visit to a text page gets 2.6 KB lighter (16 to 19%). No reader
will feel that: it buys room under Plan 0003's page budget, where the largest text page goes from 1.07 KB over its
15 KB to 1.54 KB under. Deleting the comments from `static/style.css` by hand would give nine tenths of that with no
Rust, so the crate is the price of a stylesheet that keeps its comments, today's and those of every later change:
about 1,200 changed lines in three pull requests (1,100 in the prototype), and a check that stops at an unquoted
`url(…)`, a `\` outside a string or `@charset`. In the engines tried, no page of today's site looks or behaves
differently. The same crate can also take the indentation out of the layouts' HTML and scripts (steps 5 to 10, Part
C); that was built as a prototype too and is left to decision 1, since it needs about twice the changed lines for
about 0.13 KB a page.

## Where the site stands

Measured on `main` of the fork kenesparta/rust-edu.github.io at 2bd9da3, which was the head of the open upstream PR,
rust-edu/rust-edu.github.io#14, on 2026-11-11 (method at the end). rust-edu.org serves the earlier Zola build until
that PR lands, so these are the files that it will serve. "Today" is that commit, and the names of functions and
types are its names. The prototype was built six of the fork's pull requests earlier, on 601084a, and replayed on
c3a6adf; a figure that names one of those commits was taken there. KB is 1,000 B, and "with gzip" is `gzip -9 -n`,
as in Plan
0003. A text page is one of the nine pages of content without photos; the photo page is the campus tour's post.

| | Today | After step 1 | After step 4 | After step 10 (Part C) |
|---|---|---|---|---|
| `style.css` as published | 21,958 B, 5,924 B with gzip | the same | 14,294 B, 3,438 B with gzip | the same |
| HTML of a page of content, with gzip | 2,007–5,870 B | 108–124 B less | the same | 243–263 B less than today |
| Text page, first visit | 13.8–16.1 KB | 13.7–15.9 KB | 11.2–13.5 KB | 11.1–13.3 KB |
| Later text page of the same visit | 2.3–4.6 KB | 0.12 KB less | the same | 0.24–0.26 KB less than today |
| Largest text page against the budget's 15 KB | 1.07 KB over | 0.95 KB over | 1.54 KB under | 1.66 KB under |
| Photo page against its 15 KB, before its photos | 2.68 KB over | 2.57 KB over | 0.08 KB over | 0.05 KB under |

- **The stylesheet grew, and the site is over its budget.** Plan 0003 limits `style.css` to 7,632 B, and to 2.5 KB
  with gzip; its own stylesheet was 6,547 B. The theme button, the sticky header, the feed menu, the band at the top
  of the home page and the 404 page have since brought the file to 21,958 B and 5,924 B with gzip, and a first visit
  to the largest text page, `/workshop/cfp-2022/`, to 16.07 KB against that plan's 15 KB. Only review holds the
  limits, and it has not held them: the stylesheet went over both of its own limits on 2026-11-09, the date of Plan
  0003, and has stayed over them through 16 versions.
- **Comments are most of the saving, and the part that grows fastest.** The stylesheet holds 38 comments, 5,023 B of
  its 21,958 B. Without them and the lines that they fill, it is 2.3 KB smaller with gzip; taking the indentation
  and the empty lines out as well saves 0.2 KB more. What step 4 takes out was 21% of the stylesheet with gzip on
  2026-10-28 and is 42% today, since each new rule explains itself.
- **Every page carries the same layout.** The three comments of its two inline scripts cost each page of content
  108–124 B with gzip, and the indentation and empty lines of the layouts 125–144 B more.
- **GitHub Pages sends gzip only.** A request that accepts only brotli gets the file uncompressed (checked on
  rust-edu.org and on the fork's Pages site on 2026-11-11), and Pages serves no file that was compressed ahead of
  time. A smaller file is the only way to send fewer bytes.
- **No reader will feel the saving.** By the bytes alone, 2.6 KB is about 13 ms at 1.6 Mbit/s and 52 ms at 400
  kbit/s, and no round trip goes away; no timing was measured. Plan 0003 found the same: its steps took 5 KB off a
  first visit, and first paint was no better.
- **What Plan 0003 said.** It left "minified CSS or HTML" out, at "about 0.8 KB once for the stylesheet and 0.06 KB
  a page" on the files of its step 4. Today the same is worth 2.5 KB once and 0.25 KB a page (0.12 KB without Part
  C), most of the second in the two inline scripts, which the layout of that time did not have.

## The case against, and the case for

**Against.** No reader will feel 2.6 KB. The 15 KB limit that the saving serves is the project's own, and nothing
but review holds it. Step 1 and the comments deleted by hand reach 2.4 of the 2.6 KB with no Rust: the largest text
page is then 1.33 KB under its limit, against 1.54 KB. What the crate adds to that is 0.21 KB once a visit, and the
comments stay in the file. Its price is about 1,200 changed lines; a check that no version of the stylesheet has
failed so far, but that refused 14 of 29 edits tried as likely next ones, 11 of them valid CSS; a published
stylesheet that is no longer the file in git; and a fifth crate in a generator of which a maintainer has asked, on
the open upstream PR, whether the project is sure that it wants to maintain it (decision 2).

**For.** What the crate takes out is the part of the stylesheet that grows fastest: since the look of Plan 0003
merged on 2026-11-09, the file has grown by 3.4 KB with gzip as written and by 1.7 KB as step 4 would publish it.
With the crate, explaining a rule costs a reader nothing; without it, each comment is a choice between explaining a
rule and sending fewer bytes. The crate needs no change when the CSS changes. It gives the stylesheet its first
check: a comment or a string left open stops the build with a line and a column. It makes the stylesheet inside each
page cheaper, which is the one step of Plan 0003 that moves first paint (its step 9): 3.4 KB a page instead of 5.8
KB, where the comments deleted by hand give 3.6 KB. And reverting the PR of step 4 takes it out of the site again,
since the stylesheet stays plain CSS.

## Principles

| Topic | Rule |
|---|---|
| White space | Only the space, the tab, the line feed and the carriage return are white space here, and in a stylesheet the form feed too; all of them but the line feed are called blanks. A no-break space is content |
| What changes | In a stylesheet, comments go, each line loses the blanks at both ends, and empty lines go. In a layout (Part C), a run of white space that holds a line break becomes one line break. Nothing else changes |
| Bytes are only deleted | The published text is its source with bytes taken out, and no value, number, color, quote or name is rewritten: the crate writes only bytes that it has read. Two more statements are not given by that rule, only tested (step 3, *Risks*): minifying the published text again changes nothing, and every token of the source is there, in the same order, with white space between two tokens exactly where it stood in the source. (A token here is the smallest piece that a browser reads in CSS, such as a name, a number, a string or a brace; it is not a color token of the stylesheet.) |
| What is not promised | That the published text is the same for everything that reads it. A script or a style query that reads a value as text sees that a comment or an indentation is gone (*Known limits*) |
| Sources, not output | The crate reads what people write by hand: each stylesheet and, in Part C, each layout, once in a build. The HTML that the generator writes itself stays as it is |
| Errors | The crate follows comments, strings and quoted addresses. It stops, with `file:line:col` and a hint in `edu check`, at what could hide one of them from it or change how a browser reads the file, and at a comment left open. It checks nothing else: a property, a selector or an at-rule that is new to the site needs no change in Rust, and each of the six refused forms (*The design*) has another spelling that passes. Plan 0002 says of the stylesheet and the layouts that "changing them needs no Rust"; Part B keeps that, with those six forms to write differently |
| One path | `edu serve` serves the bytes that `edu build` writes: no flag and no second mode, so the preview shows what is published |
| Readable output | A line of the published stylesheet is a line of its source without its comments and without the blanks at its ends; the code before and after a comment that runs over lines becomes one line. "View source" still shows one declaration a line, though no comment and no indentation |
| Generator | The stylesheet stays plain CSS, which any generator can publish as written. The crate is Rust to keep for as long as the generator is kept, and nothing else depends on it |

## What each level is worth

A minifier can stop at one of several levels. The stylesheet's column is its size today; the page's column is what
each of the ten pages of content loses with gzip.

| Level | `style.css` with gzip | A page | What it takes |
|---|---|---|---|
| As written | 5,924 B | | |
| The scripts' comments written as template comments | | 108–124 B less | 3 lines of `layouts/layout.html` |
| Comments deleted from the stylesheet by hand | 3,650 B | | No Rust; `static/style.css` loses its 38 comments |
| Comments only: the crate takes the comments out and keeps every line and its indentation | 3,697 B (estimated by a script; not built) | | The crate of step 2 without the trimming; line N of the published file is line N of its source |
| **Lines:** comments out, the blanks at both ends of each line out, empty lines out | **3,438 B** | 243–263 B less with Part C, the scripts' comments included | Built as a prototype: 1,100 changed lines for the stylesheet in three commits, 2,109 for the layouts in six |
| Lines joined: no line break after `{`, `;`, `}` or `,`, nor before `}` | 3,381 B | | About 5 more lines of Rust, tried on a first sketch; the published stylesheet becomes one line of 13,538 characters |
| Tokens: no white space that the grammar can spare | Refused today; 96 B less than lines at c3a6adf | about 27 B less than lines (estimated by a script; not built) | Built as a second prototype for CSS, without the call from `edu-site`: its crate is 786 lines of Rust and 514 of tests after three commits, where that of the lines level is 340 and 410 after two (lengths of the files, not changed lines); 16 kinds of error against 6; every at-rule but `@media` and `@supports` refused until a line of Rust allows it. For scripts, a tokenizer would have to refuse `/`, since it cannot tell a division from a regular expression |
| What the established tools do (shorter values, no optional tags or quotes) | 3,300 B (esbuild) | 369–452 B less (html-minifier-terser with its strongest settings, at c3a6adf) | A parser and a printer for each language |

The lines level reaches 95% of what the deepest tool saves on the stylesheet with gzip (2,486 of 2,624 B). The token
level was built to learn its price. At c3a6adf it saved 96 B more, once a visit, and it refused 27 of 46 samples of
ordinary CSS that an editor might write next (`@font-face`, `@keyframes`, a rule inside a rule, a string in single
quotes). The next merge wrote one more: `content: "\2192"`, a `\` inside a string, so the token level refuses
today's stylesheet, which the lines level accepts. This plan takes the lines level: for the stylesheet by default,
and for the layouts only with Part C.

## The design

One new crate, `crates/minify` (package `edu-minify`), which uses no other crate and opens no file. It reads a text
once, from its start to its end, and writes the minified text as it reads; this plan calls that reading a scan. It
returns that text, or an error with a line and a column. `edu-site` calls it.

**Inline scripts (Part A).** No JavaScript is read. The three `//` comments of the two scripts become template
comments, `{# … #}`, which the generator does not publish. Nothing checks that convention: a `//` comment that
someone adds later is published as it is, and each page carries its bytes.

**Stylesheets (Part B).** `edu_minify::css` tells code, comments and strings apart, and reads no rule, selector or
value. A comment goes. Where a comment is all that stands between two pieces of code (`a/* x */b`), an empty comment
`/**/` stays: a space there would mean something, and nothing there would join the two. A comment that starts with
`/*!` stays as written, as the established minifiers keep it, for a notice that has to stay. A string stays as
written. Then each line loses the blanks at its two ends, and empty lines go. The scan refuses six things: five
because each could hide a quote or a `/*` from it, or change how a browser reads the file, and a comment left open
because it silences the rest of the file.

| Refused | The hint |
|---|---|
| A comment without its end | close it with `*/` |
| A string that does not end on its line | write it on one line; a comment is `/* … */`, not `//` |
| A `\` outside a string, as in `.sm\:flex` | select the class as `[class~="sm:flex"]`, or rename it; a value takes no `\` |
| A `url(` whose address is not in quotes | put it in quotes, as in `url("favicon.svg")` |
| `@charset` | delete it, since the site serves a stylesheet as UTF-8 |
| A byte order mark (U+FEFF) in code, anywhere but at the start of the file | delete it |

Every `.css` file of `static/` and `content/` goes through `file::check`, which today checks the SVG images, and is
published as the scan writes it (step 4). A stylesheet that is not UTF-8 is an error too. Whoever edits the
stylesheet writes it as before, and meets one new check. It prints the same line in `edu check`, in `make check`, on
the error page of `edu serve` and as an annotation on the pull request:

```text
static/style.css:740:23: error: the address of this `url(` is not in quotes; put it in quotes, as in `url("favicon.svg")`
```

**Layouts (Part C).** Each layout is minified once, when it is read. A template tag often stands inside an HTML tag,
as in `<a href="{{ item.url }}">`, so a layout cannot be read as one HTML text. `edu-template` gains one public
method, `Template::walk`, which hands a caller each stretch of a layout's own text between two template tags, in the
order of the source, and says where a `{{ … }}` stands and where the branch of an `if` or the body of a `for` starts
and ends. From one stretch to the next the scan remembers where in the HTML it is: in text, in a tag, in a quoted
value, in the text of a `script`, `style` or `title`, or in an inline SVG image. In text and between attributes a
run of white space with a line break becomes one line break. No run is removed whole, so the gap between the home
page's button and the link beside it, and the line break inside the footer's sentence, stay. Attribute values and
the text of `pre`, `textarea`, `title`, `style` and a script with a `type` stay as written. In a script without a
`type` the rule of text applies, so every line break between two lines of code stays and the script runs as it did.
A branch or a body must end where it starts: in text if it starts in text, and inside the same tag if it starts
inside a tag. The scan refuses fourteen things (thirteen in the prototype, and the obsolete elements of step 7),
among them an HTML comment (the hint: `{# … #}`), an attribute value that is not in double quotes, a `<` that starts
no tag, a `{{ … }}` inside a script, and, in a script, any backtick and a `\` at the end of a line, since a string
could then run over lines. An error names `layouts/x.html:line:col`. This scan is also the generator's first check
of the layouts' HTML. Today an HTML comment in a layout is published, and the link check does not read `<a
href='/nope/'>`, a value in single quotes.

## Steps

One branch and one PR per step, named as in Plan 0002, near 500 changed lines (600 at most), each passing `make
check` and `make ci` on the `main` of the day it opens; where the steps merge is decision 2. Each PR gives the size
of each published file that it changes, in bytes and in bytes with `gzip -9 -n`, before and after, measured at its
own base, and says which AI tools helped and how. The counts of changed lines in parentheses are those of a
prototype: nine commits for steps 2 to 10, each passing `make check` (*How the numbers were taken*). A correction
"against the prototype" is not built unless its step says so, and each step is counted again when its PR opens.
Steps 2 to 10 merge in their order, as the prototype's commits do: step 6 adds to the crate of step 2 and step 10 to
the example sites of step 4, and each scan is tested on damaged inputs (steps 3 and 9) before the site calls it
(steps 4 and 10). Steps 0, 2, 3 and 5 to 9 publish nothing new: each of them is done only when, besides what it
names, the site built before and after is byte-identical.

**Part A — no Rust**

0. `docs/minifier-plan` — this plan; `research/0007-minifier-prototype.md`, with what the two prototypes and the
   reviews of them found, the versions of the tools, and how each comparison is run; and a dated note under *Not in
   this plan* of Plan 0003 that points here. The two prototypes go to the fork as the branches `prototype/minify`
   and `prototype/css-tokens`, which merge nowhere, as `prototype/feeds` does; the scripts of the comparisons are
   kept on the first. *Done when:* merged with the research note, and both branches pushed to the fork, with
   decisions 1 and 2 answered and recorded as in Plan 0002 ("*Decided …*"); each other decision is recorded before
   the step that it names.
1. `perf/script-comments` (three lines of `layouts/layout.html` rewritten, which are 6 changed lines, and about 4 of
   docs) — every `//` comment of the layouts' scripts (three today) becomes `{# … #}`, and `AGENTS.md` says that a
   comment in a layout, in a script too, is written so. *Done when:* no line of a script in a built page starts with
   `//`; each page from the layouts is smaller by the bytes of the comments (245 B today, and 108–124 B with gzip);
   the theme button works as before in the PR's `site-preview`, the built site that CI attaches to each PR.

**Part B — the crate and the stylesheet**

2. `feat/minify-css` (598 changed lines in the prototype: 356 of code, 228 of tests, 14 of docs; about 595 here,
   with the lines that CI's check of `Cargo.lock` needs and without the test on `static/style.css`, which is step
   3's) — the crate, with `Pos`, `Error`, `ErrorKind` and `css`, and a test that goes through a table of rows, each
   an input with the text or the refusal that it must give: a row for each case of the scan's rule, and one for each
   refusal with its line and column. `Cargo.toml` gains the member and the dependency, and the crate's own manifest
   takes the workspace's lints, as the other four do. CI fails when `Cargo.lock` names any crate but the four, so
   the same PR changes that list, its count and its message in `.github/workflows/ci.yml`, the *Standard library
   only* lines and the `crates/` row of `AGENTS.md`, and in `README.md` the *Standard library only* line, the
   heading and the entries of the list of crates, and the diagram of *How it works*. It adds a bullet to the
   *Generator* list of `AGENTS.md`: the minifier only deletes, and what its scan does not follow is an error; it
   reads no rule or value, so it is no parser, and *Strict parsers* does not ask it for a subset of CSS. It adds a
   line to the README's *AI Acknowledgement* if the crate is written with another model than the one that the README
   names for the generator, and a dated note where Plan 0002 says four crates. The `Dockerfile` and `.dockerignore`
   need no change. Against the prototype: the three hints that the table of *The design* gives for a string, for a
   `\` and for a `url(`; the prototype's hint for a `\`, "write the character itself", leads to a stylesheet that
   passes and is wrong. Nothing calls the crate yet. *Done when:* CI's check of `Cargo.lock` passes with five
   crates.
3. `test/minify-css-invariants` (195 changed lines in the prototype: 192 of tests, 3 of docs; about 290 with the
   additions, a few of them code) — four statements that must be true of every stylesheet: the published text is its
   source with bytes deleted; minifying it again changes nothing; its tokens are those of its source, by a second,
   simpler reader of CSS that is part of the test; and a text is refused exactly when that reader says so. The test
   tries them on every row of step 2's table, on `static/style.css`, on 5,000 damaged stylesheets from a seeded
   random-input test, as in the other crates, and on every text that can be put together from up to six of a few
   short fragments, such as `a`, a blank, a line feed, `/*` and `*/`. It reads `static/style.css` when it runs,
   since the `Dockerfile` lints the tests, which compiles them, before it copies `static/`. Against the prototype:
   the test prints a refusal of that file as `edu check` will; and, with a row each, `#url(` and `@url(` are not
   taken for a `url(`, nor `@charsets` for `@charset`; a `url(` right after a character outside ASCII is checked for
   quotes like any other; and five rows for the faults that reviewers put into the scan on purpose and that every
   test of the prototype let through. *Done when:* the four statements are true in `make check` on all four sets of
   inputs; the PR's description names the comparison with css-tree and the browser engines that was run by hand for
   this scan, with versions and counts, since CI cannot repeat it.
4. `feat/minified-stylesheets` (307 changed lines in the prototype: 104 of code, 118 of tests, 59 of example sites,
   26 of docs) — the call from `edu-site`: `file::check` returns the minified text of a stylesheet, `check::pages`
   keeps it, `build::plan` gets a new kind of `Output` for it, with the text and the path of its source, so that two
   files for one address still clash, and `serve::body` serves it. `crates/site/Cargo.toml` and `Cargo.lock` gain
   the dependency. New errors, each a variant of its own: one of `PageError` for the crate's error, and one of
   `FileError` for a stylesheet that is not UTF-8. Two example sites, `minified` and `unminifiable`; against the
   prototype, the second also holds an SVG image that is not UTF-8, since no test notices today when the error for
   such an image is dropped. Three of today's tests change on purpose, though the step breaks none of them: the
   build's test checks that an image, not `style.css`, is published byte for byte; the file `gone.css` of a test
   becomes `gone.txt`, since a stylesheet is no longer copied; and the server's test serves a stylesheet with a
   comment and then a broken one. Docs: in `README.md` the arrows of the diagram of *How it works*, the list of safe
   types of file, the entry of `crates/site` and, under *Layout and Style*, what a stylesheet may not hold, with
   examples, that only a `/*!` comment is published and, by decision 6, that a comment or a line break inside the
   value of a custom property is not published as written; one sentence in `docs/editing.md` where it lists the
   types of file; the `static/` row of `AGENTS.md`; the first comment of `static/style.css`, which says "no build
   step"; and a dated note at the *Other files* row of Plan 0002 and, by decision 8, at the `style.css` row of Plan
   0003's budget. *Done when:* `style.css` is published as the scan writes it (14,294 B and 3,438 B with gzip today;
   the PR gives the sizes at its own base) and no other published file changes; a refused stylesheet shows as
   `path:line:col: error:` in `edu check`, in `edu build` (which writes nothing) and on the error page of `edu
   serve`; and on `/`, `/news/`, `/news/campus-tour/` and `/404.html` at 390 and 1280 px, in light and dark, every
   element's box and every line of text are the same before and after in Chromium and in WebKit, with screenshots in
   the PR.

**Part C — the layouts, only if decision 1 takes it**

| Step | Git branch | Changed lines in the prototype | What it adds |
|---|---|---|---|
| 5 | `feat/template-walk` | 244: 149 of code, 90 of tests, 5 of docs | `Template::walk`, and `Step`, the type of what it hands to its caller; the parser keeps the position of each stretch of text, and of the start and the end of each branch of an `if` and each body of a `for` |
| 6 | `feat/minify-markup` | 565: 323 of code, 234 of tests, 8 of docs | Text, tags, attribute values and runs of white space. Until step 7 merges, the scan refuses `pre` and every element whose text is no markup |
| 7 | `feat/minify-elements` | 498: 295 of code, 195 of tests, 8 of docs; about 60 more for the corrections, by estimate | `pre`, `textarea`, scripts, `style`, `title` and inline SVG |
| 8 | `feat/minify-template-tags` | 248: 115 of code, 129 of tests, 4 of docs; about 30 more for the corrections, by estimate | Where a `{{ … }}` may stand, and the rule that a branch of an `if` or a body of a `for` ends where it starts |
| 9 | `test/minify-markup-invariants` | 289: 287 of tests, 2 of docs | The statements of step 3 for layouts, on the site's own layouts and on 5,000 damaged ones; five rows for the faults put into the scan on purpose that every test of the prototype let through |
| 10 | `feat/minified-layouts` | 265, 6 of them step 1's lines: 63 of code, 83 of tests, 86 of example sites, 33 of docs | The call from `Layouts::read`; an example layout for each kind of error; the docs |

As one PR, steps 6 and 7 would be about 1,060 lines, so they are two PRs that are opened together. The corrections
against the prototype, each with its rows in the table of its step's test:

- **Step 7, SVG text.** The text of an SVG `text` element stays as written: WebKit joins its words otherwise (9
  lines, tried as a patch without a test; the site's bytes do not change). Step 10 adds a layout with such a `text`
  to the example site `minified` and a WebKit picture of its page to its PR, since neither parse5 nor a page of the
  site can show the fault, and nothing calls the scan before step 10.
- **Step 7, elements.** The obsolete `listing`, `plaintext`, `xmp`, `noembed` and `noframes` are refused, with a
  hint that names `pre` for the first three and says to delete the other two: the prototype read the first two
  wrongly. An end tag of `svg` or `math` closes the nearest open element of its name, and nothing when none is open,
  and `<!--` is refused in `noscript` too.
- **Step 8, branches.** A branch may hold an attribute without a value, as in `<details{% if x %} open{% endif %}>`,
  and a `type` on any element but `script`. Of ten ways to write the first, the prototype refused seven, and it
  refused all three tried of the second.
- **Steps 6 to 8, hints.** Seven messages get a new hint. Each names a way out that passes the prototype's scan and
  is right: among them `&quot;` for a `"` inside a value, a comment as `{# … #}` for a backtick, and, for a branch,
  one form that passes, written out. The new wording was not tried on an edit (*Risks*).

*Done when:* for steps 5 to 9, nothing calls the new code; no error of today changes its position; each case, each
refusal and each correction has its row, with a line and a column; and step 9's description names the comparison
with parse5, acorn and the browser engines that was run by hand, with versions and counts. *Done when:* for step 10,
each page from the layouts is smaller with gzip than before step 1 (by 243–263 B for a page of content today); every
file that no layout makes is byte-identical; a refused layout shows as `layouts/x.html:line:col: error:` in `edu
check`, in `edu build` and on the error page of `edu serve`; the boxes and the lines of text of step 4's test are
the same before and after; the theme button and the feed menu work as before; and `README.md` and `AGENTS.md` say
how a layout writes a comment, an attribute and an attribute that is not always there, and that a layout's text
loses its indentation and its empty lines whatever `white-space` says.

## Page budget

These are four of the nine limits of Plan 0003's page budget, the four that this plan moves. It changes no photo and
not the logo. First paint and layout shift were not measured for it, and no PR of this plan gives their medians,
which that budget asks of each PR: each step only takes bytes out of what a page sends.

| Item | Limit | Today | After step 1 | After step 4 | After step 10 (Part C) |
|---|---|---|---|---|---|
| `style.css` | ≤ 7,632 B, and ≤ 2.5 KB with gzip | 21,958 B and 5,924 B | the same | 14,294 B and 3,438 B | the same |
| HTML of one page (a guide: a long post may pass it) | ≤ 5 KB with gzip | 2.0–5.9 KB | 1.9–5.8 KB | the same | 1.8–5.6 KB |
| First visit to a page without photos | ≤ 15 KB, ≤ 4 requests (5 on the home page with warm) | 13.8–16.1 KB, 4 requests | 13.7–15.9 KB | 11.2–13.5 KB | 11.1–13.3 KB |
| First visit to the photo page, before scrolling | ≤ 15 KB + 90 KB per photo that loads | 17.7 KB + its photos | 17.6 KB + its photos | 15.1 KB + its photos | 14.9 KB + its photos |

Step 4 brings the first visit to every text page back under its limit, the largest by 1.5 KB. The photo page stays
0.08 KB over, and comes 0.05 KB under only with step 10. Only the HTML of the campus tour's post is over the guide
of 5 KB, as that limit allows a long post to be. Step 4 does not bring `style.css` under its own two limits, and no
tool does: the deepest leave 12,362 B (lightningcss) and 3,300 B with gzip (esbuild). Those two limits were written
for a stylesheet of 6,547 B; decision 8 is about them. GitHub Pages' own gzip is larger than `gzip -9`: it serves
today's stylesheet as 6,038 B, 1.9% more, which is what `gzip -5` gives. By that measure step 4 publishes 3,484 B,
and the photo page stays over its limit after step 10 too, by 9 B (by 0.16 KB after step 4). The room does not last
by itself: between 2026-11-09 and 2026-11-11 the stylesheet as step 4 would publish it grew from 1,755 to 3,438 B
with gzip.

## Decisions for review (default first)

Decisions 1, 2 and 8 are about how much Rust the project takes and what it holds itself to. Decisions 3 to 7 are
about what whoever edits the stylesheet meets. Decisions 9 and 10 matter only with Part C. An alternative of
decision 4, 5 or 6 changes the scan and does not fit the 600 lines of step 2: it is a PR of its own between steps 3
and 4, with a new run of the comparison that step 3 names.

1. **How far** (before step 1): **steps 1 to 4, if the stylesheet's comments are to stay in the file that is
   edited** (*The case against, and the case for*). They hold 95% of what all ten steps save on a first visit (on
   average 2.61 of 2.74 KB) for a third of the changed lines (1,100 of 3,209). Or step 1 alone, and someone deletes
   the comments of `static/style.css` by hand or moves them to a note beside it: no Rust, and the largest text page
   is 1.3 KB under its limit. Or nothing: that page stays 1.07 KB over a limit that no reader feels. Or all ten
   steps: Part C is worth 125–144 B a page of content with gzip, as much as step 1: about 1% of a first visit and 2
   to 6% of each later page. Its price is 2,109 changed lines, fourteen more kinds of error (the prototype's
   thirteen refused 22 of 54 likely edits to the layouts) and one more public method of `edu-template`. It is worth
   that only if the project wants a check of the layouts' HTML, with `layouts/x.html:line:col`, for its own sake.
2. **Where and when the steps merge** (before step 0): **into the fork's `main`, as in Plan 0002 and Plan 0003:
   steps 0 and 1 at once, the Rust of steps 2 to 4 after the upstream PR has landed, and from there upstream in one
   PR for each part**. The head of rust-edu/rust-edu.github.io#14 is the fork's `main`, so every merge there joins
   that PR while it is under review. A maintainer wrote there on 2026-11-08: "We're *sure* we want to maintain our
   own static site generator forever, right? That's no small commitment." The PR's author answered yes, and the
   maintainer has since asked only that the Zola deploy be fixed before the merge, which upstream #15 did. No
   maintainer has asked for a minifier, and Part B would add 370 lines of Rust outside tests to the generator's
   5,945 (6%), and Part C 711 more (18% in all). Steps 0 and 1 have no Rust, but merged into the fork's `main` they
   join that PR too, as this document and three lines of a layout; if that is not wanted there, they wait with the
   rest. Until step 4 merges, the largest text page stays over its limit, by 0.95 KB after step 1. Or every step
   now, into the fork's `main`, where Rust has gone all through the review of #14: the Atom feed, which a maintainer
   had asked for, with an RSS feed beside it, and since then the 404 page and the links to other sites. Or on a
   branch `next` of the fork, which keeps #14 as it stands (Plan 0003, decision 2).
3. **Level** (before step 2): **lines, as built**: 2,486 B less with gzip, and 756 lines published for 986 written.
   It is the level that was compared with css-tree and in the engines; any other level changes the scan, which is
   then built and compared again before step 2. Or lines with an empty line kept wherever a line is dropped: 66 B
   more, which no reader feels, and a rule keeps its line number in the browser's developer tools, in the preview
   too (today `.site-hero h1 {` is written on line 576 and published on line 425); the scan then has to keep a line
   break that stands inside a comment. Or comments only: 259 B more than lines, with line numbers and indentation
   kept, and a value over two lines stays as written. Or lines joined: 57 B less, and one line of 13,538 characters.
   Or tokens, at the price in the table of levels: it refuses today's stylesheet.
4. **What a stylesheet may not hold** (before step 2): **an unquoted `url(…)`, `@charset` and a `\` outside a string
   are refused**, as in the prototype. Today's stylesheet has none of them, each refusal names a fix, and to refuse,
   not to guess, is what `AGENTS.md` asks of a parser; the scan is none (step 2), and keeps that rule all the same.
   Or the plain form `url(/x.svg)` passes (20 to 25 lines of code and one more kind of error, by estimate; not
   built). It is the usual hand-written form: 138 of the 162 refusals among the stylesheets of other projects that
   were tried, and 5 of the 14 refused edits; the first image or font in the stylesheet would meet the refusal. Or,
   with either, `@charset "utf-8";` passes as the first bytes of a file (about 8 lines).
5. **Comments that stay** (before step 2): **a comment that starts with `/*!`**, for a notice that has to stay. Or
   none. Or, with `/*!`, a plain comment that holds `copyright` or `license` is an error whose hint is to start it
   with `/*!` (one more kind of error; not built), so that no notice is deleted unnoticed.
6. **Custom properties** (before step 2): **a known limit, written in `README.md`, and no check**: the scan can
   change the text of a custom property's value, which a style query compares (*Known limits*). Today's stylesheet
   has no style query and the site's scripts read no style; and since `edu serve` serves what is published, a query
   that the scan changes shows in the preview as it will on the site. Or the scan refuses `style(`, with
   `file:line:col` and a hint, until the site needs a style query (about 20 lines and one kind of error; sketched,
   not compared with css-tree or in an engine). Or the scan keeps as written both the value of a custom property and
   the value inside a `style(…)` test (40 to 60 lines and one or two kinds of error; not built): both or neither,
   since a scan that keeps the one and trims the other makes a query fail that matches as written.
7. **When a scan refuses** (before step 4): **the check stops, and with it the preview and the deploy**, as for
   every other mistake in the site. That is right for a comment or a string that never closes; for valid CSS that
   the scan does not follow (the three forms of decision 4, a string carried over a line end with a `\`, and a byte
   order mark inside the code), it is the price of the refusal. Or the check and the deploy stop, and the preview
   alone serves the refused file as written, with the error in the terminal (against *One path*). Or the file is
   published as written with a warning: no edit is ever blocked, but the generator has no warnings today (not built,
   not sized), and a stylesheet can then stop being minified without anyone noticing.
8. **The budget's limit for the stylesheet** (before step 4): **no limit of its own**. The 15 KB of a first visit,
   which is what a reader pays, already limits the stylesheet: after step 1, to 5.0 KB with gzip beside the largest
   text page and to 3.35 KB beside the photo page, whose HTML is 5.8 KB. Step 4 publishes 3,438 B, 84 B more than
   the photo page leaves (*Page budget*), and each new rule has to be paid for there. Plan 0003's two limits for the
   file go, by a dated note there: step 4 leaves the stylesheet over both, and no tool brings it under them (*Page
   budget*). Or one limit for the published file: ≤ 4.1 KB with gzip, which is what 15 KB leaves beside a page at
   its own 5 KB; the photo page, which may pass that guide, could then be 0.75 KB over its 15 KB. Or Plan 0003's two
   limits stay, and the stylesheet stays over them. With any of the three: review holds the limit, as today, though
   review alone has not held one so far; or a step of CI holds it from step 4 on, as Plan 0003's step 5 proposes for
   images (8 lines of `ci.yml` there; this one is not built and not sized).
9. **Where Part C scans** (before step 5, if decision 1 takes Part C): **each layout, once, when it is read**, as in
   the prototype. Or each rendered page: steps 5 and 8 go (492 of the 2,109 changed lines, 4 of the fourteen kinds
   of error, and `Template::walk`), each page is 3 to 12 B smaller with gzip, since no empty line is left where a
   template tag stood, and the scan reads what a `{{ … }}` really writes. But an error then points into a built
   page, not into a layout, a branch that no page takes is not checked, and the scan also reads the HTML that the
   Markdown crate writes, against *Sources, not output*, though the build already passes over each rendered page for
   its links. This alternative was not built.
10. **Where the layout scan lives** (before step 5, if decision 1 takes Part C): **in `edu-minify`**, as in the
    prototype. Or in `edu-site`, beside `svg.rs`: its errors and hints name `{{ … }}` and `{# … #}`, the words of
    the template language, which `edu-minify` does not otherwise know.

## Not in this plan

- Rewriting anything: shorter colors, numbers or units, optional tags and quotes, white space between two tags,
  shorter names.
- Tools from outside (esbuild, lightningcss, html-minifier-terser). In the Docker image they would save more than
  this plan with no Rust to keep: esbuild 138 B more on the stylesheet with gzip, and html-minifier-terser about 0.3
  KB more a page than steps 1 to 4. Such a tool would be the generator's first dependency that needs updates (Plan
  0002: "no dependency updates ever"), and a second path: `cargo run -p edu-site -- serve` without Docker would no
  longer show what is published (Plan 0002: "plain `cargo` works too"), against *One path*.
- `.js` files: the site publishes none, and `edu check` stops at one.
- The SVG images, which are one line each, the feeds, `sitemap.xml`, and the HTML that the Markdown crate writes,
  which has no indentation.
- Source maps, and file names that change with the content.
- The stylesheet inside each page (Plan 0003, step 9), which step 4 would make cheaper.
- A check of the page budget by the generator. A step of CI is in decision 8.
- The two gaps named at the end of *The design*, without Part C: the link check could read a value in single quotes,
  or stop at it, and the check of a layout could stop at `<!--` (not sized).
- The addresses inside `url("…")` of a stylesheet. The link check does not read them, and the build does not put
  them under the path of `--base-url`: `url("/favicon.svg")` breaks at a fork's Pages address, and
  `url("favicon.svg")`, which a browser reads from the stylesheet's own address, works there and at rust-edu.org.
  Step 4 leaves every address in quotes, so a later step could read them.

## Risks

- **More Rust in a generator whose upstream review is open** (decision 2). The crate adds no dependency.
- **A new check that can stop whoever edits the stylesheet.** Nothing that the stylesheet or a layout has held is
  refused: all 22 versions of `static/style.css` and all 24 versions of the six layouts in the history of `main`
  pass. But of 29 small edits to the stylesheet that an editor might make next (tried at 601084a), 14 are refused,
  where today's check refuses none: 11 for valid CSS (an unquoted `url(…)` five times, a `\` three times, one of
  them the old `\9` hack, whose declaration today's browsers drop, `@charset` twice, a string carried over a line
  end) and 3 for a slip that is published broken today (a `//` comment with an apostrophe, a comment and a string
  that never close). Of 54 such edits to the layouts, 22 are refused. Every refusal named the right line and column.
  The prototype's hint alone led to a right fix in 10 of the 14 refusals of a stylesheet and in 14 of the 22 of a
  layout; steps 2 and 6 to 8 reword the others, and the new hints were not tried on an edit.
- **No test in CI sees a browser.** That each page looks the same was shown by hand: in three engines on the files
  of 601084a, and on today's in Chromium and WebKit, with Firefox on the ten pages of content as step 10 leaves
  them. Steps 4 and 10 repeat it in Chromium and WebKit, on four pages; step 1, which changes every page too, shows
  the theme button at work.
- **The tests can have the same faults as the code, and the proof is outside the generator.** Three faults of the
  first stylesheet scan (two comments one after the other, U+FEFF after a comment, `<!--url(`) were found by
  comparing with css-tree on random inputs; the second reader in the test had two of them and so could not see them.
  A reviewer cannot see in a diff that a change to a scan is sound: each change costs a run of the comparisons, with
  Node, css-tree, parse5, acorn and the browser engines, none of which the generator or its Docker image has. Step 0
  keeps the scripts.
- **A bug in a scan is a bug on every page.** There is no flag. Reverting the PR of step 4 publishes the stylesheet
  as written again, and that of step 10 the layouts.
- **A published file that is not the source.** The developer tools show another line than the source's for a rule
  (decision 3), and with Part C the line that a browser names for an error in a script is a few lines above that
  line in the layout. From step 1 on, the text of a script in `layouts/layout.html` is JavaScript only once the site
  is built: a code editor or a linter that reads the script there stops at each template comment.
- **The site changes several times a day.** The stylesheet grew from 16,239 to 21,958 B while the prototype was
  built and reviewed. Replayed on today's `main` with each conflict resolved, four of the prototype's nine commits
  conflict: the first in `README.md` and `AGENTS.md`, the one that adds `Template::walk` in `README.md`, and both
  calls from `edu-site`, in `build.rs` and `serve.rs` and in `render.rs` and `AGENTS.md`. Sizes are counted again at
  each step's base.
- **`make check` gets slower.** The two tests of 5,000 damaged inputs take 1.9 s (stylesheets) and 2.7 s (layouts)
  in a debug build on one laptop, where the slowest test file takes 0.35 s today; step 3's test of every short text
  takes about 1.9 s more of one core.

## Known limits

Of Part B:

- **Text that a browser keeps as written.** The value of a custom property, a value with `var()`, the condition of
  `@supports` and a part of `:is()` that an engine cannot read are handed to a script as text, so a comment or a
  line break with indentation inside one reads differently in the published file. A style query compares such text,
  in `@container style(--x: …)` and in the `if()` function with a `style(…)` test, so there the scan can change what
  is painted; the smallest case is `a{--x:b /**/ c}` with a query for the same value without the comment, in all
  four engines tried. Today's stylesheet declares 16 custom properties 42 times, each on one line without a comment;
  its 78 values with `var()` and its two `@supports` conditions are on one line each; and it has no style query. Two
  of its earlier versions wrote `--font-sans` over two lines.
- **CSS from elsewhere is refused one time in seven, and can lose its notice.** Of 1,167 stylesheets of other
  projects, found on the laptop where the numbers were taken, 162 are refused: 138 for an unquoted `url(…)`, 11 for
  `@charset`, 7 for a `\`, 6 for a string that does not end on its line. Of the 1,005 that pass, 605 hold a
  copyright or license notice, and 219 of them would lose it, since only `/*!` marks a notice.
- **Every `.css` file is taken for the site's own.** One that a page offers to read or to download, in `content/`
  too, is published without its comments and indentation, and nothing marks a file to be published as written.
- **The scan is no validator.** `}}} not css {{{` passes, as it does today, and a `//` line is published as it is.
- **An empty comment stays in a few places where it need not**, as in `a {}/* x */b {}`: the scan reads no tokens.
- **An unfinished stylesheet stops the preview.** `edu serve` shows the error in place of every page until the file
  is fixed. A stylesheet in a draft folder can stop the deploy, as an SVG image there can today.
- **One error for each file at a time.** A pasted stylesheet with three unquoted addresses is fixed in three rounds.
- **Line ends.** A stylesheet saved with CR LF line ends is published with line feeds, the same bytes as if it had
  been saved with line feeds, except for the inside of a `/*!` comment, which stays as written.
- **Bytes pinned elsewhere.** An `integrity` attribute or a precache list would have to hash the published file. The
  layouts have none.

Of Part C, besides the corrections that steps 6 to 8 name:

- The scan's rule for white space is right only while nothing but `pre` and `textarea` gets `white-space: pre`,
  `pre-wrap`, `pre-line` or `break-spaces`, or the same through `white-space-collapse`, from the stylesheet, from a
  `style` element or from a `style` attribute of a layout. Today's stylesheet and layouts have none; nothing checks
  it.
- The scan does not pair tags. A `</pre>` or a `</svg>` that a browser ignores, as a `</pre>` inside an open
  `table`, ends the element for the scan (once in the 280,000 generated layouts that were tried).
- A `{{ … | safe }}` is taken for whole elements, as the Markdown crate writes them; nothing can check a value
  before a page renders.
- `noscript` is read as a browser with scripts reads it, as text that is no markup. A browser without scripts reads
  markup there, and a `</noscript` inside a value of that markup ends the element for the scan.
- An empty line stays where a `{% … %}` tag stood alone on its line, and where the line breaks of two layouts meet
  the one that ends a page's text: 114 lines and 72 B with gzip in all, over the ten pages of c3a6adf.
- What scripts read changes in white space only: the text of text nodes, and in WebKit one space of `innerText`
  before an image or a form control.
- A script comment has to be a template comment, a script with a `type` is published as written, a `{{ … }}` cannot
  stand in a script or in JSON-LD, and a backtick is refused even inside a comment.
- A layout with very many template tags is read slowly, since step 5 as built finds each position by reading the
  layout from its start: about 20 s for 180,000 template comments on one line and four to five times that with one
  comment a line, and under 0.01 s for the site's layouts.

## Not tested

Safari itself (WebKit ran as Playwright's build, and Safari 27.2's as macOS's WKWebView for stylesheets only);
phones; screen readers; engines older or newer than Chromium 151, Firefox 159 and 160, and WebKit 26.5; Firefox and
Safari 27.2's WebKit on the rules of today's stylesheet (both read those of the versions up to 11e72aa); Firefox on
today's `404.html`, on today's pages in the light theme and on them as step 4 leaves them; any timing in a browser,
first paint and layout shift among them; a Windows checkout; GitHub Actions itself (its check of `Cargo.lock` and
the filter that makes its annotations were run by hand); Lighthouse. The corrections that the steps name, except the
one for SVG text, and the reworded hints. The alternatives of decisions 3 to 9, except the token level, which was
built: they are sized by a script, by estimate or not at all.

## How the numbers were taken

Sizes are those of `edu build` at 2bd9da3, with Apple gzip 487.40.2 as `gzip -9 -n -c <file> | wc -c`. A first visit
is counted as in Plan 0003: the gzip sizes of the page, `style.css`, the logo (2,752 B) and the icon (1,932 B), plus
0.3 KB of headers for each of the four requests. A later page is its own gzip size plus 0.3 KB. A step's count of
changed lines is its added plus its deleted lines, with the test modules inside the source files of `edu-site`
counted as tests; where a part "adds" lines, the deleted ones are taken off. The generator's 5,945 lines are those
of `crates/*/src` above each file's tests.

The prototype is nine commits on 601084a: three for the stylesheet (1,100 changed lines) and six for the layouts
(2,109), each passing `make check`, the last passing the check stage of the `Dockerfile`, with 171 tests against the
142 of that commit. Replayed on c3a6adf they conflict once, in `README.md`, pass `make check` at each commit, and
fail CI's check of `Cargo.lock` until step 2's change to `ci.yml`; on today's `main` four of the nine conflict
(*Risks*). Today's sizes after steps 4 and 10 were taken without a replay: the stylesheet and each layout went
through the prototype's two scans, and today's generator built the site from them. A replay on today's `main`, made
for the last review of this plan, passes `make check` at each of its nine commits and builds the same site, byte for
byte. A second prototype of three commits built the token level for CSS. Before anything was built, the estimate was
about 450 changed lines for the stylesheet and 270 for the layouts: to read a stylesheet and a layout as a browser
does took 2.4 and 7.8 times that.

What was compared, with css-tree 2.2.1, parse5 8.1.1, acorn 8.19.0, esbuild 0.28.2, lightningcss 1.33.0,
html-minifier-terser 7.2.0, Chromium 151.0.7922.34 and WebKit 26.5 (Playwright 1.62.1), Firefox 159.0.1 and 160.0,
and Safari 27.2's WebKit:

- **The stylesheet scan:** 12.9 million texts, 12.3 million of them every text that can be put together from a few
  short fragments: up to five, six or seven of them, by the set of fragments. For each of the 4.8 million that were
  accepted, the published text is its source with bytes deleted, minifying it again changes nothing, and a tokenizer
  written for this comparison from the CSS Syntax Level 3 specification reads the same tokens in it as in its
  source. So does css-tree, except for 68 texts whose published text starts with U+FFFE, which css-tree skips as it
  skips a byte order mark and none of the three engines tried does. In each of the four engines the source and the
  published text give the same rules for the 20 versions of `static/style.css` up to 11e72aa, for 23,000 generated
  stylesheets (5,500 in Safari's WebKit) and for the 652 stylesheets of other projects that the scan changes, except
  for text that an engine keeps as written (*Known limits*), which has the same tokens and other white space: the
  value of a custom property or a value with `var()` that is written over lines, in the first two of those versions
  and in 10 of the 652, and such text with a comment or a line break in it in 12 to 16% of the generated
  stylesheets. Today's stylesheet, the 22nd version, ran in Chromium and in Playwright's WebKit: its 113 top rules
  read the same.
- **The pages at 601084a:** parse5 reads the same DOM from all 12 HTML files (the ten pages of content, `404.html`
  and the redirect page), once a run of white space with a line break counts as one line break; acorn reads the same
  syntax tree from the two scripts of each of the ten pages; 88 of 88 views are the same in each of Chromium, WebKit
  and Firefox (11 pages, the ten of content and `404.html`, at four window sizes, light and dark): every element's
  box, every line of text, and the picture, for which three views needed more than one load in Chromium, whose
  picture of one build varies; the theme button and the feed menu go through the same states.
- **The pages today,** as step 4 leaves them and as step 10 leaves them: the same DOM; 44 of 44 views the same in
  each of Chromium and WebKit (the same 11 pages, two window sizes, light and dark), with the theme button through
  the same states; and, for step 10, 20 of 20 views in Firefox 160.0 (the ten pages of content, two window sizes,
  the dark theme).
- **The layout scan:** 280,000 generated layouts, 98,516 accepted: no panic, no error at a wrong position, and 32
  layouts whose pages differ: 16 in the obsolete elements that step 7 refuses, 15 in two of the known limits of Part
  C, and one that only parse5 shows.
- **Likely edits:** 83 small edits to a copy of the site at 601084a, 29 to the stylesheet and 54 to the layouts.

The prototypes and their logs are not in the repository; step 0 keeps the prototypes and the scripts on the fork.
