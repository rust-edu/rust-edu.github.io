# Research 0006 — Atom and RSS Feeds of the News
**Date:** 2026-11-09, updated 2026-11-10 · **Status:** decided; merged into `main` of the fork
kenesparta/rust-edu.github.io as its pull requests 38 to 47, and so part of upstream PR #14

rust-edu.org has never had a feed. Upstream issue #6 (2022) asks for "a way to follow progress", and on upstream PR #14
the maintainer wrote that "the Atom feed is going to need to happen fairly quickly". This note checks what exists today,
settles the shape of two feeds of the news, one in Atom and one in RSS, maps them onto the generator, and reports a
prototype that was built, tested and shown in a draft pull request on the fork, kenesparta/rust-edu.github.io#36. On
2026-11-10 its open points were decided (§5), and the prototype became ten pull requests on the fork, with three more
fixes, which merged into the fork's `main` the same day (§7.2).

**The short version.** Two feeds of the same posts: Atom 1.0 at `/atom.xml` and RSS 2.0 at `/rss.xml`, written by
`edu build` next to `sitemap.xml`. They list every live post of a section with `sort_by: date`, which today means the
three news posts, newest first. Each post comes with its whole text, and every link in it is made absolute. Each post
keeps its permalink at `site.yaml`'s `base_url` as its id in Atom and its guid in RSS, so a deploy at another address
does not show old posts as new. Times come from the posts' dates, never from the clock, so CI's two builds stay
identical. Every page with the site's layout names both feeds for feed readers, Atom first, and `/news/` links to both.

Two gaps close with the feeds. Unicode noncharacters pass both parsers, and two of them, U+FFFE and U+FFFF, are not
allowed in XML. And the links of a post's text go unchecked when its layout leaves the text out. A third bug, found on
the way, stops any pull request that has a draft news post. The prototype is six commits on the fork's branch
`prototype/feeds`, on top of the fork's `main`, and each passes `make check`. Both feeds are valid on the W3C Feed
Validation Service, with 0 errors, and FreshRSS found them by itself from `/news/`. The work landed as ten stacked
pull requests on the fork, of 7 to 512 changed lines, which joined upstream PR #14 as they merged (§7.2). Deployed on
the fork's Pages site, both feeds are valid by address too, with no warning (§7.3).

## 1. What Exists Today

| Check (2026-11-09) | Result |
|---|---|
| `config.toml` of upstream `main` (2ca5e7c) and every earlier version (`git log -G`) | Never sets Zola's `generate_feed`, nor `generate_feeds`, its newer name; Zola writes no feed by default |
| Upstream `gh-pages` (d63ba44), the deployed site | 29 files; `sitemap.xml` is the only XML file |
| `/atom.xml`, `/rss.xml`, `/feed.xml`, `/index.xml`, `/news/atom.xml`, `/feed/` on rust-edu.org | All 404 |
| Wayback Machine: the 138 addresses it has archived on rust-edu.org and its subdomains, the first on 2022-07-11 | No feed: its ten captures of five feed paths on rust-edu.org (2026-03-26 and 2026-05-09) are all 404 |
| `<head>` of the live home page | No `<link rel="alternate">` |

So the switch to the generator removes no feed, and no feed reader can have received a post from any address yet. What
the maintainer follows today, perhaps one of GitHub's Atom feeds of the repository, is not known; the plan is to ask.
The addresses chosen now are permanent: GitHub Pages has no per-path redirects, and a `redirect_to` page is HTML, which
feed readers do not follow.

## 2. What Was Asked Before

- **Issue #6** (2022-09-10, by an account since deleted) asks for a way to follow the project with less detail than
  watching the whole repository: GitHub releases' Atom feed, a blog built by a static site generator, or a fediverse
  account. The maintainer answered the same day: "I've been thinking about setting up a shared SSG blog for Rust-Edu
  with an RSS and/or Atom feed."
- **The August spec** (US-0003 and FR-0004, carried over by research/0004:22): an Atom (RFC 4287) news feed at a
  stable URL, autodiscovery on all pages, a Subscribe link, the author "Rust-Edu" when a post has none, no drafts, and
  0 errors on the W3C Feed Validation Service.
- **research/0002:121-122**: `/atom.xml`, strings through the generator's one escape function, and the feed's
  `updated` equal to the newest post's date, never the build time. Plan 0002:164 makes the feed the first follow-up,
  and PR #14's description said that it was not in that PR.
- **Both formats.** After the maintainer's comment on PR #14, its author decided to offer an RSS 2.0 feed beside the
  Atom one.

## 3. The Feeds

### 3.1 What Both Feeds Share

