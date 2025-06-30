#!/bin/bash
set -e

# Cross-platform build script for MCP Test Harness
# Builds binaries for Linux, macOS, and Windows

PROJECT_NAME="mcp-test-harness"
VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
BUILD_DIR="target/release"
DIST_DIR="dist"

echo "🔨 Building MCP Test Harness v$VERSION for multiple platforms..."

# Clean previous builds
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"

# Build targets
TARGETS=(
    "x86_64-unknown-linux-gnu"
    "x86_64-apple-darwin"
    "aarch64-apple-darwin"
    "x86_64-pc-windows-gnu"
)

for target in "${TARGETS[@]}"; do
    echo "🏗️  Building for $target..."
    
    # Install target if not already installed
    rustup target add "$target" || true
    
    # Build for target
    cargo build --release --target "$target"
    
    # Determine binary extension
    if [[ "$target" == *"windows"* ]]; then
        BINARY_EXT=".exe"
    else
        BINARY_EXT=""
    fi
    
    # Create distribution directory
    DIST_TARGET_DIR="$DIST_DIR/$PROJECT_NAME-$VERSION-$target"
    mkdir -p "$DIST_TARGET_DIR"
    
    # Copy binary
    cp "target/$target/release/$PROJECT_NAME$BINARY_EXT" "$DIST_TARGET_DIR/"
    
    # Copy configs and documentation
    cp -r configs "$DIST_TARGET_DIR/"
    cp README.md "$DIST_TARGET_DIR/" 2>/dev/null || echo "README.md not found, skipping"
    cp LICENSE* "$DIST_TARGET_DIR/" 2>/dev/null || echo "LICENSE not found, skipping"
    
    # Create archive
    cd "$DIST_DIR"
    if [[ "$target" == *"windows"* ]]; then
        zip -r "$PROJECT_NAME-$VERSION-$target.zip" "$PROJECT_NAME-$VERSION-$target"
    else
        tar -czf "$PROJECT_NAME-$VERSION-$target.tar.gz" "$PROJECT_NAME-$VERSION-$target"
    fi
    cd ..
    
    echo "✅ Built $target"
done

echo "🎉 Build complete! Distribution files:"
ls -la "$DIST_DIR"/*.tar.gz "$DIST_DIR"/*.zip 2>/dev/null || echo "No archives found"

echo ""
echo "📦 To install locally:"
echo "  cargo install --path ."
echo ""
echo "🐳 To build Docker image:"
echo "  docker build -t mcp-test-harness:$VERSION -f docker/Dockerfile ."
echo ""
echo "🚀 To publish to crates.io:"
echo "  cargo publish"
