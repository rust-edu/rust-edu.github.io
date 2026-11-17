# Write a news post

The news of rust-edu.org is its blog: <https://rust-edu.org/news/>. A post is one Markdown file, and adding that file
is all it takes: the news list and the feeds pick it up by themselves. You need a GitHub account, and nothing
installed.

This page is for a person who writes a post. It is also complete enough for an AI assistant to write one from it:
see [With an AI assistant](#with-an-ai-assistant).

## A whole post

```markdown
---
title: Rust Week 2027
date: 2027-03-01
authors:
  - Jane Doe
---
Rust Week 2027 takes place in **Lima** from March 1 to 5, with talks for teachers and a workshop for students.

## What to expect

- Talks on teaching Rust in a first course
- A workshop, for which you need only a laptop

Read the [call for participation](/news/call-for-participation/), and write to
[contact@rust-edu.org](mailto:contact@rust-edu.org) with any question.
```

Saved as `content/news/rust-week-2027.md`, this is the page `/news/rust-week-2027/`. The four steps below make one
like it.

## 1. Start the file

**[Start a new post on GitHub](https://github.com/rust-edu/rust-edu.github.io/new/main?filename=content/news/your-post-name.md&value=---%0Atitle%3A%20Your%20title%0Adate%3A%20YYYY-MM-DD%0Aauthors%3A%0A%20%20-%20Your%20Name%0A---%0AThe%20first%20paragraph%20of%20your%20post.%0A%0A%23%23%20A%20heading%0A%0AMore%20of%20your%20post.%0A)** opens GitHub's editor with the header already in the file. Change
`your-post-name.md`, in the box above the text, to the name of your post.

To start it by hand instead, open the folder `content/news/` on GitHub and choose *Add file → Create new file*.

The name of the file becomes the address of the post, so:

- use lowercase letters and digits joined by `-`, and end it with `.md`: `rust-week-2027.md`;
- keep it once the post is published, since feed readers know a post by its address.

## 2. Fill in the header

The file starts with a header between two lines of `---`:

| Line | What to write |
|---|---|
| `title:` | The title of the post. Put it in double quotes when it has a colon in it: `title: "Campus Tour: Rust Africa"`. |
| `date:` | The day of the post, as year, month and day: `2027-03-01`. The news list is sorted by it, newest first. |
| `authors:` | Who wrote it: one name per line, each after two spaces and `- `. |
| `draft: true` | Only while the post is not ready: it keeps the post off the live site. Take the line out to publish. |

Nothing else goes in the header.

## 3. Write the post

Below the header, write in plain Markdown:

| For | Write |
|---|---|
| A heading | `## A heading`, and `### A smaller heading` below it. The title is already the top heading of the page. |
| Bold and italic | `**bold**` and `*italic*` |
| A list | Lines that start with `- `, or with `1. ` for a numbered list |
| A link to another site | `[the text](https://example.org/)`, or the address itself as `<https://example.org/>` |
| A link to a page of this site | `[the text](/news/campus-tour/)`: from the first `/`, and with a `/` at the end |
| An email address | `[contact@rust-edu.org](mailto:contact@rust-edu.org)` |
| A photo | `![What the photo shows](/images/your-photo.webp)` |
| A quote | `> The words of the quote` |
| Code | `` `code` `` inside a line, or lines of code between two lines of three backticks |

Leave an empty line between two paragraphs, and before and after a heading, a list or a photo.

**A photo** goes in `static/images/`, in the same pull request: 960 pixels wide, in the 16:9 shape, saved as WebP of
at most 90 KB.

**What a post cannot have:** HTML, tables, footnotes, `- [ ]` task lists, `~~strikethrough~~`, and a line of `---`
as a rule across the page. An address written without `<…>` or `[the text](…)` is not a link. The site would show
these differently from GitHub's preview, so the check stops at them. [editing.md](editing.md) has the whole list.

## 4. Propose it

1. Look at the *Preview* tab of GitHub's editor, which shows the header as a table.
2. Choose *Commit changes… → Propose changes → Create pull request*. Without write access to the repository,
   GitHub first asks to *Fork this repository*; say yes.
3. Fill in the description of the pull request, which starts from a short template.
4. A check reads the post. When something is wrong, it names the file, the line and the column, and says what to
   write instead:

   ```
   content/news/rust-week-2027.md:3:7: error: a date is a day written as `2027-03-01`
   ```

   Fix that line in the pull request, and the check runs again. On your first pull request here, the check waits
   until a maintainer lets it run.
5. A maintainer reads the post and merges it. A few minutes later it is on the site and in the feeds.

To see the post as the site shows it before you propose it, run `docker compose up` in the repository, with
[Docker](https://docs.docker.com/get-started/get-docker/) installed, and open <http://127.0.0.1:1111/news/>.

## With an AI assistant

An assistant can draft the file from your notes; you are still the author, so read what it wrote and check each
name, date and link. Give it this page and your notes, for example:

```
Read https://raw.githubusercontent.com/rust-edu/rust-edu.github.io/main/docs/new-post.md and follow it exactly to
write a news post for rust-edu.org from my notes. Give me the name of the file and the whole file in one block of
code. Ask me for what is missing; do not make anything up.

My notes: …
```

Then say in the pull request which tool helped and how, under *AI acknowledgement*; [ai.md](ai.md) says why.

### Rules for the assistant

1. **One file.** Write `content/news/<name>.md`, where `<name>` is lowercase letters and digits joined by `-`.
   Change no other file, except to add a photo to `static/images/`.
2. **The header first.** The file starts with a line `---`, then `title`, `date` as `YYYY-MM-DD` and `authors` as
   a list, then a line `---`. `draft: true` is the only other key. Put the title in double quotes when it has `: `
   in it or `:` at its end.
3. **Only the Markdown of step 3.** Start the headings at `##`. Do not write HTML or an HTML comment, a table, a
   footnote, a task list, `~~strikethrough~~`, a horizontal rule, a heading underlined with `===`, code indented by
   four spaces, three `*` or `_` in a row (bold italic is `**_text_**`), a link with a title
   (`[text](url "title")`), a reference link (`[text][1]`), or a character reference other than `&amp;`, `&lt;`,
   `&gt;` and `&quot;`: type the character itself, such as `é` or `—`.
4. **Links that lead somewhere.** A link to a page of the site starts with `/` and ends with `/`, as in
   `/news/campus-tour/`, and that page has to exist; never write a relative link such as `../news/`. A link to
   another site is the full `https://` address. Write a bare address as `<https://example.org/>`.
5. **Only the author's facts.** Do not make up a name, a date, a quote, a number or a link. Ask for the title, the
   date and the authors when the notes do not give them.
6. **Check it.** If you can run commands in the repository, run `make check` and fix what it reports until it
   passes. If you cannot, read the file once more against rules 1 to 4.
7. **Hand it over.** Give the author the path of the file and the whole file, and remind them to say in the pull
   request which tool helped and how.
