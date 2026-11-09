# One command for each job, for people and for AI coding agents; see AGENTS.md.
# `make` alone runs `check`, since `serve` runs until it is stopped.
.PHONY: check ci serve

check: ## Check format and lint, run the tests, and check every page and link
	cargo fmt --all --check
	cargo clippy --workspace --all-targets --locked -- -D warnings
	cargo test --workspace --locked
	cargo run --locked -p edu-site -- check

ci: ## Run the full check of CI in its pinned Docker image
	docker build --target check .

serve: ## Serve the site with its drafts at http://127.0.0.1:1111, until stopped
	cargo run --locked -p edu-site -- serve --drafts
