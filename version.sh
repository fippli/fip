#!/bin/bash

set -e

if [ -z "$1" ]; then
    echo "Usage: ./version.sh <major|minor|patch>"
    echo "Example: ./version.sh patch  # 0.1.0 -> 0.1.1"
    echo "         ./version.sh minor  # 0.1.0 -> 0.2.0"
    echo "         ./version.sh major  # 0.1.0 -> 1.0.0"
    exit 1
fi

INCREMENT_TYPE="$1"

if [[ ! "$INCREMENT_TYPE" =~ ^(major|minor|patch)$ ]]; then
    echo "Error: Invalid increment type '$INCREMENT_TYPE'"
    echo "Must be one of: major, minor, patch"
    exit 1
fi

# Get current version from Cargo.toml
CURRENT_VERSION=$(grep '^version = ' Cargo.toml | sed 's/^version = "\(.*\)"/\1/')

if [ -z "$CURRENT_VERSION" ]; then
    echo "Error: Could not find version in Cargo.toml"
    exit 1
fi

echo "Current version: $CURRENT_VERSION"

# Parse version components
IFS='.' read -ra VERSION_PARTS <<< "$CURRENT_VERSION"
MAJOR="${VERSION_PARTS[0]}"
MINOR="${VERSION_PARTS[1]}"
PATCH="${VERSION_PARTS[2]}"

# Increment based on type
case "$INCREMENT_TYPE" in
    major)
        MAJOR=$((MAJOR + 1))
        MINOR=0
        PATCH=0
        ;;
    minor)
        MINOR=$((MINOR + 1))
        PATCH=0
        ;;
    patch)
        PATCH=$((PATCH + 1))
        ;;
esac

NEW_VERSION="$MAJOR.$MINOR.$PATCH"

echo "New version: $NEW_VERSION"
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

