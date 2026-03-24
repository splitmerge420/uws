# uws — Universal Workspace CLI
# Multi-stage Docker image for uws binary distribution and MCP server
#
# Usage:
#   # Run CLI
#   docker run --rm ghcr.io/splitmerge420/uws uws --help
#
#   # Run MCP server (stdio)
#   docker run --rm -i ghcr.io/splitmerge420/uws python3 mcp_server/server.py --transport stdio
#
#   # Run MCP server (HTTP)
#   docker run --rm -p 8787:8787 ghcr.io/splitmerge420/uws python3 mcp_server/server.py --transport http --port 8787

# ── Stage 1: Build the Rust binary ───────────────────────────────────────────
FROM rust:1-slim-bookworm AS builder

WORKDIR /build

# Cache dependencies separately from source
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main(){}' > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Build the real binary
COPY src ./src
RUN touch src/main.rs && cargo build --release

# ── Stage 2: Minimal runtime image ───────────────────────────────────────────
FROM python:3.12-slim-bookworm AS runtime

LABEL org.opencontainers.image.source="https://github.com/splitmerge420/uws"
LABEL org.opencontainers.image.description="Universal Workspace CLI — one tool for Google Workspace, Microsoft 365, Apple, Android, Chrome"
LABEL org.opencontainers.image.licenses="MIT"
LABEL org.opencontainers.image.vendor="splitmerge420"

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy uws binary
COPY --from=builder /build/target/release/uws /usr/local/bin/uws

# Copy MCP server
COPY mcp_server/ /app/mcp_server/

WORKDIR /app

# Default: print help
ENTRYPOINT ["uws"]
CMD ["--help"]
