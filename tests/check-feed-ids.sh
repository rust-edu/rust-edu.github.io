#!/bin/sh
# Build a fork preview without touching the contributor's public/ directory.
set -eu
fork=https://example.github.io/rust-edu
work=$(mktemp -d "${TMPDIR:-/tmp}/edu-feed.XXXXXXXX")
trap 'rm -rf "$work"' EXIT
cp -R content layouts static site.yaml urls.yaml rust-toolchain.toml "$work/"
cd "$work"
"$@" build --drafts --base-url "$fork/"

# Each feed field below occupies one line in the generated XML. A feed reader uses the
# canonical IDs to recognize posts even when a fork serves the links at another address.
sed -n 's|^[[:space:]]*<id>\([^<]*\)</id>$|\1|p' public/atom.xml > atom-ids
sed -n 's|^[[:space:]]*<link rel="alternate" type="text/html" href="\([^"]*\)"/>$|\1|p' public/atom.xml > atom-links
sed -n 's|^[[:space:]]*<guid isPermaLink="false">\([^<]*\)</guid>$|\1|p' public/rss.xml > rss-guids

if [ "$(head -n 1 atom-ids)" != "https://rust-edu.org/atom.xml" ]; then
    echo "atom.xml: the feed ID must stay at https://rust-edu.org/atom.xml" >&2
    exit 1
fi
sed '1d' atom-ids > post-ids
if [ ! -s post-ids ] || grep -qv '^https://rust-edu\.org/.' post-ids; then
    echo "atom.xml: expected post IDs under https://rust-edu.org/" >&2
    exit 1
fi
diff -u post-ids rss-guids
{
    echo "$fork/"
    sed "s|^https://rust-edu\.org/|$fork/|" post-ids
} > expected-links
diff -u expected-links atom-links
