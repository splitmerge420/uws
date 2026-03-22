# Developer convenience Makefile for uws
#
# Works in GitHub Codespaces, CI, and local environments.
# Run `make help` to see all available targets.

.PHONY: help build build-release test lint fmt check clean audit docker mcp docs

# Default target
help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
	  awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

# ── Rust ─────────────────────────────────────────────────────────────────────

build: ## Build debug binary
	cargo build

build-release: ## Build optimised release binary
	cargo build --release

test: ## Run all unit and integration tests
	cargo test --verbose

lint: ## Run clippy (treat warnings as errors)
	cargo clippy -- -D warnings

fmt: ## Format all Rust code
	cargo fmt --all

fmt-check: ## Check formatting without modifying files (used in CI)
	cargo fmt --all -- --check

check: fmt-check lint test ## Run fmt-check + lint + test (full pre-commit check)

audit: ## Run cargo audit (security advisory scan)
	cargo audit

clean: ## Remove build artefacts
	cargo clean

# ── Install ───────────────────────────────────────────────────────────────────

install: build-release ## Install uws to /usr/local/bin (requires sudo)
	sudo install -m 755 target/release/uws /usr/local/bin/uws
	@echo "✓ uws installed to /usr/local/bin/uws"

install-user: build-release ## Install uws to ~/.local/bin (no sudo)
	mkdir -p ~/.local/bin
	install -m 755 target/release/uws ~/.local/bin/uws
	@echo "✓ uws installed to ~/.local/bin/uws"

# ── MCP Server ───────────────────────────────────────────────────────────────

mcp: ## Start the MCP server in stdio mode (for Claude Desktop / VS Code)
	python3 mcp_server/server.py --transport stdio

mcp-http: ## Start the MCP server in HTTP mode on port 8787
	python3 mcp_server/server.py --transport http --port 8787

# ── Docs ─────────────────────────────────────────────────────────────────────

docs: ## Build the MkDocs documentation site
	pip install -q mkdocs-material mkdocs-git-revision-date-localized-plugin
	mkdocs build

docs-serve: ## Serve docs locally with live reload
	pip install -q mkdocs-material mkdocs-git-revision-date-localized-plugin
	mkdocs serve

# ── Docker ───────────────────────────────────────────────────────────────────

docker: ## Build the Docker image locally
	docker build -t uws:local .

docker-run: ## Run the Docker image (prints help)
	docker run --rm uws:local

# ── Changesets ───────────────────────────────────────────────────────────────

changeset: ## Create a new changeset
	pnpm changeset

version: ## Apply changesets and bump version
	pnpm changeset version

release: ## Create a git tag from the current version
	bash scripts/tag-release.sh