| Topic | Choice | Why |
|---|---|---|
| Formats | Atom 1.0 (RFC 4287) and RSS 2.0, with the same posts in the same order | Atom is what the maintainer asked for, and it has an RFC; RSS 2.0 is the other common format. FreshRSS and Python's feedparser, both run on the prototype, read both feeds; by their source, so do Miniflux and NetNewsWire. Someone who subscribes to both sees each post twice. |
| Addresses | `/atom.xml` and `/rss.xml`, both listed in `urls.txt` | `atom.xml` is Zola's default name, and the address that research/0002, research/0004 and Plan 0002 name; `rss.xml` is the name of Zola's built-in RSS template. Miniflux tries `atom.xml` first when a page names no feed. GitHub Pages serves `.xml` as `application/xml` (checked on `/sitemap.xml`). |
| Posts | Every live page that a section with `sort_by: date` and no `redirect_to` lists, newest first, with no limit | No new header key: `render::listing` already picks and orders what `/news/` shows. Today there are 3 posts; the feeds are 19,387 bytes (Atom) and 19,332 bytes (RSS). |
| Text | The whole body as HTML, escaped once: Atom's `<content type="html">`, RSS's `<description>` | Posts are short, and the maintainer follows sites in a feed reader. Atom's `type="xhtml"` needs well-formed XHTML, and a summary would only repeat the text. The RSS Profile puts the full content in `description` rather than `content:encoded` "because it has the widest support". |
| Links in the text | Every `href` and `src` made absolute: `/x` becomes `{base}/x`, and `#id` becomes `{base}{post}#id`; other addresses stay as written | Readers resolve relative links differently: Miniflux ignores `xml:base`, NetNewsWire honours it only since 7.1.3 (August 2026), and feedparser resolves `#id` against the feed's own address. The validator does not check URLs inside Atom content, so a test must. |
| Ids | Permalinks at `site.yaml`'s `base_url`, such as `https://rust-edu.org/news/campus-tour/`: Atom's `<id>` and RSS's `<guid>` are the same string | RFC 4287 §4.2.6: an id "MUST NOT change" when a document is relocated, and readers compare ids character by character (Miniflux keys each entry by a hash of its id or guid). The deploy builds with the `--base-url` that `configure-pages` returns, so ids made from it would change with that address. Since every id starts with `base_url`, that setting must not change either: a move to a new domain would need a fixed base for the ids. |
| Times | 00:00 UTC of the post's date: `2026-04-26T00:00:00Z` in Atom, `Sun, 26 Apr 2026 00:00:00 +0000` in RSS | Atom needs a full date-time (RFC 4287 §3.3): a bare `2026-04-26` is an error in the validator. RSS takes the dates of RFC 822 §5, as §3.3 below explains. The hour was decision 1 (§5). |
| The feed's own date | The newest post's time: Atom's `<updated>`, RSS's `<lastBuildDate>` | Never the build time: CI builds the site twice and compares the two (`Dockerfile:28-29`). An edit to an older post does not change it. |
| Name | `site.yaml`'s `title` and `description`, and the language `en`, as the pages have | Readers show the title as the subscription's name. RSS requires a description of the channel, so a site with posts now needs both keys: two new errors, `NoSiteTitle` and `NoSiteDescription`. A blank one is an error too (§6.5). |
| Discovery | One `<link rel="alternate">` per feed in `layouts/layout.html`, Atom first, titled "Rust Edu (Atom)" and "Rust Edu (RSS)"; and a visible line on `/news/`, "Follow the news in a feed reader: Atom or RSS" | WHATWG HTML: readers find a feed from any page through this element. By the RSS Autodiscovery spec, when a page names several feeds, "the title enables users to differentiate between the feeds". Browsers no longer show feeds (Firefox since version 64), so people need a link they can see. Naming both in `<head>` was decision 3 (§5). |
| Drafts | Never in the feeds | The live build drops drafts, and the folder of each draft `_index.md`, before it renders anything. After the fix in §6, no list shows a page with `draft: true`. A post in the folder of a draft `_index.md` can still be listed in a preview, so the feeds also leave out every post that is not live. A test then shows that a preview's feeds list what the live feeds list. |
| Escaping | `edu_template::escape` for every text and for the HTML | It turns `& < > " '` into entities: all that XML text and attributes need. |

### 3.2 Atom

| Topic | Choice | Why |
|---|---|---|
| The feed | `title`, `subtitle`, a `self` and an `alternate` link, the id `https://rust-edu.org/atom.xml`, `updated` and `author`, with `xml:lang="en"` on `<feed>` | RFC 4287 §4.1.1. |
| Authors | One `<author>` per name in `authors`; the feed's own `<author>` is `site.yaml`'s title | RFC 4287 §4.1.1: a feed needs an author unless every entry has one; by §4.2.1, an entry without one takes the feed's. Two authors are two elements, not the byline's "A and B". The title is "Rust Edu", not the August spec's "Rust-Edu"; changing it changes every page's `<title>` too. |
| Left out | `<generator>`, `<summary>`, `xml:base`, an XSLT stylesheet, `fh:complete` | Optional, or harmful: every link is already absolute; Chrome removes XSLT from version 158 (2026-11-17); and `fh:complete` (RFC 5005) tells readers to drop every entry that is not in the file. |

