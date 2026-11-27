---
name: new-news-post
description: Create a new Rust Edu blog or news entry under content/news/ from an author's notes. Use when drafting a new post for rust-edu.org.
---

# Write a Rust Edu news post

Read the [post specification](../../../docs/ai/new-post.md) for the template, accepted Markdown, and photo rules.
Use the author's title, publication date, byline, and notes. Ask for any missing facts. If they request a draft,
set `draft: true`.

For a file in this checkout, use the author's task branch or start a `content/<slug>` branch from current
`main` as described in the [branch workflow](../../../README.md#branch-workflow). Check that the chosen
`content/news/<slug>.md` path and `/news/<slug>/` URL are available. If the requested slug is taken, suggest
another one to the author. Existing posts and their public addresses must stay intact.

Write the file and run `make check` from the repository root. The generator derives the news listing, public URL
and feed entries from the post. Report the path and check result so the author can review it. When the author
wants text to paste into GitHub or is working in chat without a checkout, return the path and Markdown as
described in the post specification.
