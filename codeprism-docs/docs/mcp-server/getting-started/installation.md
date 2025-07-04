---
title: Installation Guide
description: Complete setup instructions for CodePrism across different platforms and environments
sidebar_position: 1
---

# Installation Guide

This guide will walk you through installing and setting up CodePrism on your system. CodePrism is available for Linux, macOS, and Windows, with support for multiple installation methods.

## Prerequisites

Before installing CodePrism, ensure you have:

- **Rust 1.70+** - Download from [rustup.rs](https://rustup.rs/)
- **Git** - For cloning the repository
- **A supported operating system**: Linux, macOS, or Windows

## Installation Methods

### Method 1: Install from Releases (Recommended)

Download the latest pre-built binary from our GitHub releases:

```bash
# Linux x86_64
curl -L https://github.com/rustic-ai/codeprism/releases/latest/download/codeprism-linux-x86_64.tar.gz | tar xz
sudo mv codeprism /usr/local/bin/
sudo chmod +x /usr/local/bin/codeprism

# macOS
curl -L https://github.com/rustic-ai/codeprism/releases/latest/download/codeprism-macos.tar.gz | tar xz
sudo mv codeprism /usr/local/bin/
sudo chmod +x /usr/local/bin/codeprism

# Windows (PowerShell)
Invoke-WebRequest -Uri "https://github.com/rustic-ai/codeprism/releases/latest/download/codeprism-windows.zip" -OutFile "codeprism.zip"
Expand-Archive codeprism.zip -DestinationPath .
# Add to PATH manually or copy to a directory in PATH
```

### Method 2: Build from Source

```bash
# Clone the repository
git clone https://github.com/rustic-ai/codeprism.git
cd codeprism

# Build the release version
cargo build --release

# The binary will be available at target/release/codeprism
# Copy to your preferred location:
sudo cp target/release/codeprism /usr/local/bin/
```

### Method 3: Install with Cargo

```bash
# Install directly from the git repository
cargo install --git https://github.com/rustic-ai/codeprism.git

# Or install from crates.io (when available)
cargo install codeprism
```

## Verify Installation

After installation, verify that CodePrism is working correctly:

```bash
# Check version
codeprism --version

# Run basic health check
codeprism --help
```

## MCP Server Setup

To use CodePrism as an MCP server with AI assistants:

### Claude Desktop Configuration

Add to your Claude Desktop configuration file:

**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
**Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

```json
{
  "mcpServers": {
    "codeprism": {
      "command": "codeprism",
      "args": ["--mcp"],
      "env": {
        "CODEPRISM_PROJECT_ROOT": "/path/to/your/project"
      }
    }
  }
}
```

### VS Code Configuration

Install the MCP extension and add CodePrism to your settings:

```json
{
  "mcp.servers": [
    {
      "name": "codeprism",
      "command": "codeprism",
      "args": ["--mcp"],
      "workspaceFolder": "${workspaceFolder}"
    }
  ]
}
```

### Cursor Configuration

CodePrism integrates natively with Cursor. Add to your Cursor settings:

```json
{
  "mcp.servers": {
    "codeprism": {
      "command": "codeprism",
      "args": ["--mcp", "--project-root", "${workspaceFolder}"]
    }
  }
}
```

## Quick Start

Once installed, you can start using CodePrism immediately:

```bash
# Analyze your current directory
codeprism analyze .

# Start MCP server for AI assistant integration
codeprism --mcp --project-root /path/to/your/project

# Get repository statistics
codeprism stats /path/to/your/project

# Search for symbols
codeprism search "function_name" /path/to/your/project
```

## Configuration

CodePrism can be configured through environment variables or a configuration file.

### Environment Variables

```bash
export CODEPRISM_PROJECT_ROOT="/path/to/your/project"
export CODEPRISM_LOG_LEVEL="info"
export CODEPRISM_CACHE_SIZE="1000"
export CODEPRISM_ENABLE_WATCH="true"
```

### Configuration File

Create `~/.config/codeprism/config.toml`:

```toml
[server]
project_root = "/path/to/your/project"
log_level = "info"
cache_size = 1000
enable_watch = true

[analysis]
languages = ["python", "javascript", "typescript"]
ignore_patterns = ["node_modules", ".git", "target"]

[mcp]
enable_resources = true
enable_tools = true
enable_prompts = true
```

## Platform-Specific Setup

### Linux

```bash
# Ubuntu/Debian dependencies
sudo apt-get update
sudo apt-get install build-essential pkg-config libssl-dev

# Arch Linux
sudo pacman -S base-devel openssl

# Add to PATH (if needed)
echo 'export PATH="$PATH:$HOME/.local/bin"' >> ~/.bashrc
source ~/.bashrc
```

### macOS

```bash
# Install Xcode command line tools
xcode-select --install

# Using Homebrew (optional)
brew install openssl pkg-config

# Add to PATH (if needed)
echo 'export PATH="$PATH:/usr/local/bin"' >> ~/.zshrc
source ~/.zshrc
```

### Windows

1. Install Visual Studio Build Tools or Visual Studio with C++ support
2. Install Git for Windows
3. Add the installation directory to your PATH environment variable

## Troubleshooting

### Common Issues

**Command not found**
```bash
# Check if binary is in PATH
which codeprism

# If not found, add installation directory to PATH
export PATH="$PATH:/usr/local/bin"
```

**Permission denied**
```bash
# Make binary executable
chmod +x /usr/local/bin/codeprism

# Or run with sudo if needed for installation
sudo cp target/release/codeprism /usr/local/bin/
```

**Build errors on older systems**
```bash
# Update Rust to latest version
rustup update stable

# Use system OpenSSL if needed
export OPENSSL_DIR=/usr/include/openssl
```

### Getting Help

If you encounter issues:

1. Check the [troubleshooting guide](../../test-harness/troubleshooting)
2. Review the [GitHub issues](https://github.com/rustic-ai/codeprism/issues)
3. Join our [community discussions](https://github.com/rustic-ai/codeprism/discussions)

## Next Steps

After installation:

1. **[Learn the basics](../../intro)** - Get familiar with CodePrism commands
2. **[Explore the architecture](../../architecture/overview)** - Understand how CodePrism works
3. **[Check current status](../../architecture/current-status)** - See what features are available
4. **[Configure your AI assistant](../overview)** - Set up MCP integration

---

**Installation complete?** Continue with the [API Reference](../api-reference) to explore CodePrism's capabilities! 