### 3.3 RSS 2.0

| Topic | Choice | Why |
|---|---|---|
| The channel | `title`, `link`, `description`, `language`, an `atom:link` with `rel="self"`, and `lastBuildDate`, then the items | RSS 2.0 requires the first three. The RSS Profile says that "a feed should contain an atom:link used for this purpose", and the validator warns `MissingAtomSelfLink` without it. By the Profile, "all item elements should appear after all of the other elements in a channel"; the validator warns `MisplacedItem` otherwise. |
| An item | `title`, `link`, `guid`, `pubDate`, one `dc:creator` per author, and `description` | The values of the Atom entry. `link` comes before `guid`. With `isPermaLink="false"` the order does not matter to feedparser; without the attribute, or with `"true"`, feedparser marks the guid as the item's link unless a `link` came first. |
| The guid | `isPermaLink="false"` | Without the attribute a guid is a permalink, and the RSS Profile says that it then "must be the permanent URL of the web page associated with the item". But a build for another address, such as the fork's deploy, links each post there. With `"false"`, "the guid may employ any syntax": right in every build, and the validator accepts it. Zola leaves the attribute out. No reader checked takes the attribute into an item's identity, so it can change later without duplicates. |
| Dates | `Sun, 26 Apr 2026 00:00:00 +0000`: the day of the week, a two-digit day, a four-digit year, and `+0000` | RSS 2.0 takes the dates of RFC 822, with a year of "two characters or four characters (four preferred)". RFC 822 §5.2: the day of the week "must be the day implied by the date", and the validator calls a wrong one an error (`IncorrectDOW`). RFC 5322 §3.3: 'The form "+0000" SHOULD be used to indicate a time zone at Universal Time', and `GMT` is obsolete syntax there. `Date::parse` refuses a year before 1900 (§6.4), and computes the day of the week by Tomohiko Sakamoto's method; a test walks every year from 1900 to 9999. |
| Authors | `dc:creator`, one per name | RSS's own `author` is "the email address of the author of the item", and the validator calls `<author>Bart Massey</author>` an error (`InvalidContact`). Dublin Core's creator is a name, and the RSS Profile says that "an item may contain more than one dc:creator element to credit multiple authors". |
| Left out | `content:encoded`, the channel's `pubDate`, `ttl`, `generator`, `image`, `managingEditor`, `webMaster`, `copyright`, `docs` | `content:encoded` would repeat `description`. A `ttl` would only delay new posts: Miniflux takes it as the shortest time between two checks. `image` allows only GIF, JPEG or PNG, and the logos are SVG. `managingEditor` and `webMaster` need an email address. |

The prototype's two feeds for rust-edu.org, cut after the authors of the first post:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom" xml:lang="en">
    <title>Rust Edu</title>
    <subtitle>Rust education in academia (and beyond)</subtitle>
    <link rel="self" type="application/atom+xml" href="https://rust-edu.org/atom.xml"/>
    <link rel="alternate" type="text/html" href="https://rust-edu.org/"/>
    <id>https://rust-edu.org/atom.xml</id>
    <updated>2026-04-26T00:00:00Z</updated>
    <author><name>Rust Edu</name></author>
    <entry>
        <title>Announcing the 2026 Rust-Edu Refresh and CFP</title>
        <link rel="alternate" type="text/html" href="https://rust-edu.org/news/call-for-participation/"/>
        <id>https://rust-edu.org/news/call-for-participation/</id>
        <published>2026-04-26T00:00:00Z</published>
        <updated>2026-04-26T00:00:00Z</updated>
        <author><name>Mordecai Etukudo</name></author>
        <author><name>Bart Massey</name></author>
```

```xml
<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom" xmlns:dc="http://purl.org/dc/elements/1.1/">
    <channel>
        <title>Rust Edu</title>
        <link>https://rust-edu.org/</link>
        <description>Rust education in academia (and beyond)</description>
        <language>en</language>
        <atom:link rel="self" type="application/rss+xml" href="https://rust-edu.org/rss.xml"/>
        <lastBuildDate>Sun, 26 Apr 2026 00:00:00 +0000</lastBuildDate>
        <item>
            <title>Announcing the 2026 Rust-Edu Refresh and CFP</title>
            <link>https://rust-edu.org/news/call-for-participation/</link>
            <guid isPermaLink="false">https://rust-edu.org/news/call-for-participation/</guid>
            <pubDate>Sun, 26 Apr 2026 00:00:00 +0000</pubDate>
            <dc:creator>Mordecai Etukudo</dc:creator>
            <dc:creator>Bart Massey</dc:creator>
