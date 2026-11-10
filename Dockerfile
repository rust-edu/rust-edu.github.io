# The generator's toolchain, the same on laptops and in CI. Check the generator and every page,
# and build the site twice to see that both builds are the same, with:
#   docker build --target check .
# Preview the site, drafts too, at http://127.0.0.1:1111 with `docker compose up`.
# The tag names the Rust version of rust-toolchain.toml (CI fails when they differ); the digest
# pins the image, since tags are re-pushed. Dependabot bumps the tag, the digest and
# rust-toolchain.toml in one pull request.
FROM rust:1.99.1-slim-trixie@sha256:b8e062b1c3a80183df83d9ba187cadd2b5ab84bc78d453dbfeb2fccd5f982247 AS toolchain
# The image installs Rust's minimal profile, which lacks these two.
RUN rustup component add clippy rustfmt
WORKDIR /src

# Format, lint and build, then check the pages and test, with no network: the generator uses the
# standard library only.
FROM toolchain AS check
COPY Cargo.toml Cargo.lock clippy.toml rust-toolchain.toml ./
COPY crates/ crates/
RUN --network=none cargo fmt --all --check
RUN --network=none cargo clippy --workspace --all-targets --locked --offline -- -D warnings
RUN --network=none cargo build --workspace --release --locked --offline
# The settings, the layouts, the static files and then the pages come last, so that editing one
# reuses every layer above.
COPY site.yaml urls.txt ./
COPY layouts/ layouts/
COPY static/ static/
COPY content/ content/
RUN --network=none target/release/edu check
# The tests run once the pages pass: one of them builds this site, for the ids of its feeds.
RUN --network=none cargo test --workspace --locked --offline
RUN --network=none target/release/edu build && mv public /tmp/first \
    && target/release/edu build && diff -r /tmp/first public

# The live site, which deploy.yml publishes, at the address that GitHub Pages gives, or else at the
# base_url of site.yaml:
#   docker build --target site --build-arg BASE_URL=https://example.github.io/rust-edu -o site .
FROM check AS live
ARG BASE_URL
RUN --network=none if [ -n "$BASE_URL" ]; then target/release/edu build --base-url "$BASE_URL"; \
    else target/release/edu build; fi
FROM scratch AS site
COPY --from=live /src/public /

# The site with its drafts, which CI attaches to each pull request:
#   docker build --target preview -o preview .
FROM check AS drafts
RUN --network=none target/release/edu build --drafts
FROM scratch AS preview
COPY --from=drafts /src/public /

# `edu serve` on the files of the site, which compose.yaml mounts at /site, read-only: the server
# builds the site in memory.
FROM toolchain AS serve
COPY Cargo.toml Cargo.lock clippy.toml rust-toolchain.toml ./
COPY crates/ crates/
RUN --network=none cargo build --release --locked --offline \
    && install -m 0755 target/release/edu /usr/local/bin/edu
WORKDIR /site
EXPOSE 1111
CMD ["edu", "serve", "--drafts", "--bind", "0.0.0.0:1111"]
