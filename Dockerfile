# Multi-stage Dockerfile for CodePrism MCP Server
# Optimized for size and security

# Build stage
FROM rust:1.82-slim as builder

# Install system dependencies for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /usr/src /codeprism

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY rust-toolchain.toml ./
COPY crates/ crates/

# Build dependencies (this step is cached if dependencies don't change)
RUN cargo build --release --bin codeprism-mcp

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -r -s /bin/false codeprism

# Copy the binary from builder stage
COPY --from=builder /usr/src /codeprism/target/release/codeprism-mcp /usr/local/bin/codeprism-mcp

# Create workspace directory
RUN mkdir -p /workspace && chown codeprismcodeprism /workspace

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
LABEL org.opencontainers.image.source="https://github.com/rustic-ai /codeprism"
LABEL org.opencontainers.image.documentation="https://github.com/rustic-ai /codeprism/blob/main/README.md" 