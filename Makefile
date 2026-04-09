# Makefile for NanoKVM Project (Rust Edition)
#
# This Makefile provides build, test, and management commands for the
# NanoKVM Rust workspace.

# Configuration
CARGO := cargo
IMAGE_NAME := nanokvm-rust-builder
RISCV_TARGET := riscv64gc-unknown-linux-musl
UID := $(shell id -u)
GID := $(shell id -g)
PWD := $(shell pwd)

# Docker run parameters
DOCKER_RUN := docker run -e UID=$(UID) -e GID=$(GID) -v $(PWD):/workspace -w /workspace --rm

.PHONY: help all build release test lint fmt clean docker-build check-root \
        dev run version info install-tools

# Default target
all: build

# Help target
help:
	@echo "NanoKVM Build System (Rust Edition)"
	@echo ""
	@echo "Build Commands:"
	@echo "  build         - Build debug version (native)"
	@echo "  release       - Build release version (native)"
	@echo "  docker-build  - Build release for RISC-V using Docker"
	@echo "  clean         - Clean build artifacts"
	@echo ""
	@echo "Development Commands:"
	@echo "  dev           - Run in development mode"
	@echo "  run           - Run the server"
	@echo "  test          - Run all tests"
	@echo "  lint          - Run clippy lints"
	@echo "  fmt           - Format code"
	@echo ""
	@echo "Version Commands:"
	@echo "  version       - Show current version"
	@echo ""
	@echo "Info Commands:"
	@echo "  info          - Show project information"
	@echo "  install-tools - Install development tools"
	@echo ""
	@echo "Or use: ./scripts/manage.sh <command>"

# Build debug version
build:
	@echo "Building NanoKVM (debug)..."
	$(CARGO) build --workspace

# Build release version
release:
	@echo "Building NanoKVM (release)..."
	$(CARGO) build --release --workspace

# Build for RISC-V target using Docker
docker-build: check-root
	@echo "Building for RISC-V using Docker..."
	@if ! docker image inspect $(IMAGE_NAME) >/dev/null 2>&1; then \
		echo "Building Docker image..."; \
		docker build -t $(IMAGE_NAME) -f docker/Dockerfile.rust ./; \
	fi
	$(DOCKER_RUN) $(IMAGE_NAME) cargo build --release --target $(RISCV_TARGET)

# Run the server (debug mode)
run:
	@echo "Running NanoKVM server..."
	$(CARGO) run --bin nanokvm-server

# Development mode with hot reload (requires cargo-watch)
dev:
	@echo "Starting development server..."
	@if command -v cargo-watch >/dev/null 2>&1; then \
		cargo watch -x "run --bin nanokvm-server"; \
	else \
		echo "cargo-watch not installed. Installing..."; \
		cargo install cargo-watch; \
		cargo watch -x "run --bin nanokvm-server"; \
	fi

# Run all tests
test:
	@echo "Running tests..."
	$(CARGO) test --workspace

# Run clippy lints
lint:
	@echo "Running lints..."
	$(CARGO) fmt --check
	$(CARGO) clippy --workspace -- -D warnings

# Format code
fmt:
	@echo "Formatting code..."
	$(CARGO) fmt --all

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	$(CARGO) clean
	@# Clean legacy artifacts
	@if [ -f server/NanoKVM-Server ]; then \
		rm -f server/NanoKVM-Server; \
		echo "Removed legacy server/NanoKVM-Server"; \
	fi
	@if [ -d support/sg2002/build ]; then \
		rm -rf support/sg2002/build; \
		echo "Removed support/sg2002/build"; \
	fi
	@echo "Clean completed."

# Show version
version:
	@grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/NanoKVM v\1/'

# Show project info
info:
	@echo "NanoKVM Project Info"
	@echo "===================="
	@./scripts/manage.sh info

# Security check - prevent running as root
check-root:
	@if [ "$$(id -u)" -eq 0 ]; then \
		echo "Error: Cannot run as root"; \
		exit 1; \
	fi

# Install development tools
install-tools:
	@echo "Installing development tools..."
	cargo install cargo-watch
	cargo install cargo-edit
	cargo install cargo-audit
	@echo "Tools installed."

# Legacy compatibility targets
app: build
support:
	@echo "Support libraries are now built as part of the Rust workspace"
	@echo "See crates/board-support/"