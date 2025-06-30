# Installation Guide

Complete installation instructions for the MCP Test Harness on all supported platforms.

## 📋 System Requirements

### Minimum Requirements
- **Operating System**: Linux, macOS, or Windows 10+
- **Memory**: 2GB RAM available
- **Disk Space**: 500MB free space
- **Network**: Internet access for downloading dependencies

### Recommended Requirements
- **Operating System**: Linux (Ubuntu 20.04+) or macOS 12+
- **Memory**: 4GB RAM available
- **Disk Space**: 2GB free space
- **CPU**: Multi-core processor for parallel testing

### Dependencies
- **Rust**: Version 1.70+ (automatically installed via rustup)
- **Git**: For cloning the repository
- **Docker** (optional): For containerized testing
- **Python 3.8+** (optional): For custom validation scripts

## 🚀 Quick Installation

### Option 1: Install from Pre-built Binary (Recommended)

```bash
# Download the latest release
curl -L https://github.com/rustic-ai/codeprism/releases/latest/download/mcp-test-harness-linux.tar.gz | tar xz

# Move to system PATH
sudo mv mcp-test-harness /usr/local/bin/

# Verify installation
mcp-test-harness --version
```

### Option 2: Install via Cargo

```bash
# Install directly from crates.io
cargo install mcp-test-harness

# Or install from GitHub
cargo install --git https://github.com/rustic-ai/codeprism.git mcp-test-harness
```

### Option 3: Build from Source

```bash
# Clone the repository
git clone https://github.com/rustic-ai/codeprism.git
cd codeprism

# Build the test harness
cargo build --release --bin mcp-test-harness

# Install to system PATH
cargo install --path crates/mcp-test-harness
```

## 🐧 Linux Installation

### Ubuntu/Debian

```bash
# Update package list
sudo apt update

# Install dependencies
sudo apt install -y curl build-essential pkg-config libssl-dev

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install MCP Test Harness
cargo install mcp-test-harness

# Add to PATH (if not already added)
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### CentOS/RHEL/Fedora

```bash
# Install dependencies
sudo dnf install -y curl gcc gcc-c++ openssl-devel

# Or for older versions (CentOS 7)
# sudo yum install -y curl gcc gcc-c++ openssl-devel

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install MCP Test Harness
cargo install mcp-test-harness
```

### Arch Linux

```bash
# Install dependencies
sudo pacman -S rust cargo

# Install MCP Test Harness
cargo install mcp-test-harness
```

## 🍎 macOS Installation

### Using Homebrew (Recommended)

```bash
# Install Homebrew if not already installed
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install Rust
brew install rust

# Install MCP Test Harness
cargo install mcp-test-harness
```

### Manual Installation

```bash
# Install Xcode command line tools
xcode-select --install

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install MCP Test Harness
cargo install mcp-test-harness
```

## 🪟 Windows Installation

### Option 1: Using PowerShell (Recommended)

```powershell
# Install Rust
Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile "rustup-init.exe"
.\rustup-init.exe -y
refreshenv

# Install MCP Test Harness
cargo install mcp-test-harness
```

### Option 2: Using Windows Subsystem for Linux (WSL)

```bash
# Install WSL2 and Ubuntu
wsl --install

# Follow Ubuntu installation instructions above
```

### Option 3: Using Visual Studio

1. Install Visual Studio Community with C++ build tools
2. Install Rust using the installer from https://rustup.rs/
3. Open Command Prompt or PowerShell
4. Run: `cargo install mcp-test-harness`

## 🐳 Docker Installation

### Pull Pre-built Image

```bash
# Pull the latest image
docker pull ghcr.io/rustic-ai/mcp-test-harness:latest

# Create an alias for easy usage
echo 'alias mcp-test-harness="docker run --rm -v $(pwd):/workspace ghcr.io/rustic-ai/mcp-test-harness:latest"' >> ~/.bashrc
source ~/.bashrc
```

### Build Your Own Image

```bash
# Clone repository
git clone https://github.com/rustic-ai/codeprism.git
cd codeprism

# Build Docker image
docker build -t mcp-test-harness -f docker/Dockerfile .

# Run container
docker run --rm -v $(pwd):/workspace mcp-test-harness --help
```

### Docker Compose Setup

```yaml
# docker-compose.yml
version: '3.8'
services:
  mcp-test-harness:
    image: ghcr.io/rustic-ai/mcp-test-harness:latest
    volumes:
      - ./config:/app/config
      - ./test-results:/app/results
    environment:
      - RUST_LOG=info
    command: test --config /app/config/test-harness.yaml
```

## ☁️ Cloud Installation

### AWS EC2

```bash
# Launch EC2 instance (Amazon Linux 2)
# SSH into instance