```

## 4. How It Fits the Generator

The line numbers are those of `main` before the feeds (38c1acc, and b7c131d, which changes no code), for the code that
already exists there.

- **The posts.** A `Page` (`crates/site/src/page.rs:93`) has everything a post needs: its URL, title, date, authors,
  and its body as HTML without the layout (`page.rs:134`). `build::plan` (`build.rs:126`) never sees it. It gets only
  the rendered pages through `Checked` (`check.rs:90`), and the list of `Page`s ends inside `check::pages`
  (`check.rs:115`). So a new `feed::posts` takes the posts there, from `render::listing` (`render.rs:240`), which
  already gives what `/news/` shows, newest first, and keeps the live ones. `Checked` carries them to `build::plan`.
- **One enum for the two feeds.** `feed::Format` has two cases, `Atom` and `Rss`. It gives each feed's file name, the
  form of its times and its writer, and `Format::ALL` lists both, Atom first. `build::plan` adds each feed as one more
  `Output::Made` (`build.rs:137`), and the check treats both files alike. Every match names each case.
- **The writers.** `crates/site/src/feed.rs` writes the XML by hand, as `sitemap()` does (`build.rs:161`). Every text
  goes through `edu_template::escape` (`crates/template/src/lib.rs:213`). A file of the site named `atom.xml` or
  `rss.xml` is still refused, for now because `.xml` is not a type that the site publishes.
- **Dates.** `Date` keeps its RFC 822 form beside the two forms it has, computed in `Date::parse` from the numbers
  that it already reads, so nothing reads a date twice.
- **Absolute links.** A `Link` enum names what a link goes to: another site or an email address, an id of the same
  page, a page or file of the site, or anything else, which the check refuses. The link check and the rebase of a
  preview's links (`build.rs:183`) match on it, and `links::prefix` is the loop over `href` and `src` that `rebase()`
  had (`links::links`, `links.rs:201`). The feeds add the base URL before a `/`, and the post's own address before a
  `#`.
- **The check.** `/atom.xml` and `/rss.xml` become link targets when there is a post, so the `<link>`s in
  `layout.html` and the lines in `urls.txt` pass `edu check`. The check also follows the links of each post's whole
  text (§6.2).
- **A test of this repository's own site.** It builds the site with its drafts for another address, and checks that
  the feed's id is `https://rust-edu.org/atom.xml`, and that the id and the guid of each post are the address of its
  link, at `https://rust-edu.org`. It works the ids out from the feed instead of listing the posts, so a post removed,
  or renamed with a redirect, still passes, while a change of `base_url` fails it. Since that test reads the
  pages, the `Dockerfile`, `make check` and the README now run `edu check` before the tests, so a mistake in a page
  still shows first as the check's message, with its file, line and column. The name of the step in `ci.yml` follows
  the new order.
- **No change** to `edu serve`, which serves every made file and already answers `.xml` as `application/xml`, as Pages
  does (`serve.rs:221`), nor to `deploy.yml`.
- **Docs.** `docs/editing.md` gets a section on the news feeds: what they hold, that `site.yaml`'s title and
  description name them, and that a post's file name must not change once it is published, because readers know a
  post by its address. The comment in `site.yaml` says that `base_url` starts every id and guid of the feeds, so it
  must not change. The README names both feeds.

## 5. Decisions (Default First)

The author of PR #14 decided each of these on 2026-11-10, as its default, and the pull requests of §7.2 follow them.
Whether the generator stays at all is the maintainers' decision.

1. *Decided: 00:00 UTC.* **The hour of each post: 00:00 UTC**, as Zola writes it, or 10:00 UTC. A post has a date but no
   time. RFC 3339 §5.2 suggests that readers show times in their user's time zone; no reader was tested. If they do, at
   00:00 UTC every place west of UTC sees each post on the day before: a post of 2026-04-26 reads as 25 April, 5 pm, in
   Portland. At 10:00 UTC the day stays the same from UTC−10 (Hawaii) to UTC+13 (New Zealand in summer). Zola (§8)
   writes 00:00 too, so going back to Zola would change nothing that readers see; with 10:00, going back changes the
   time of every post, unless Zola gets its own feed templates. Choose before the first publish: a later change changes
   the time of every post at once, in both feeds.
2. *Decided: `base_url`.* **Ids at `site.yaml`'s `base_url`**, or at the build's own address, as Zola does (a few lines
   less). With the second, the ids follow whatever address `configure-pages` gives the deploy. Any change of that
   address then gives every post a new id, and readers show all of them again as new. In RSS, the first goes with
   `isPermaLink="false"` (§3.3).
