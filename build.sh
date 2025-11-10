#!/bin/bash

set -e

echo "Building main project..."
cargo build --release

echo "Building linter..."
cargo build --release --manifest-path tools/linter/Cargo.toml

echo "Building formatter..."
cargo build --release --manifest-path tools/format/Cargo.toml

echo "Build complete!"
echo "Binaries:"
echo "  - target/release/fip"
echo "  - tools/linter/target/release/fip-lint"
echo "  - tools/format/target/release/fip-format"
