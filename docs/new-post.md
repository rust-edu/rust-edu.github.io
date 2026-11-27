# Write a news post

For anyone who writes a post for the site's blog, <https://rust-edu.org/news/>. To change another page, see
[editing.md](editing.md); to run, check or deploy the site, see the [README](../README.md).

A post is one Markdown file. Its filename gives it a URL, and the news list and feeds pick it up automatically.
You can leave `urls.yaml` alone when adding a post. You need a GitHub account, and nothing installed.

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

- use lowercase ASCII letters (`a`–`z`) and digits (`0`–`9`) joined by single `-`, and end it with `.md`:
  `rust-week-2027.md`;
- keep it once the post is published, since feed readers know a post by its address.

## 2. Fill in the header

The file starts with a header between two lines of `---`:

| Line | What to write |
|---|---|
| `title:` | The title of the post. Put it in double quotes when it contains `: ` or ` #`, ends with `:`, or starts with punctuation: `title: "Campus Tour: Rust Africa"`. |
| `date:` | The day of the post, as year, month and day: `2027-03-01`. The news list is sorted by it, newest first. |
| `authors:` | Who wrote it: one name per line, each after two spaces and `- ` (put a name in double quotes if it has `: ` or ` #`, or starts with punctuation). |
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