3. *Decided: both, Atom first.* **Both feeds named in every page's `<head>`, Atom first**, or only the Atom feed there,
   with RSS linked from `/news/`. WHATWG HTML allows several such links, and when a user agent has the concept of a
   default feed, "the first such element (in tree order) should be used as the default". The RSS Autodiscovery spec also
   allows several links, the first being "the site's main feed", but it advises one link per page, and against this case
   in particular: "Publishers who offer the same feed content in several syndication formats should not use
   autodiscovery links for all of them". Given `/news/`, FreshRSS took the first, the Atom feed. By their source, not
   run: Miniflux lists both and asks, and NetNewsWire gives both the same score, so which one it takes is arbitrary. One
   link would be one line less, and a predictable default.
4. *Decided: into PR #14, through pull requests on the fork that join it one at a time.* **When it lands.** First, the
   maintainers decide whether the generator stays: on Zola the Atom feed is one line (§8). If it stays, either the
   feeds go into PR #14 before it merges, or they come right after the merge, as PR #14's description said of the
   Atom feed. The first gives the new site its feeds from its first day, but makes PR #14 larger while it is under
   review, and each new version of it waits again for a maintainer to approve its CI run. The second lets PR #14 merge
   sooner, and gives the feeds' decisions small pull requests of their own. The pull requests of §7.2 keep both
   benefits in part: each is small and reviewed on its own on the fork.
5. *Decided: fixed.* **The known limits of the prototype** (§7.1): a `date` before 1900, an empty `title` or author
   name, and the style of the line on `/news/`. Each is fixed before the feeds land (§6.4, §6.5 and §7.2).

## 6. Gaps Found on the Way

The first three were checked with a build of `main` at dede921; the code that they touch is the same at 38c1acc and at
b7c131d. They are described as they were then, before their fixes. The last two are known limits of the prototype
(§7.1). All five are fixed in the pull requests of §7.2.

1. **Noncharacters reach the HTML.** XML 1.0 allows neither U+FFFE nor U+FFFF. Both parsers refuse control characters
   (`crates/markdown/src/lib.rs:97`, `crates/yaml/src/lib.rs:124`), but noncharacters are not control characters. So
   they pass from a title, an author, `site.yaml` or a post's text into the pages. One U+FFFE or U+FFFF would make a
   feed malformed. The other 64 noncharacters are allowed in XML but discouraged, and HTML's parser reports all 66
   ("noncharacter-in-input-stream"). So the fix refuses all 66, with `file:line:col`, in both parsers. Today's content
   has none.
2. **The links of a post's text can go unchecked.** `links::check` (`links.rs:44`) reads each rendered page. A post
   whose layout leaves out `page.content` keeps links in its text that no check sees, and the feeds would publish
   them. A test site with `[rel](a/)` passed `edu check` with such a layout, and failed with the normal one. The fix,
   in its own commit before the feeds, checks each post's text as well.
3. **A draft news post stops every check.** With one `draft: true` post, `edu check` and `edu build --drafts` both
   fail with "`/news/draft-post/` goes to a draft, which the live site leaves out". The `/news/` list links every post,
   drafts included. But the live `/news/` page may not link to a draft (`LinkError::ToDraft`, `links.rs:140`).
   `docs/editing.md:49` promises that "pull request builds still include it". CI runs `edu check`, so a pull request
   with a draft news post fails today. Once merged, such a post would stop every deploy too, since the deploy runs the
   same check (`Dockerfile:27`, `Dockerfile:34`). The tests missed it, because no test site has a news list next to a
   draft.

   The fix is one line at the end of `render::listing` (`render.rs:240`): a list leaves out the pages with `draft:
   true`. A live page is then the same in every build. A preview still builds the draft's own page at its address, and
   a link written by hand from a live page to a draft is still an error. With its tests and two sentences of
   `docs/editing.md` (lines 49 and 130, the second on the local preview), it is 73 changed lines.
4. **A date before 1900.** RSS takes the dates of RFC 822, and RFC 5322, its current successor, has no year before
   1900. Some parsers misread an early year: Python's `email.utils` reads `0050` as 2050, and feedparser gives no date
   for the year 0; by their source, NetNewsWire also reads `0050` as 2050, and the parser in FreshRSS reads `0500` as
   2400. So `Date::parse` now refuses a year before 1900, with its own error, `ValueError::EarlyDate`, at the date's
   line and column, before it checks the day. The day of the week then never needs a year before 1899, and the
   400-year shift of the prototype is gone. A test walks every year from 1900 to 9999, and the test site
   `unrenderable` gets a page of 1899. Today's content starts in 2022.
