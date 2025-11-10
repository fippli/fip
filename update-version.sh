#!/bin/bash

set -e

if [ -z "$1" ]; then
    echo "Usage: ./update-version.sh <version>"
    echo "Example: ./update-version.sh 0.2.0"
    exit 1
fi

NEW_VERSION="$1"

echo "Updating version to $NEW_VERSION..."

# Update main Cargo.toml
sed -i '' "s/^version = \".*\"/version = \"$NEW_VERSION\"/" Cargo.toml

# Update linter Cargo.toml
sed -i '' "s/^version = \".*\"/version = \"$NEW_VERSION\"/" tools/linter/Cargo.toml

# Update formatter Cargo.toml
sed -i '' "s/^version = \".*\"/version = \"$NEW_VERSION\"/" tools/format/Cargo.toml

echo "Version updated to $NEW_VERSION in:"
echo "  - Cargo.toml"
echo "  - tools/linter/Cargo.toml"
echo "  - tools/format/Cargo.toml"

