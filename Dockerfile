# The generator's toolchain, the same on laptops and in CI. Check the generator and every page,
# and build the site twice to see that both builds are the same, with:
#   docker build --target check .
# The tag names the Rust version of rust-toolchain.toml (CI fails when they differ); the digest
# pins the image, since tags are re-pushed.
FROM rust:1.99.1-slim-trixie@sha256:b8e062b1c3a80183df83d9ba187cadd2b5ab84bc78d453dbfeb2fccd5f982247 AS toolchain
# The image installs Rust's minimal profile, which lacks these two.
RUN rustup component add clippy rustfmt
WORKDIR /src

# Format, lint, test and build with no network: the generator uses the standard library only.
FROM toolchain AS check
COPY Cargo.toml Cargo.lock clippy.toml rust-toolchain.toml ./
COPY crates/ crates/
RUN --network=none cargo fmt --all --check
RUN --network=none cargo clippy --workspace --all-targets --locked --offline -- -D warnings
RUN --network=none cargo test --workspace --locked --offline
RUN --network=none cargo build --workspace --release --locked --offline
# The settings, the layouts, the static files and then the pages come last, so that editing one
# reuses every layer above.
COPY site.yaml ./
COPY layouts/ layouts/
COPY static/ static/
COPY content/ content/
RUN --network=none target/release/edu check
RUN --network=none target/release/edu build && mv public /tmp/first \
    && target/release/edu build && diff -r /tmp/first public