5. **An empty title or author name.** A `title` of `""` or of spaces alone passed the check, and so did such a name in
   `authors`: the page got an empty heading, the post a byline with an empty name, and the feeds an empty element. Now
   each is an error at its line and column, with a hint, and so is a blank `title` or `description` in `site.yaml`,
   with four new variants of `ValueError`. Blank means empty, or only Unicode White_Space, as Rust's `str::trim` sees
   it; a value made only of invisible characters that are not White_Space, such as U+200B, still passes.

## 7. The Prototype and the Pull Requests

### 7.1 The Prototype

The prototype is six commits on the fork's branch `prototype/feeds`, on top of the fork's `main` (38c1acc), and the
draft pull request kenesparta/rust-edu.github.io#36 shows it with screenshots. That pull request was closed on
2026-11-10 in favour of the pull requests of §7.2; the branch stays. It follows `AGENTS.md`: the standard
library only, no `unsafe`, no panics on input, its own error variants, enums matched in full, and tests with each
change.

| # | Commit | Changed lines | What it does |
|---|---|---|---|
| 1 | 548bcd1 fix: leave the drafts out of a list of pages, so a draft news post passes | 73 (+56 −17) | The fix of §6.3 |
| 2 | 3eea05c fix: refuse Unicode's noncharacters in a page and in site.yaml | 112 (+104 −8) | `edu-yaml` and `edu-markdown` refuse the 66 noncharacters with `file:line:col` (§6.1) |
| 3 | c535aca refactor: name what a link goes to, and share the loop that rebases links | 112 (+83 −29) | The `Link` enum and `links::prefix` (§4); the built site does not change |
| 4 | c6a5ba1 feat: check the links in the whole text of each news post, for the feed | 176 (+162 −14) | `feed::posts` and the check of each post's text (§6.2); the built site does not change |
| 5 | 8c389a1 feat: write an Atom feed of the news at /atom.xml | 486 (+447 −39) | The Atom writer, its `<head>` link and the line on `/news/`, `NoSiteTitle`, `/atom.xml` in `urls.txt`, the test site `feed`, the test of this site's ids, the order of the checks, and the docs |
| 6 | 06e5aaf feat: write an RSS 2.0 feed of the news at /rss.xml | 441 (+340 −101) | `Format`, the RSS writer, the RFC 822 form of `Date`, `NoSiteDescription`, the second `<head>` link, `/rss.xml` in `urls.txt`, and the docs |

Together they change 1,194 lines (+1,089 −105) in 38 files, and the generator grows from 7,553 to 8,448 lines of Rust:
446 of code and 449 of tests. Plan 0002's limit is "near 500 changed lines (600 at most), not counting content moves
and deletions". Each commit fits and the whole does not, so the work lands as separate pull requests (§7.2). The
first stands alone: it fixes `main` as it is today.

| Check | Result |
|---|---|
| `make check` (fmt, clippy with `-D warnings`, the tests and `edu check`; from commit 5 on, `edu check` runs before the tests) at each of the six commits | Passes: 129 tests after the first commit, 140 after the last |
| CI of the draft pull request: the checks in Docker, two builds compared, and the preview | Passes |
| Two builds, compared with `diff -r` | Byte-identical: 11 pages and 24 files |
| `--base-url https://kenesparta.github.io/rust-edu.github.io` | Every link that the generator writes into the feeds starts with that address; the ids and guids stay at `https://rust-edu.org` |
| `xmllint --noout` | Both feeds are well-formed |
| W3C Feed Validation Service, by direct input, for both builds | "This is a valid Atom 1.0 feed" and "This is a valid RSS feed": 0 errors, and 1 warning each, `SelfDoesntMatchLocation`, which pasted input always gets because it has no address |
| Scripts that check the rules of RFC 4287 and RSS 2.0, and the site's own: order, drafts, absolute links, the day of the week of each date, the same posts in both feeds | Pass; the RSS script fails on six copies broken on purpose |
| Python's feedparser 6.0.14 | Reads both with no error: 3 posts each, with the same titles, links, ids, dates, authors and text |
| A copy of the site with a draft post, and a draft section with a dated post | The feeds are byte-identical with and without `--drafts` |
| `edu serve` | `/atom.xml` and `/rss.xml` answer 200 with `Content-Type: application/xml`, and the bytes of `edu build` |
| FreshRSS 1.30.4 in Docker, with the site built for the fork's address and served locally | Given `/news/`, it found the feeds and took the Atom feed; given `/rss.xml`, it showed the same three posts and dates; a post shows its author and its photos |
| Ten changes to the code made on purpose: no 400-year shift, a wrong month in the weekday table, guids at the build's address, no `isPermaLink`, only the Atom feed written, no description required, and four texts left unescaped | Each fails a test |
| A code review that tried to break the feeds | No high-severity finding. Its three medium findings were fixed: the order of the checks, the size of the Atom commit (now commits 4 and 5), and no test of escaped author names and descriptions |

**Known limits of the prototype,** from the same review. The first three are fixed in the pull requests of §7.2:

