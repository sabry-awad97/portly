#!/usr/bin/env bash
# Portly Installation Script for Unix-like Systems
# Downloads and installs the latest release from GitHub

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
GRAY='\033[0;90m'
NC='\033[0m' # No Color

# Configuration
REPO="sabry-awad97/portly"
VERSION="${1:-latest}"
INSTALL_DIR="${PORTLY_INSTALL_DIR:-$HOME/.local/bin}"

echo -e "${CYAN}Installing Portly...${NC}"

# Detect OS and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$ARCH" in
    x86_64) ARCH="x86_64" ;;
    aarch64|arm64) ARCH="aarch64" ;;
    *)
        echo -e "${RED}Error: Unsupported architecture: $ARCH${NC}"
        exit 1
        ;;
esac

echo -e "${GRAY}Detected: $OS $ARCH${NC}"

# Check dependencies
for cmd in curl tar; do
    if ! command -v $cmd &> /dev/null; then
        echo -e "${RED}Error: Required command '$cmd' not found${NC}"
        echo -e "${YELLOW}Please install $cmd and try again${NC}"
        exit 1
    fi
done

# Get release information
if [ "$VERSION" = "latest" ]; then
    RELEASE_URL="https://api.github.com/repos/$REPO/releases/latest"
else
    RELEASE_URL="https://api.github.com/repos/$REPO/releases/tags/v$VERSION"
fi

echo -e "${GRAY}Fetching release information...${NC}"
RELEASE_DATA=$(curl -sL "$RELEASE_URL" -H "User-Agent: portly-installer")

if [ -z "$RELEASE_DATA" ]; then
    echo -e "${RED}Error: Failed to fetch release information${NC}"
    echo -e "${YELLOW}Please check your internet connection and try again${NC}"
    echo -e "${GRAY}URL attempted: $RELEASE_URL${NC}"
    exit 1
fi

VERSION=$(echo "$RELEASE_DATA" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')
echo -e "${GREEN}Found version: $VERSION${NC}"

# Find matching asset
ASSET_PATTERN="$OS.*$ARCH.*tar.gz"
DOWNLOAD_URL=$(echo "$RELEASE_DATA" | grep "browser_download_url" | grep -i "$ASSET_PATTERN" | head -n 1 | cut -d '"' -f 4)

if [ -z "$DOWNLOAD_URL" ]; then
    echo -e "${RED}Error: No binary found for $OS $ARCH${NC}"
    echo -e "${YELLOW}Available assets:${NC}"
    echo "$RELEASE_DATA" | grep "browser_download_url" | cut -d '"' -f 4
    exit 1
fi

# Download and extract
TMP_DIR=$(mktemp -d)
trap "rm -rf $TMP_DIR" EXIT

echo -e "${GRAY}Downloading from $DOWNLOAD_URL...${NC}"
curl -sL "$DOWNLOAD_URL" -o "$TMP_DIR/portly.tar.gz"
echo -e "${GREEN}Downloaded successfully${NC}"

echo -e "${GRAY}Extracting binary...${NC}"
tar -xzf "$TMP_DIR/portly.tar.gz" -C "$TMP_DIR"
echo -e "${GREEN}Extracted successfully${NC}"

# Install binary
mkdir -p "$INSTALL_DIR"
BINARY_PATH="$INSTALL_DIR/portly"

if [ -f "$TMP_DIR/portly" ]; then
    mv "$TMP_DIR/portly" "$BINARY_PATH"
elif [ -f "$TMP_DIR/target/release/portly" ]; then
    mv "$TMP_DIR/target/release/portly" "$BINARY_PATH"
else
    echo -e "${RED}Error: Binary not found in archive${NC}"
    exit 1
fi

chmod +x "$BINARY_PATH"
echo -e "${GREEN}Installed to $BINARY_PATH${NC}"

# Check if in PATH
if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
    echo -e "${YELLOW}Warning: $INSTALL_DIR is not in your PATH${NC}"
    echo -e "${CYAN}Add this to your shell profile (~/.bashrc, ~/.zshrc, etc.):${NC}"
    echo -e "${GRAY}  export PATH=\"\$PATH:$INSTALL_DIR\"${NC}"
fi

# Verify installation
echo -e "\n${GRAY}Verifying installation...${NC}"
if [ -x "$BINARY_PATH" ]; then
    INSTALLED_VERSION=$("$BINARY_PATH" --version 2>&1 || echo "version check failed")
    echo -e "${GREEN}✓ Portly installed successfully!${NC}"
    echo -e "${GRAY}  Version: $INSTALLED_VERSION${NC}"
    echo -e "${GRAY}  Location: $BINARY_PATH${NC}"
    echo -e "\n${CYAN}Get started with: portly --help${NC}"
else
    echo -e "${RED}Error: Binary not executable${NC}"
    exit 1
fi
