#!/bin/bash

# Cortex 1.1.2 Global Installer
# Purpose: Auto-install Cortex to the user's PATH

set -e

# ANSI Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}Cortex 1.1.2 Installer Starting...${NC}"

# 1. Dependency Checks
command -v git >/dev/null 2>&1 || { echo -e >&2 "${RED}Error: git is required but not installed.${NC}"; exit 1; }
command -v cargo >/dev/null 2>&1 || { echo -e >&2 "${RED}Error: rust/cargo is required but not installed. Visit https://rustup.rs${NC}"; exit 1; }

# 2. Setup Temporary Workspace
TMP_DIR=$(mktemp -d)
cd "$TMP_DIR"

echo -e "${BLUE}Cloning Cortex repository...${NC}"
git clone --depth 1 https://github.com/Dopove/Cortex.git .

# 3. Build
echo -e "${BLUE}Building Cortex 1.1.2 (Release)...${NC}"
cd rust
cargo build --release

# 4. Installation
BINARY_PATH="target/release/cortex"
INSTALL_DIR="$HOME/.local/bin"

if [ ! -d "$INSTALL_DIR" ]; then
    INSTALL_DIR="/usr/local/bin"
fi

echo -e "${BLUE}Installing binary to $INSTALL_DIR...${NC}"

if [ "$INSTALL_DIR" == "/usr/local/bin" ]; then
    sudo cp "$BINARY_PATH" "$INSTALL_DIR/cortex"
else
    mkdir -p "$INSTALL_DIR"
    cp "$BINARY_PATH" "$INSTALL_DIR/cortex"
fi

# 5. Initialization
echo -e "${GREEN}Cortex installed successfully!${NC}"
echo -e "${BLUE}Running 'cortex init'...${NC}"
"$INSTALL_DIR/cortex" init

echo -e "\n${GREEN}========================================${NC}"
echo -e "${GREEN}Cortex 1.1.2 is now available globally.${NC}"
echo -e "Try running: ${BLUE}cortex --version${NC}"
echo -e "${GREEN}========================================${NC}"

# Cleanup
rm -rf "$TMP_DIR"
