#!/usr/bin/env bash
# Run the CIM Keys GUI using nix develop

set -e

# Default output directory
OUTPUT_DIR="${1:-/tmp/cim-keys-output}"

echo "🔐 CIM Keys GUI Launcher"
echo "📁 Output directory: $OUTPUT_DIR"
echo ""

# Build and run in nix develop shell
exec nix develop --command bash -c "
  cargo build --bin cim-keys-gui --features gui --release 2>/dev/null || \
  cargo build --bin cim-keys-gui --features gui
  echo 'Starting GUI...'
  ./target/debug/cim-keys-gui '$OUTPUT_DIR' || \
  ./target/release/cim-keys-gui '$OUTPUT_DIR'
"