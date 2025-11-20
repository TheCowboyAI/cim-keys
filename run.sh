#!/usr/bin/env bash
# CIM Keys - Quick Launch Script

set -e

# Default to release mode for better performance
BUILD_MODE="${1:-release}"

if [ "$BUILD_MODE" = "debug" ]; then
    echo "🔧 Running in debug mode..."
    cargo run --bin cim-keys-gui --features gui
else
    echo "🚀 Running in release mode..."
    cargo run --release --bin cim-keys-gui --features gui
fi
