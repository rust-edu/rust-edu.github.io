# Plan 0007 — Images for news posts

**Status** — Proposed for discussion in [issue #41](https://github.com/rust-edu/rust-edu.github.io/issues/41).

An author should be able to find a post's photos beside its text, place each one where it belongs in the story and
prepare a camera original for the web with one command. These are proposed changes to the authoring workflow.

## Keep photos beside the post

For new posts, use the same slug for the Markdown file and its image directory. For a post with two photos the
repository would contain these files.

```text
content/news/robotics-workshop.md
content/news/robotics-workshop/soldering-lab.webp
content/news/robotics-workshop/student-demo.webp
```

Place each image at the point where the reader should see it.

```markdown
![Two students solder a sensor onto their robot](/news/robotics-workshop/soldering-lab.webp)
```

The order of the references in Markdown controls reading order. Descriptive filenames can stay the same when the
author rearranges the story. The directory holds assets only. Adding an `_index.md` there would claim the same
public URL as the post. Artwork reused across posts, such as a logo, belongs in `static/`.

This layout works with the current generator. In a disposable post on revision `0d635ed`, `edu check` accepted the
link, the build copied the WebP byte for byte, and a build under a Pages project prefix added that prefix to the
image URL. Atom and RSS used the absolute image URL. **A draft's images are public in a live build** even when the
page and feed entry are omitted. The build copies assets independently of page visibility. Keep unpublished
originals outside `content/` and `static/`, and add derivatives to the post folder when they can be public.
When deleting a post, review its sibling folder too.

Apply the convention to new photos first. The campus-tour post already publishes four WebPs under `/images/`,
and `urls.yaml` preserves four earlier PNG addresses. Keep all eight files at their existing addresses. A future
move would need an explicit plan for callers of each old file URL, since the current redirect rules accept page
paths. A new photo only needs its file and Markdown reference. [Campus-tour post](../../content/news/campus-tour.md)
and [preserved URLs](../../urls.yaml).

## Prepare a photo

The first version would take one local original and the slug of an existing post. This example shows the proposed
interface.

```sh
tools/post-image --post robotics-workshop --input "/photos/Soldering lab.JPG" --name soldering-lab
```

- Accept PNG, JPG and JPEG, including uppercase extensions. Check that the file decodes as a supported still image
  and that the post and output name are valid slugs. Write to `content/news/<post>/<name>.webp`.
- Apply the camera's EXIF orientation before resizing. Preserve transparency, and remove location and camera
  metadata from the output. The [photo guidelines](../new-post.md) call for 960 pixels wide, 16:9 and at most
  90 KB. Use 960 × 540 and an explicit limit of 90,000 bytes. If the input needs cropping, ask the author to
  supply a 16:9 crop. If it is too small, ask for a larger original. This preserves composition and avoids stretching.
- Measure the encoded file. Try a bounded range of quality settings. If the image still exceeds the limit at the
  agreed quality floor, report the measured size and ask for a different crop or source. Establish the floor with
  the sample photos before releasing the command.
- Leave the original intact. Encode to a temporary file and publish it only after all checks pass. An identical
  result can succeed on a rerun. If a different photo would take an existing name, report the conflict and leave
  the existing file intact. Print the final path, byte count and a Markdown reference for the author to fill in
  with meaningful alt text.

Commit the prepared WebP with the Markdown. Contributors editing text and the site build can use the committed
files without installing an encoder.

## Encoder and format choices

**Recommend WebP and an optional author-side command using ImageMagick with WebP support.** It can read JPEG and
PNG, apply orientation and write quality-controlled WebP. Document a tested tool version and check for the WebP
encoder at startup. The existing campus-tour WebPs give reviewers a baseline for judging quality and size.
[ImageMagick orientation](https://imagemagick.org/command-line-options/#auto-orient),
[ImageMagick WebP options](https://imagemagick.org/webp/) and
[the earlier site photo measurements](Plan-0003—Visual-Refresh-and-Page-Speed.md).

**A Rust command is feasible.** A small author-side Rust program can validate paths and invoke the encoder with
separate arguments using `std::process::Command`. Keep it outside the four-crate generator workspace. If a
standalone binary with bundled codecs becomes important, evaluate a separate tools workspace and lockfile with PNG
and JPEG decoders and a lossy WebP encoder. The `image` crate's WebP encoder in 0.25.10 supports lossless encoding
only. Writing codecs from scratch would also make the project responsible for decoding malformed inputs, encoder
quality and format interoperability. Reusing a maintained encoder meets the authoring need with less upkeep.
[Rust process API](https://doc.rust-lang.org/std/process/struct.Command.html) and
[image 0.25.10 WebP encoder](https://docs.rs/image/0.25.10/image/codecs/webp/struct.WebPEncoder.html#limitations).

**Evaluate AVIF for photos.** Compare it with WebP at 960 × 540 and similar visible quality on the four campus-tour
photos. Record encoded bytes and encoding time, and check that the results display in the local preview, on Pages
and in a feed reader. The generator accepts `.avif`, but the current local preview serves it as
`application/octet-stream`. Include a test for `image/avif` and a preview fix in any adoption PR. Use the results
to decide whether a second format earns its maintenance cost. [AVIF reference tools](https://github.com/AOMediaCodec/libavif#command-line-tool-usage).

## Small PRs if approved

| PR | Scope | How to verify |
|---|---|---|
| 1. Authoring convention | Update the human and AI author guides with the folder layout, draft visibility, alt text and Markdown ordering. | Build a two-photo fixture at the site root and under a project prefix. Verify article and feed order, image URLs and draft behavior. Check that existing campus-tour links still resolve. |
| 2. WebP command | Add the optional command, its usage and tests. | Cover a rotated JPEG, an uppercase extension, transparent PNG, a bad input, cropping, undersized photos, byte-limit failure and filename collisions. Verify format, dimensions, size, metadata removal and safe reruns. Run the site checks with the command absent. |
| 3. AVIF evaluation | Record the comparison and decide whether to offer AVIF. | If adopted, verify preview MIME type and rendering before documenting it for authors. |

The decisions for review are the post-local layout, the initial encoder and crop policy, and whether measured
results warrant supporting AVIF.
