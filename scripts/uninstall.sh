#!/usr/bin/env bash
# Portly Uninstallation Script for Unix-like Systems
# Removes Portly binary and optionally configuration files

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
GRAY='\033[0;90m'
NC='\033[0m' # No Color

# Configuration
INSTALL_DIR="${PORTLY_INSTALL_DIR:-$HOME/.local/bin}"
CONFIG_DIR="$HOME/.config/portly"
BINARY_PATH="$INSTALL_DIR/portly"

# Parse arguments
FORCE=false
REMOVE_CONFIG=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --force|-f)
            FORCE=true
            shift
            ;;
        --remove-config|-c)
            REMOVE_CONFIG=true
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --force, -f          Skip confirmation prompts"
            echo "  --remove-config, -c  Also remove configuration files"
            echo "  --help, -h           Show this help message"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

echo -e "${CYAN}Uninstalling Portly...${NC}"

# Check if installed
if [ ! -f "$BINARY_PATH" ]; then
    echo -e "${YELLOW}Portly is not installed at $BINARY_PATH${NC}"
    exit 0
fi

# Confirmation
if [ "$FORCE" = false ]; then
    echo -e "${YELLOW}Are you sure you want to uninstall Portly? (y/N)${NC}"
    read -r response
    if [ "$response" != "y" ] && [ "$response" != "Y" ]; then
        echo -e "${YELLOW}Uninstall cancelled${NC}"
        exit 0
    fi
fi

# Remove binary
echo -e "${GRAY}Removing binary from $BINARY_PATH...${NC}"
rm -f "$BINARY_PATH"
echo -e "${GREEN}✓ Binary removed${NC}"

# Remove config if requested
if [ "$REMOVE_CONFIG" = true ] && [ -d "$CONFIG_DIR" ]; then
    if [ "$FORCE" = false ]; then
        echo -e "${YELLOW}Also remove configuration files from $CONFIG_DIR? (y/N)${NC}"
        read -r response
        if [ "$response" = "y" ] || [ "$response" = "Y" ]; then
            rm -rf "$CONFIG_DIR"
            echo -e "${GREEN}✓ Configuration files removed${NC}"
        else
            echo -e "${GRAY}Configuration files kept${NC}"
        fi
    else
        rm -rf "$CONFIG_DIR"
        echo -e "${GREEN}✓ Configuration files removed${NC}"
    fi
fi

echo -e "\n${GREEN}✓ Portly uninstalled successfully!${NC}"
echo -e "${GRAY}Note: You may need to manually remove $INSTALL_DIR from your PATH${NC}"