# Install dependencies
sudo yum update -y
sudo yum install -y gcc gcc-c++

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install MCP Test Harness
cargo install mcp-test-harness

# Configure firewall (if testing HTTP/WebSocket servers)
sudo firewall-cmd --add-port=3000-3100/tcp --permanent
sudo firewall-cmd --reload
```

### Google Cloud Platform

```bash
# Create VM instance
gcloud compute instances create mcp-test-harness \
  --image-family=ubuntu-2004-lts \
  --image-project=ubuntu-os-cloud \
  --machine-type=e2-medium

# SSH into instance
gcloud compute ssh mcp-test-harness

# Follow Ubuntu installation instructions
```

### Azure

```bash
# Create VM
az vm create \
  --resource-group myResourceGroup \
  --name mcp-test-harness \
  --image UbuntuLTS \
  --admin-username azureuser \
  --generate-ssh-keys

# SSH into VM
az vm run-command invoke \
  --resource-group myResourceGroup \
  --name mcp-test-harness \
  --command-id RunShellScript \
  --scripts "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
```

## 🔧 Development Installation

For contributors and developers who want to work on the MCP Test Harness itself:

```bash
# Clone with all submodules
git clone --recursive https://github.com/rustic-ai/codeprism.git
cd codeprism

# Install development dependencies
cargo install cargo-watch cargo-tarpaulin

# Build in development mode
cargo build --workspace

# Run tests
cargo test --workspace

# Install in development mode (with debugging symbols)
cargo install --path crates/mcp-test-harness --debug
```

## ✅ Verification

After installation, verify everything is working correctly:

```bash
# Check version
mcp-test-harness --version

# Check help
mcp-test-harness --help

# Run self-test
mcp-test-harness validate --help

# Create a minimal test configuration
cat > test-installation.yaml << EOF
global:
  max_global_concurrency: 1
  global_timeout_seconds: 30

server:
  transport: "stdio"
  start_command: "echo"
  args: ["Hello, MCP Test Harness!"]

test_suites:
  - name: "installation_test"
    test_cases:
      - id: "echo_test"
        tool_name: "echo"
        input_params: {}
        expected:
          allow_any_response: true
EOF

# Run installation test
mcp-test-harness test --config test-installation.yaml --dry-run
```

Expected output:
```
✅ Configuration validation passed
✅ Server connectivity test passed
✅ Installation verification successful
```

## 🔄 Updating

### Update via Cargo

```bash
# Update to latest version
cargo install mcp-test-harness --force

# Update all Rust packages
cargo install-update --all
```

### Update Docker Image

```bash
# Pull latest image
docker pull ghcr.io/rustic-ai/mcp-test-harness:latest

# Remove old image (optional)
docker image prune
```

## 🗑️ Uninstallation

### Remove Cargo Installation

```bash
# Remove binary
cargo uninstall mcp-test-harness

# Remove Rust (if no longer needed)
rustup self uninstall
```

### Remove Docker Installation

```bash
# Remove images
docker rmi ghcr.io/rustic-ai/mcp-test-harness:latest

# Remove containers
docker container prune
```

## 🐛 Troubleshooting Installation

### Common Issues

#### "cargo: command not found"
```bash
# Ensure Rust is installed and in PATH
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

#### "linking with `cc` failed" on Linux
```bash
# Install build tools
sudo apt install build-essential  # Ubuntu/Debian
sudo dnf install gcc gcc-c++      # Fedora/CentOS
```

#### "failed to run custom build command for `openssl-sys`"
```bash
# Install OpenSSL development headers
sudo apt install libssl-dev pkg-config  # Ubuntu/Debian
sudo dnf install openssl-devel          # Fedora/CentOS
```

#### Permission denied on macOS
```bash
# Fix permissions
sudo chown -R $(whoami) ~/.cargo
```

#### Slow compilation
```bash
# Use parallel compilation
export CARGO_BUILD_JOBS=$(nproc)

# Use alternative registry
export CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
```

### Getting Help

If you encounter issues not covered here:

1. Check the [Troubleshooting Guide](../troubleshooting.md)
2. Search existing [GitHub Issues](https://github.com/rustic-ai/codeprism/issues)
3. Create a new issue with:
   - Your operating system and version
   - Rust version (`rustc --version`)
   - Complete error message
   - Steps to reproduce

## 📚 Next Steps

After successful installation:

1. Read the [First Test Guide](first-test.md) to run your first test
2. Learn about [Basic Configuration](basic-configuration.md)
3. Explore the [User Guide](../user-guide.md) for comprehensive usage instructions
4. Check out [Example Configurations](../examples/) for real-world setups

---

**Installation complete!** 🎉 You're ready to start testing MCP servers. 