#!/bin/bash

# Cortex Professional Global Installer
# Purpose: High-speed binary installation of Cortex runtime

set -e

# ANSI Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}Cortex Professional Installer Starting...${NC}"

# 1. Dependency Checks
command -v curl >/dev/null 2>&1 || { echo -e >&2 "${RED}Error: curl is required.${NC}"; exit 1; }
command -v tar >/dev/null 2>&1 || { echo -e >&2 "${RED}Error: tar is required.${NC}"; exit 1; }

# Check for Python (runtime dependency)
if ! command -v python3 >/dev/null 2>&1; then
    echo -e "${YELLOW}Warning: python3 is not installed. Cortex requires Python 3.11.1+${NC}"
fi

# 2. Version Discovery
echo -e "${BLUE}Detecting latest version...${NC}"
# Use GitHub API to find latest release tag
LATEST_TAG=$(curl -s https://api.github.com/repos/Dopove/Cortex/releases/latest | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$LATEST_TAG" ]; then
    echo -e "${YELLOW}Warning: Could not detect latest version via API, falling back to v2.5.9${NC}"
    LATEST_TAG="v2.5.9"
fi

echo -e "${GREEN}Target Version: $LATEST_TAG${NC}"

# 3. Platform Detection
OS="$(uname -s)"
ARCH="$(uname -m)"
SUFFIX=""

case "$OS" in
    Linux)
        if [ "$ARCH" == "x86_64" ]; then
            # Detect distribution context for suffix alignment
            if grep -qi "ubuntu" /etc/os-release || grep -qi "debian" /etc/os-release; then
                SUFFIX="x64_linux_ubuntu"
            else
                SUFFIX="x64_linux_fedora_arch"
            fi
        else
            echo -e "${RED}Unsupported Linux architecture: $ARCH. Building from source required.${NC}"
            CAN_DOWNLOAD=false
        fi
        ;;
    Darwin)
        if [ "$ARCH" == "arm64" ]; then
            SUFFIX="arm64_macos"
        elif [ "$ARCH" == "x86_64" ]; then
            SUFFIX="x64_macos"
        fi
        ;;
    MINGW*|MSYS*|CYGWIN*)
        OS="Windows"
        SUFFIX="x64_windows"
        ;;
    *)
        echo -e "${RED}Unsupported OS: $OS${NC}"
        exit 1
        ;;
esac

if [ "$OS" == "Windows" ]; then
    EXT="zip"
else
    EXT="tar.gz"
fi

DOWNLOAD_URL="https://github.com/Dopove/Cortex/releases/download/${LATEST_TAG}/cortex_${LATEST_TAG}_${SUFFIX}.${EXT}"

echo -e "${BLUE}Platform: $OS ($ARCH)${NC}"
echo -e "${BLUE}Downloading binary from: $DOWNLOAD_URL${NC}"

TMP_DIR=$(mktemp -d)
cd "$TMP_DIR"

BINARY_ARCHIVE="cortex.${EXT}"

if curl -L --fail "$DOWNLOAD_URL" -o "$BINARY_ARCHIVE"; then
    if [ "$OS" == "Windows" ]; then
        unzip -q "$BINARY_ARCHIVE" -d "$TMP_DIR"
        BINARY_PATH=$(find "$TMP_DIR" -name "cortex*.exe" -type f | head -n 1)
    else
        tar -xzf "$BINARY_ARCHIVE" -C "$TMP_DIR"
        # Find the binary (it might be named cortex or cortex_vX.Y.Z_...)
        BINARY_PATH=$(find "$TMP_DIR" -name "cortex*" -type f -not -name "*.sh" -not -name "*.tar.gz" | head -n 1)
    fi

    if [ -z "$BINARY_PATH" ] || [ ! -f "$BINARY_PATH" ]; then
        echo -e "${RED}Error: Could not find cortex binary in extracted archive.${NC}"
        exit 1
    fi
else
    echo -e "${YELLOW}Binary not found for your platform. Falling back to source build...${NC}"
    # Source fallback logic
    command -v cargo >/dev/null 2>&1 || { echo -e >&2 "${RED}Error: rust/cargo is required to build from source.${NC}"; exit 1; }
    git clone --depth 1 https://github.com/Dopove/Cortex.git .
    cd rust && cargo build --release
    BINARY_PATH="target/release/cortex"
fi

# 5. Installation
INSTALL_DIR="$HOME/.local/bin"
if [ ! -d "$INSTALL_DIR" ]; then
    INSTALL_DIR="/usr/local/bin"
fi

echo -e "${BLUE}Installing binary to $INSTALL_DIR...${NC}"

if [ "$INSTALL_DIR" == "/usr/local/bin" ]; then
    if command -v sudo >/dev/null 2>&1; then
        sudo cp "$BINARY_PATH" "$INSTALL_DIR/cortex"
        sudo chmod +x "$INSTALL_DIR/cortex"
    else
        cp "$BINARY_PATH" "$INSTALL_DIR/cortex"
        chmod +x "$INSTALL_DIR/cortex"
    fi
else
    mkdir -p "$INSTALL_DIR"
    cp "$BINARY_PATH" "$INSTALL_DIR/cortex"
    chmod +x "$INSTALL_DIR/cortex"
fi

# 6. Initialization
echo -e "${GREEN}Cortex installed successfully!${NC}"
echo -e "${BLUE}Running 'cortex init'...${NC}"
"$INSTALL_DIR/cortex" init

echo -e "\n${GREEN}========================================${NC}"
echo -e "${GREEN}Cortex $LATEST_TAG is now available globally.${NC}"
echo -e "Try running: ${BLUE}cortex --version${NC}"
echo -e "${GREEN}========================================${NC}"

# Cleanup
rm -rf "$TMP_DIR"
