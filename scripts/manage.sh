#!/usr/bin/env bash
#
# NanoKVM Management Script
# Provides build, clean, run, and version management commands
#

set -euo pipefail

# Script configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
CARGO_TOML="$PROJECT_ROOT/Cargo.toml"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Target triple for RISC-V cross-compilation
RISCV_TARGET="riscv64gc-unknown-linux-musl"

# Docker image name
DOCKER_IMAGE="nanokvm-builder"

#
# Helper Functions
#

print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

get_version() {
    grep '^version' "$PROJECT_ROOT/crates/nanokvm-server/Cargo.toml" | head -1 | sed 's/.*"\(.*\)".*/\1/' || echo "0.1.0"
}

set_version() {
    local new_version="$1"
    
    # Update workspace Cargo.toml
    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' "s/^version = \".*\"/version = \"$new_version\"/" "$CARGO_TOML"
    else
        sed -i "s/^version = \".*\"/version = \"$new_version\"/" "$CARGO_TOML"
    fi
    
    print_success "Version updated to $new_version"
}

#
# Build Commands
#

cmd_build() {
    local mode="${1:-debug}"
    local target="${2:-native}"
    
    print_info "Building NanoKVM Server (mode: $mode, target: $target)..."
    
    cd "$PROJECT_ROOT"
    
    if [[ "$target" == "riscv" ]]; then
        if [[ "$mode" == "release" ]]; then
            cargo build --release --target "$RISCV_TARGET"
        else
            cargo build --target "$RISCV_TARGET"
        fi
    else
        if [[ "$mode" == "release" ]]; then
            cargo build --release
        else
            cargo build
        fi
    fi
    
    print_success "Build complete!"
}

cmd_release() {
    cmd_build "release" "${1:-native}"
}

cmd_docker_build() {
    print_info "Building with Docker..."
    
    cd "$PROJECT_ROOT"
    
    # Build Docker image if needed
    if ! docker image inspect "$DOCKER_IMAGE" >/dev/null 2>&1; then
        print_info "Building Docker image..."
        docker build -t "$DOCKER_IMAGE" -f docker/Dockerfile ./
    fi
    
    # Run build in Docker
    docker run --rm \
        -v "$PROJECT_ROOT:/workspace" \
        -w /workspace \
        "$DOCKER_IMAGE" \
        cargo build --release --target "$RISCV_TARGET"
    
    print_success "Docker build complete!"
}

#
# Clean Commands
#

cmd_clean() {
    print_info "Cleaning build artifacts..."
    
    cd "$PROJECT_ROOT"
    cargo clean
    
    # Clean any legacy Go artifacts
    if [[ -f "$PROJECT_ROOT/server/NanoKVM-Server" ]]; then
        rm -f "$PROJECT_ROOT/server/NanoKVM-Server"
        print_info "Removed legacy Go binary"
    fi
    
    # Clean support build artifacts
    if [[ -d "$PROJECT_ROOT/support/sg2002/build" ]]; then
        rm -rf "$PROJECT_ROOT/support/sg2002/build"
        print_info "Removed support build artifacts"
    fi
    
    print_success "Clean complete!"
}

#
# Run Commands
#

cmd_run() {
    local mode="${1:-debug}"
    
    print_info "Running NanoKVM Server (mode: $mode)..."
    
    cd "$PROJECT_ROOT"
    
    if [[ "$mode" == "release" ]]; then
        cargo run --release --bin nanokvm-server
    else
        cargo run --bin nanokvm-server
    fi
}

cmd_dev() {
    print_info "Starting development server with watch mode..."
    
    cd "$PROJECT_ROOT"
    
    if command -v cargo-watch &>/dev/null; then
        cargo watch -x "run --bin nanokvm-server"
    else
        print_warning "cargo-watch not installed. Run: cargo install cargo-watch"
        cmd_run "debug"
    fi
}

#
# Test Commands
#

cmd_test() {
    print_info "Running tests..."
    
    cd "$PROJECT_ROOT"
    cargo test --workspace
    
    print_success "Tests complete!"
}

