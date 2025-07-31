# Multi-stage Dockerfile for CodePrism MCP Server
# Supports both source builds (CI) and crates.io installs (releases)

# Build stage
FROM rustlang/rust:nightly-slim AS builder

# Install system dependencies for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Conditional build: either from source (CI) or from crates.io (releases)
ARG CRATE_VERSION
COPY . /tmp/source

# If CRATE_VERSION is set, install from crates.io (release builds)
# Otherwise, build from source (CI/development builds)
RUN if [ -n "$CRATE_VERSION" ]; then \
        echo "Building from crates.io version: $CRATE_VERSION" && \
        cargo install codeprism-mcp-server --version ${CRATE_VERSION} --locked; \
    else \
        echo "Building from source (CI/development)" && \
        cd /tmp/source && \
        cargo build --release --bin codeprism && \
        cp target/release/codeprism /usr/local/cargo/bin/; \
    fi

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -r -s /bin/false codeprism

# Copy the binary from cargo install location
COPY --from=builder /usr/local/cargo/bin/codeprism /usr/local/bin/codeprism

# Create workspace directory
RUN mkdir -p /workspace && chown codeprism:codeprism /workspace

# Switch to non-root user
USER codeprism

# Set working directory
WORKDIR /workspace

# Expose default port (if needed for future HTTP interface)
EXPOSE 8080

# Environment variables
ENV RUST_LOG=info
ENV REPOSITORY_PATH=/workspace

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD codeprism --help || exit 1

# Default command - run as MCP server
CMD ["codeprism", "--mcp"]

# Labels for metadata
LABEL org.opencontainers.image.title="CodePrism MCP Server"
LABEL org.opencontainers.image.description="100% AI-generated code intelligence MCP server"
LABEL org.opencontainers.image.vendor="The Rustic Initiative"
LABEL org.opencontainers.image.licenses="MIT OR Apache-2.0"
LABEL org.opencontainers.image.source="https://github.com/rustic-ai/codeprism"
LABEL org.opencontainers.image.documentation="https://github.com/rustic-ai/codeprism/blob/main/README.md" 