- `Date::parse` accepted a year before 1900, which some parsers misread (§6.4).
- An empty `title` or author name passed the check, and the feeds then carried an empty element (§6.5).
- The line on `/news/` reused the style of a post's byline, so a later change of the byline would change it too. It
  now has a class of its own, `news-feeds`, which joins the selector lists of the byline in `style.css`, so both still
  look the same and a later change can part them.
- The README links to `https://rust-edu.org/atom.xml` and `/rss.xml`, which exist only once the feeds reach
  rust-edu.org.

### 7.2 The Pull Requests

The work is ten pull requests on the fork, stacked: each was based on the one before. They merged into the fork's
`main` in order on 2026-11-10, each with a merge commit, and joined upstream PR #14 as they merged. The numbers are
those of kenesparta/rust-edu.github.io, and the changed lines are those of each pull request against the one below it.

| Fork PR | Branch | Changed lines | What it does |
|---|---|---|---|
| 38 | `fix/draft-news-list` | 73 | Commit 1 of the prototype: the fix of §6.3 |
| 39 | `ci/faster-checks` | 7 | CI: no export of an image that no step uses, a log that streams, and `ci.yml` on pull requests only |
| 40 | `docs/ai-acknowledgement` | 11 | The README's AI acknowledgement, and the line of `AGENTS.md` that asks each pull request to name its AI tools |
| 41 | `docs/theme-script` | 7 | Plan 0002 records the two short scripts of the theme button |
| 42 | `fix/noncharacters` | 112 | Commit 2: the fix of §6.1 |
| 43 | `fix/empty-title-and-author` | 154 | New: the fix of §6.5 |
| 44 | `refactor/link-kinds` | 112 | Commit 3: the `Link` enum and `links::prefix` |
| 45 | `feat/post-text-links` | 176 | Commit 4: the fix of §6.2 |
| 46 | `feat/atom-feed` | 512 | Commit 5, with the line on `/news/` in a class of its own, the settings of `site.yaml` read once, the test of this site's ids that §4 describes, and the comment of the `Dockerfile` on the new order of the checks |
| 47 | `feat/rss-feed` | 488 | Commit 6, with no date before 1900 (§6.4), and the same test extended to the guids |

Together they change 1,436 lines (+1,292 −144) in 43 files, and the generator grows from 7,553 to 8,583 lines of
Rust, tests included. The largest, 46, is under Plan 0002's limit of 600. Every new error has its own variant, and
every match on the new enums names each case. The feeds have the same bytes as the prototype's, for rust-edu.org and
for the fork's address. Pull request 49, merged after them, added this note.

### 7.3 Checked on the Pull Requests

| Check (2026-11-10) | Result |
|---|---|
| `make check` at each of the 11 commits of the stack | Passes: 129 tests after 38, 142 after 47, with "checked 11 pages with 5 layouts" each time |
| `make ci` on the top of the stack | Passes, with the two builds the same |
| CI of 38 to 47 | Passes for each, with its preview attached |
| The feeds of the top of the stack, built for rust-edu.org, for the fork's address and for `https://example.github.io/rust-edu`, against the prototype's | The same bytes (`cmp`) |
| The fork's Pages site, deployed by hand from the top of the stack, before the merge | Served `/atom.xml` and `/rss.xml` as `application/xml`, with the bytes of a local build for the fork's address; the home page named both feeds, Atom first |
| W3C Feed Validation Service, given the address of each feed on that site | "This is a valid Atom 1.0 feed." and "This is a valid RSS feed.": 0 errors, 0 warnings, and no recommendation. With an address, the validator can match each feed's `self` link to it, so the `SelfDoesntMatchLocation` of the direct input is gone |
| The fork's `main` after each of the ten merges | Holds the files of the pull request just merged, so after the last one those of the top of the stack |
| The fork's Pages site, deployed by `deploy.yml` from the fork's `main` after the merge | Serves the 35 files of a local build of `main` for the fork's address, byte for byte, `/atom.xml` and `/rss.xml` among them, both as `application/xml` |

**Done when:**

