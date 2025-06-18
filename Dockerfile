# Multi-stage Dockerfile for Prism MCP Server
# Optimized for size and security

# Build stage
FROM rust:1.82-slim as builder

# Install system dependencies for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /usr/src/prism

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY rust-toolchain.toml ./
COPY crates/ crates/

# Build dependencies (this step is cached if dependencies don't change)
RUN cargo build --release --bin prism-mcp

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -r -s /bin/false prism

# Copy the binary from builder stage
COPY --from=builder /usr/src/prism/target/release/prism-mcp /usr/local/bin/prism-mcp

# Create workspace directory
RUN mkdir -p /workspace && chown prism:prism /workspace

# Switch to non-root user
USER prism

# Set working directory
WORKDIR /workspace

# Expose default port (if needed for future HTTP interface)
EXPOSE 8080

# Environment variables
ENV RUST_LOG=info
ENV REPOSITORY_PATH=/workspace

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD prism-mcp --help || exit 1

# Default command
CMD ["prism-mcp"]

# Labels for metadata
LABEL org.opencontainers.image.title="Prism MCP Server"
LABEL org.opencontainers.image.description="100% AI-generated code intelligence MCP server"
LABEL org.opencontainers.image.vendor="The Rustic Initiative"
LABEL org.opencontainers.image.licenses="MIT OR Apache-2.0"
LABEL org.opencontainers.image.source="https://github.com/rustic-ai/prism"
LABEL org.opencontainers.image.documentation="https://github.com/rustic-ai/prism/blob/main/README.md" 