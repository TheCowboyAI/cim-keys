.PHONY: run run-debug build build-release check test clean help

# Default target
help:
	@echo "CIM Keys - Build Commands"
	@echo ""
	@echo "Usage:"
	@echo "  make run          - Run GUI in release mode (optimized)"
	@echo "  make run-debug    - Run GUI in debug mode"
	@echo "  make build        - Build release binary"
	@echo "  make build-debug  - Build debug binary"
	@echo "  make check        - Check code without building"
	@echo "  make test         - Run tests"
	@echo "  make clean        - Clean build artifacts"

# Run the GUI in release mode (optimized)
run:
	@echo "🚀 Launching CIM Keys GUI (release)..."
	cargo run --release --bin cim-keys-gui --features gui

# Run the GUI in debug mode
run-debug:
	@echo "🔧 Launching CIM Keys GUI (debug)..."
	cargo run --bin cim-keys-gui --features gui

# Build release binary
build:
	@echo "🔨 Building release binary..."
	cargo build --release --features gui

# Build debug binary
build-debug:
	@echo "🔨 Building debug binary..."
	cargo build --features gui

# Check code compilation
check:
	@echo "✅ Checking code..."
	cargo check --features gui

# Run tests
test:
	@echo "🧪 Running tests..."
	cargo test --all-features

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean
