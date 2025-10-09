#!/usr/bin/env bash

set -e

REPO="yourusername/yourrepo"  # <-- Change this to your GitHub repo
BINARY_NAME="yourbinary"      # <-- Change this to your binary name
INSTALL_DIR="/usr/local/bin"

# Detect OS and ARCH
OS="$(uname -s)"
ARCH="$(uname -m)"

# Normalize OS
case "$OS" in
    Linux*)     OS="linux" ;;
    Darwin*)    OS="macos" ;;
    *)          echo "Unsupported OS: $OS" && exit 1 ;;
esac

# Normalize ARCH
case "$ARCH" in
    x86_64) ARCH="amd64" ;;
    arm64|aarch64) ARCH="arm64" ;;
    *)      echo "Unsupported architecture: $ARCH" && exit 1 ;;
esac

# Find latest release tag via GitHub API
echo "Fetching latest release info..."
LATEST_TAG=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep -oP '"tag_name": "\K(.*)(?=")')

if [ -z "$LATEST_TAG" ]; then
    echo "Could not fetch the latest release tag."
    exit 1
fi

# Construct download URL
FILENAME="${BINARY_NAME}-${OS}-${ARCH}"
URL="https://github.com/${REPO}/releases/download/${LATEST_TAG}/${FILENAME}"

echo "Downloading ${FILENAME} from $URL..."

# Download binary
curl -L -o "$BINARY_NAME" "$URL"

# Make it executable
chmod +x "$BINARY_NAME"

# Move to /usr/local/bin (use sudo if needed)
if [ -w "$INSTALL_DIR" ]; then
    mv "$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
else
    sudo mv "$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
fi

echo "✅ Installed $BINARY_NAME to $INSTALL_DIR"
echo "   Run '$BINARY_NAME --help' to get started."