cmd_lint() {
    print_info "Running lints..."
    
    cd "$PROJECT_ROOT"
    
    cargo fmt --check
    cargo clippy --workspace -- -D warnings
    
    print_success "Lint complete!"
}

cmd_fmt() {
    print_info "Formatting code..."
    
    cd "$PROJECT_ROOT"
    cargo fmt --all
    
    print_success "Format complete!"
}

#
# Version Commands
#

cmd_version() {
    local current_version
    current_version=$(get_version)
    echo "Current version: $current_version"
}

cmd_bump() {
    local bump_type="${1:-patch}"
    local current_version
    current_version=$(get_version)
    
    # Parse version
    IFS='.' read -r major minor patch <<< "$current_version"
    
    case "$bump_type" in
        major)
            major=$((major + 1))
            minor=0
            patch=0
            ;;
        minor)
            minor=$((minor + 1))
            patch=0
            ;;
        patch)
            patch=$((patch + 1))
            ;;
        *)
            print_error "Invalid bump type: $bump_type (use: major, minor, patch)"
            exit 1
            ;;
    esac
    
    local new_version="$major.$minor.$patch"
    set_version "$new_version"
    
    print_info "Version bumped from $current_version to $new_version"
}

cmd_set_version() {
    local new_version="$1"
    
    if [[ ! "$new_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        print_error "Invalid version format: $new_version (use: X.Y.Z)"
        exit 1
    fi
    
    set_version "$new_version"
}

#
# Info Commands
#

cmd_info() {
    echo "NanoKVM Project Info"
    echo "===================="
    echo "Version: $(get_version)"
    echo "Project root: $PROJECT_ROOT"
    echo "Rust version: $(rustc --version)"
    echo "Cargo version: $(cargo --version)"
    echo ""
    echo "Workspace members:"
    ls -1 "$PROJECT_ROOT/crates"
}

#
# Help
#

cmd_help() {
    cat << EOF
NanoKVM Management Script

Usage: $(basename "$0") <command> [options]

Build Commands:
  build [debug|release] [native|riscv]  Build the project
  release [native|riscv]                Build release version
  docker-build                          Build using Docker (for cross-compilation)
  clean                                 Clean build artifacts

Run Commands:
  run [debug|release]                   Run the server
  dev                                   Run in development mode with hot reload

Test Commands:
  test                                  Run all tests
  lint                                  Run lints (fmt check + clippy)
  fmt                                   Format code

Version Commands:
  version                               Show current version
  bump [major|minor|patch]              Bump version (default: patch)
  set-version <X.Y.Z>                   Set specific version

Info Commands:
  info                                  Show project information
  help                                  Show this help message

Examples:
  $(basename "$0") build                      # Debug build (native)
  $(basename "$0") build release              # Release build (native)
  $(basename "$0") build release riscv        # Release build for RISC-V
  $(basename "$0") docker-build               # Build using Docker
  $(basename "$0") bump minor                 # Bump minor version
  $(basename "$0") set-version 1.0.0          # Set version to 1.0.0

EOF
}

#
# Main Entry Point
#

main() {
    local command="${1:-help}"
    shift || true
    
    case "$command" in
        build)
            cmd_build "$@"
            ;;
        release)
            cmd_release "$@"
            ;;
        docker-build)
            cmd_docker_build
            ;;
        clean)
            cmd_clean
            ;;
        run)
            cmd_run "$@"
            ;;
        dev)
            cmd_dev
            ;;
        test)
            cmd_test
            ;;
        lint)
            cmd_lint
            ;;
        fmt)
            cmd_fmt
            ;;
        version)
            cmd_version
            ;;
        bump)
            cmd_bump "$@"
            ;;
        set-version)
            cmd_set_version "$@"
            ;;
        info)
            cmd_info
            ;;
        help|--help|-h)
            cmd_help
            ;;
        *)
            print_error "Unknown command: $command"
            cmd_help
            exit 1
            ;;
    esac
}

main "$@"