- The CI of each pull request passes, with its two builds the same (done for 38 to 47).
- The W3C Feed Validation Service, given the addresses of the deployed feeds, says that both are valid, with no
  warnings (done on the fork's Pages site; to repeat on rust-edu.org).
- A feed reader given only `https://rust-edu.org/`, once PR #14 is merged and deployed, finds a feed, and shows the
  three posts with their dates, authors, images and working links. FreshRSS did part of this, with the site built for
  the fork's address and served locally: given `/news/`, not the home page, it found the feeds and took the Atom feed,
  and it showed the three posts with their dates, and the campus tour post with its author and its photos. No link in
  a post was followed.
- `urls.txt` lists both feeds, and a test checks that the ids and guids stay at `https://rust-edu.org` under another
  address (done).

## 8. With Zola, for Comparison

On Zola the Atom feed is one line. Adding `generate_feeds = true` to the last Zola `config.toml` in this repository's
history makes Zola 0.23.6 write `/atom.xml` from today's content: 19,858 bytes, with the three posts, permalinks as
ids, midnight UTC as the time, and the full text. Zola can write `rss.xml` too, from its built-in template, with
`feed_filenames = ["atom.xml", "rss.xml"]`; that was not tested here. But the template would not be valid for this
site: it puts the name of a post's first author in RSS's `<author>`, which the validator calls an error, and leaves
out the others. A valid RSS feed would need a template of its own.

Here the two feeds cost the six pull requests after the docs, 42 to 47 of §7.2, with the fixes of §6: 1,346 changed
lines as one diff (+1,222 −124), tests and docs included, or 1,554 counted per pull request, since later pull requests
change lines of earlier ones. The prototype's five commits after the draft fix were 1,129 lines as one diff.
research/0002:144 had guessed 200 for one feed. That is the price of the generator's own checks, and of owning it.

## 9. Not Verified

- Validation by address on rust-edu.org itself, which is only possible once the feeds are deployed there. On the
  fork's Pages site, which GitHub Pages serves in the same way, both feeds passed by address with no warning (§7.3).
- How each reader shows the times, and so whether 00:00 UTC really shows posts a day early in the Americas.
- Readers other than FreshRSS and feedparser. Miniflux and NetNewsWire were read in their source but not run;
  Thunderbird, Feedly, Inoreader and browser extensions were not checked. For Feedly and Inoreader, no first-party
  documentation on ids, `xml:base` or relative links was found.
- The look of `/news/` on a phone: headless Chrome has a minimum window width, so the 390-pixel screenshot shows the
  page in a frame of that width.
- Which reader and address the maintainer uses: the plan is to ask, and to test the feeds there.

## Sources

- [RFC 4287, the Atom Syndication Format](https://www.rfc-editor.org/rfc/rfc4287),
  [RFC 3339, date and time on the Internet](https://www.rfc-editor.org/rfc/rfc3339), and
  [RFC 5005, feed paging and archiving](https://www.rfc-editor.org/rfc/rfc5005)
- [The RSS 2.0 specification](https://www.rssboard.org/rss-specification),
  [the RSS Best Practices Profile](https://www.rssboard.org/rss-profile) and
  [RSS Autodiscovery](https://www.rssboard.org/rss-autodiscovery), from the RSS Advisory Board
- [RFC 822](https://www.rfc-editor.org/rfc/rfc822) and [RFC 5322](https://www.rfc-editor.org/rfc/rfc5322), for the
  dates of RSS, and [Dublin Core's terms](https://www.dublincore.org/specifications/dublin-core/dcmi-terms/), for
  `dc:creator`
- [W3C Feed Validation Service](https://validator.w3.org/feed/) and its
  [documentation](https://validator.w3.org/feed/docs/)
- [WHATWG HTML, `rel="alternate"`](https://html.spec.whatwg.org/multipage/links.html#rel-alternate),
  [WHATWG HTML, the noncharacter parse error](https://html.spec.whatwg.org/multipage/parsing.html#parse-error-noncharacter-in-input-stream),
  and [XML 1.0, characters](https://www.w3.org/TR/xml/#charsets)
- [Zola, feeds](https://www.getzola.org/documentation/templates/feeds/),
  [Zola, configuration](https://www.getzola.org/documentation/getting-started/configuration/),
  [Zola's built-in RSS template](https://github.com/getzola/zola/blob/master/components/templates/src/builtins/rss.xml),
  and [jekyll-feed](https://github.com/jekyll/jekyll-feed)
- [feedparser, relative links](https://feedparser.readthedocs.io/en/latest/resolving-relative-links/),
  [Miniflux](https://github.com/miniflux/v2),
  [NetNewsWire's release notes](https://github.com/Ranchero-Software/NetNewsWire/blob/main/Technotes/ReleaseNotes-Mac.markdown),
  [Newsboat](https://github.com/newsboat/newsboat) and [FreshRSS](https://github.com/FreshRSS/FreshRSS)
- [Chrome: removing XSLT](https://developer.chrome.com/docs/web-platform/deprecating-xslt) and
  [Firefox 64's release notes](https://www.mozilla.org/en-US/firefox/64.0/releasenotes/)
- [The Wayback Machine's CDX index of rust-edu.org](https://web.archive.org/cdx/search/cdx?url=rust-edu.org&matchType=domain&collapse=urlkey)
- [Upstream issue #6](https://github.com/rust-edu/rust-edu.github.io/issues/6) and
  [PR #14](https://github.com/rust-edu/rust-edu.github.io/pull/14)
