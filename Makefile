# One command for each job, for people and for AI coding agents; see AGENTS.md.
# `make` alone runs `check`, since `serve` runs until it is stopped.
.PHONY: check ci serve

# The pages are checked before the tests, one of which builds this site, as in the Dockerfile.
check: ## Check format and lint, check every page and link, then run the tests
	cargo fmt --all --check
	cargo clippy --workspace --all-targets --locked -- -D warnings
	cargo run --locked -p edu-site -- check
	cargo test --workspace --locked

ci: ## Run the full check of CI in its pinned Docker image
	docker build --target check .

serve: ## Serve the site with its drafts at http://127.0.0.1:1111, until stopped
	cargo run --locked -p edu-site -- serve --drafts
