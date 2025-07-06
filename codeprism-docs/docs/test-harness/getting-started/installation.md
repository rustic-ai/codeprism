---
title: Installation Guide
description: Complete installation instructions for all platforms and use cases
sidebar_position: 2
---

# Installation Guide - Mandrel MCP Test Harness

Complete installation instructions for the Mandrel MCP Test Harness across different platforms and use cases.

## 🚀 Quick Install (Recommended)

### Prerequisites

- **Rust 1.70 or later** - [Install from rustup.rs](https://rustup.rs/)
- **Git** - For cloning the repository

### From Source (Stable)

```bash
# Clone the repository
git clone https://github.com/rustic-ai/codeprism.git
cd codeprism

# Install moth binary globally
cargo install --path crates/mandrel-mcp-th

# Verify installation
moth --version
```

This installs the `moth` binary to your Cargo bin directory (usually `~/.cargo/bin/`), which should be in your PATH.

## 📦 Platform-Specific Installation

### Linux (Ubuntu/Debian)

```bash
# Install Rust if not already installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install build dependencies
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev

# Clone and install
git clone https://github.com/rustic-ai/codeprism.git
cd codeprism
cargo install --path crates/mandrel-mcp-th

# Add cargo bin to PATH if not already done
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc

# Verify
moth --version
```

### Linux (RHEL/CentOS/Fedora)

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install build dependencies
sudo dnf install -y gcc openssl-devel pkgconf-pkg-config

# Clone and install
git clone https://github.com/rustic-ai/codeprism.git
cd codeprism
cargo install --path crates/mandrel-mcp-th

# Verify
moth --version
```

### macOS

```bash
# Install Rust if not already installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install Xcode command line tools if needed
xcode-select --install

# Clone and install
git clone https://github.com/rustic-ai/codeprism.git
cd codeprism
cargo install --path crates/mandrel-mcp-th

# Verify
moth --version
```

### Windows

```powershell
# Install Rust from https://rustup.rs/ or use winget
winget install Rustlang.Rustup

# Install Git if not already installed
winget install Git.Git

# Clone and install
git clone https://github.com/rustic-ai/codeprism.git
cd codeprism
cargo install --path crates/mandrel-mcp-th

# Verify (restart terminal first)
moth --version
```

## 🔧 Development Installation

For contributors or users who want to build from source with development features:

### Full Development Setup

```bash
# Clone repository
git clone https://github.com/rustic-ai/codeprism.git
cd codeprism

# Install development dependencies
rustup component add clippy rustfmt

# Build in development mode
cd crates/mandrel-mcp-th
cargo build

# Run tests to verify installation
cargo test

# Build optimized release version
cargo build --release

# Binary location: ../../target/release/moth
../../target/release/moth --version
```

### Editable Installation

For development where you frequently modify the code:

```bash
# In the mandrel-mcp-th directory
cd crates/mandrel-mcp-th

# Create a symlink to the binary (Linux/macOS)
mkdir -p ~/.local/bin
ln -sf $(pwd)/../../target/release/moth ~/.local/bin/moth

# Add to PATH if needed
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc

# Now rebuild will automatically update the binary
cargo build --release
```

## 🐳 Docker Installation

### Using Pre-built Image (Future)

*Note: Docker images are planned for future releases*

```bash
# Pull the official image (when available)
docker pull codeprism/mandrel-mcp-th:latest

# Run tests with Docker
docker run --rm -v $(pwd):/workspace codeprism/mandrel-mcp-th moth test /workspace/spec.yaml
```

### Building Docker Image

```bash
# In the project root
docker build -f crates/mandrel-mcp-th/Dockerfile -t mandrel-mcp-th .

# Run with Docker
docker run --rm -v $(pwd):/workspace mandrel-mcp-th moth test /workspace/spec.yaml
```

Sample Dockerfile (create as `crates/mandrel-mcp-th/Dockerfile`):

```dockerfile
FROM rust:1.70 as builder

WORKDIR /app
COPY . .
RUN cargo build --release --bin moth

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/moth /usr/local/bin/moth

WORKDIR /workspace
ENTRYPOINT ["moth"]
```

## 📋 Verification

### Basic Installation Test

```bash
# Check version
moth --version

# Validate help output
moth --help

# Test with a minimal specification
cat > test-spec.yaml << 'EOF'
name: "Test Server"
version: "1.0.0"
capabilities:
  tools: false
  resources: false
  prompts: false
  sampling: false
  logging: false
server:
  command: "echo"
  args: ["test"]
  transport: "stdio"
EOF

# Validate the specification
moth validate test-spec.yaml

# Clean up
rm test-spec.yaml
```

Expected output:
```
✅ Specification validation successful
  Name: Test Server
  Version: 1.0.0
  Tools: 0, Resources: 0, Prompts: 0
  Server: echo test
```

### Advanced Verification

```bash
# Test all commands
moth --help
moth test --help
moth validate --help
moth list --help
moth version

# Test with example specifications
cd docs/test-harness/examples
moth validate codeprism-mcp.yaml
moth list codeprism-mcp.yaml --detailed
```

## 🚨 Troubleshooting Installation

### Common Issues

#### "moth: command not found"

**Problem**: The binary is not in your PATH.

**Solutions**:
```bash
# Check if cargo bin is in PATH
echo $PATH | grep -q "$HOME/.cargo/bin" && echo "Cargo bin in PATH" || echo "Cargo bin NOT in PATH"

# Add cargo bin to PATH
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc

# Or use the full path
~/.cargo/bin/moth --version
```

#### Compilation Errors

**Problem**: Build dependencies missing.

**Linux Solutions**:
```bash
# Ubuntu/Debian
sudo apt install build-essential pkg-config libssl-dev

# RHEL/CentOS/Fedora
sudo dnf install gcc openssl-devel pkgconf-pkg-config
```

**macOS Solutions**:
```bash
# Install Xcode command line tools
xcode-select --install

# Update Rust toolchain
rustup update
```

#### Permission Errors

**Problem**: Cannot write to installation directory.

**Solutions**:
```bash
# Install to user directory instead
cargo install --path crates/mandrel-mcp-th --root ~/.local

# Add local bin to PATH
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
```

#### Rust Version Issues

**Problem**: Rust version too old.

**Solutions**:
```bash
# Update Rust toolchain
rustup update

# Check Rust version
rustc --version

# Should be 1.70 or later
```

### Getting Help

If you encounter installation issues:

1. **Check Prerequisites**: Ensure Rust 1.70+ and Git are installed
2. **Update Everything**: Update Rust, Git, and system packages
3. **Check Documentation**: Review platform-specific instructions
4. **Search Issues**: Check [GitHub Issues](https://github.com/rustic-ai/codeprism/issues)
5. **Create Issue**: If problem persists, create a detailed issue report

## 🔄 Updating

### Update from Source

```bash
# Navigate to cloned repository
cd codeprism

# Pull latest changes
git pull origin main

# Rebuild and reinstall
cargo install --path crates/mandrel-mcp-th

# Verify new version
moth --version
```

### Clean Reinstall

```bash
# Remove existing installation
cargo uninstall moth

# Fresh install
cargo install --path crates/mandrel-mcp-th
```

## 🏢 Enterprise Installation

### System-wide Installation

For organization-wide deployment:

```bash
# Install to system location (requires sudo)
sudo cargo install --path crates/mandrel-mcp-th --root /usr/local

# Binary will be at /usr/local/bin/moth
which moth
```

### Package Manager Integration

For future package manager support:

```bash
# Homebrew (macOS) - planned
brew install rustic-ai/tap/moth

# APT (Ubuntu/Debian) - planned
sudo apt install moth

# Winget (Windows) - planned
winget install RusticAI.Moth
```

## 🎯 Next Steps

After successful installation:

1. **[Quick Start Guide](quick-start.md)** - Run your first test in 5 minutes
2. **[CLI Reference](../cli-reference.md)** - Complete command documentation
3. **[Configuration Guide](../configuration-reference.md)** - Learn YAML specification format
4. **[Examples](../examples/)** - Real-world test specifications

---

**Need additional help?** Check our [troubleshooting guide](../troubleshooting.md) or [open an issue](https://github.com/rustic-ai/codeprism/issues) on GitHub. 