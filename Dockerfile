# Multi-stage Dockerfile for CodePrism MCP Server
# Optimized for size and security - installs from crates.io

# Build stage
FROM rust:1.82-slim AS builder

# Install system dependencies for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install the published crate directly from crates.io
# This is much faster and more consistent than building from source
ARG CRATE_VERSION
RUN cargo install codeprism-mcp --version ${CRATE_VERSION} --locked

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -r -s /bin/false codeprism

# Copy the binary from cargo install location
COPY --from=builder /usr/local/cargo/bin/codeprism-mcp /usr/local/bin/codeprism-mcp

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
    CMD codeprism-mcp --help || exit 1

# Default command
CMD ["codeprism-mcp"]

# Labels for metadata
LABEL org.opencontainers.image.title="CodePrism MCP Server"
LABEL org.opencontainers.image.description="100% AI-generated code intelligence MCP server"
LABEL org.opencontainers.image.vendor="The Rustic Initiative"
LABEL org.opencontainers.image.licenses="MIT OR Apache-2.0"
LABEL org.opencontainers.image.source="https://github.com/rustic-ai/codeprism"
LABEL org.opencontainers.image.documentation="https://github.com/rustic-ai/codeprism/blob/main/README.md" 