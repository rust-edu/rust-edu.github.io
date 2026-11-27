# One command for each job; README.md lists them.
# `make` alone runs `check`, since `serve` runs until it is stopped.
.PHONY: check guards guard-lock guard-live guard-rust ci preview serve

# The crates of this repository, in the order of `sort`: the only names that Cargo.lock may list.
# A new crate of our own is added here on purpose.
CRATES := edu-markdown edu-site edu-template edu-yaml

check: guards ## Check the guards, format and lint, check every page and link, then run the tests
	cargo fmt --all --check
	cargo clippy --workspace --all-targets --locked -- -D warnings
	cargo run --locked -p edu-site -- check
	sh tests/check-feed-ids.sh cargo run --manifest-path "$$(pwd)/Cargo.toml" --locked --offline -p edu-site --
	cargo test --workspace --locked

# The rules of README.md that neither cargo nor the Docker build can check. An error has the shape
# of an error of `edu check`, `file:line:col: error: …`, which ci.yml shows on that line of a pull
# request.
guards: guard-lock guard-live guard-rust ## Check Cargo.lock, the live stage and the Rust version

# Cargo.lock lists only crates of this repository. A crate copied into crates/ has no source line,
# so the names are checked too.
guard-lock:
	@if [ ! -f Cargo.lock ]; then \
	  echo "Cargo.lock:1:1: error: there is no Cargo.lock" >&2; \
	  exit 1; \
	fi
	@lines=$$(grep -n '^source = ' Cargo.lock | cut -d: -f1); \
	for line in $$lines; do \
	  echo "Cargo.lock:$$line:1: error: Cargo.lock lists a crate from a registry or git; the generator uses the Rust standard library only" >&2; \
	done; \
	[ -z "$$lines" ]
	@crates=$$(sed -n 's/^name = "\(.*\)"$$/\1/p' Cargo.lock | LC_ALL=C sort | tr '\n' ' '); \
	packages=$$(grep -c '^\[\[package\]\]$$' Cargo.lock); \
	echo "Cargo.lock: $$crates"; \
	if [ "$$crates" != "$(CRATES) " ] || [ "$$packages" != $(words $(CRATES)) ]; then \
	  echo "Cargo.lock:1:1: error: Cargo.lock lists a crate other than $(CRATES); the generator uses the Rust standard library only" >&2; \
	  exit 1; \
	fi

# The live site is built on the checks. The line must be exactly this one.
guard-live:
	@if ! grep -qx 'FROM check AS live' Dockerfile; then \
	  line=$$(grep -n ' AS live' Dockerfile | head -n 1 | cut -d: -f1); \
	  echo "Dockerfile:$${line:-1}:1: error: the live stage is not FROM check; that stage is what stops a bad push to main from publishing (see README.md)" >&2; \
	  exit 1; \
	fi

# The Dockerfile and rust-toolchain.toml name the same Rust.
guard-rust:
	@image=$$(sed -n 's/^FROM rust:\([0-9.]*\)-.*/\1/p' Dockerfile); \
	channel=$$(sed -n 's/^channel = "\([0-9.]*\)"$$/\1/p' rust-toolchain.toml); \
	echo "Dockerfile: rust:$$image; rust-toolchain.toml: $$channel"; \
	if [ -z "$$image" ] || [ "$$image" != "$$channel" ]; then \
	  line=$$(grep -n '^FROM rust:' Dockerfile | head -n 1 | cut -d: -f1); \
	  echo "Dockerfile:$${line:-1}:1: error: the FROM tag names Rust $${image:-?} and rust-toolchain.toml names $${channel:-?}; bump both together" >&2; \
	  exit 1; \
	fi

# The image of the check stage is not kept: no command uses it, and its layers stay in the cache.
ci: guards ## Run what CI runs: the guards, then the full check in the pinned Docker image
	docker build --target check --output type=cacheonly .

preview: ## Build the site with its drafts into preview/, as CI does for each pull request
	docker build --target preview -o preview .

serve: ## Serve the site with its drafts at http://127.0.0.1:1111, until stopped
	cargo run --locked -p edu-site -- serve --drafts
