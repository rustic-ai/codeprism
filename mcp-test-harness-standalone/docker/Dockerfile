# Multi-stage build for MCP Test Harness
FROM rust:1.80-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy manifests
COPY Cargo.toml ./

# Create dummy source to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# Copy actual source code
COPY src/ ./src/
COPY configs/ ./configs/

# Build application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    nodejs \
    npm \
    python3 \
    python3-pip \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false mcp-test-harness

# Copy binary and configs
COPY --from=builder /app/target/release/mcp-test-harness /usr/local/bin/
COPY --from=builder /app/configs/ /etc/mcp-test-harness/configs/

# Create directories for data
RUN mkdir -p /var/lib/mcp-test-harness/baselines \
             /var/lib/mcp-test-harness/reports \
             /tmp/mcp-test-harness \
    && chown -R mcp-test-harness:mcp-test-harness /var/lib/mcp-test-harness \
    && chown -R mcp-test-harness:mcp-test-harness /tmp/mcp-test-harness

# Switch to non-root user
USER mcp-test-harness

# Set working directory
WORKDIR /tmp/mcp-test-harness

# Expose default port (for HTTP MCP servers)
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD mcp-test-harness --help || exit 1

# Default command
ENTRYPOINT ["mcp-test-harness"]
CMD ["--help"]
