#!/bin/bash

# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

# Setup Zephyr Workspace for Wasmbed
# Installs Zephyr SDK, west, and initializes workspace

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=== Zephyr Workspace Setup ===${NC}"
echo ""

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

# Check Python
if ! command -v python3 &> /dev/null; then
    echo -e "${RED}✗ Python3 not found${NC}"
    exit 1
fi

# Install west if not available
if ! command -v west &> /dev/null; then
    echo -e "${YELLOW}⚠ Installing west...${NC}"
    
    # Try pipx first (recommended for system-wide Python)
    if command -v pipx &> /dev/null; then
        pipx install west
        echo -e "${GREEN}✓ west installed via pipx${NC}"
    else
        # Fallback: use virtual environment
        VENV_DIR="$PROJECT_ROOT/.venv"
        if [ ! -d "$VENV_DIR" ]; then
            python3 -m venv "$VENV_DIR"
        fi
        source "$VENV_DIR/bin/activate"
        pip install west
        echo -e "${GREEN}✓ west installed in virtual environment${NC}"
        echo "Note: Activate virtual environment with: source $VENV_DIR/bin/activate"
    fi
else
    echo -e "${GREEN}✓ west already installed${NC}"
fi

# Check Zephyr SDK
ZEPHYR_SDK_VERSION="0.16.5"
ZEPHYR_SDK_DIR="$PROJECT_ROOT/zephyr-sdk-$ZEPHYR_SDK_VERSION"
ZEPHYR_SDK_TAR="$PROJECT_ROOT/zephyr-sdk-${ZEPHYR_SDK_VERSION}_linux-x86_64.tar.xz"

if [ ! -d "$ZEPHYR_SDK_DIR" ]; then
    echo -e "${YELLOW}⚠ Zephyr SDK not found. Downloading...${NC}"
    
    if [ ! -f "$ZEPHYR_SDK_TAR" ]; then
        echo "Downloading Zephyr SDK $ZEPHYR_SDK_VERSION..."
        SDK_URL="https://github.com/zephyrproject-rtos/sdk-ng/releases/download/v${ZEPHYR_SDK_VERSION}/zephyr-sdk-${ZEPHYR_SDK_VERSION}_linux-x86_64.tar.xz"
        wget "$SDK_URL" -O "$ZEPHYR_SDK_TAR" || {
            echo -e "${RED}✗ Failed to download Zephyr SDK${NC}"
            exit 1
        }
    fi
    
    echo "Extracting Zephyr SDK..."
    tar xvf "$ZEPHYR_SDK_TAR"
    echo -e "${GREEN}✓ Zephyr SDK extracted${NC}"
else
    echo -e "${GREEN}✓ Zephyr SDK found${NC}"
fi

# Setup Zephyr SDK environment
export ZEPHYR_SDK_INSTALL_DIR="$ZEPHYR_SDK_DIR"
echo "ZEPHYR_SDK_INSTALL_DIR=$ZEPHYR_SDK_DIR" >> "$PROJECT_ROOT/.env.zephyr"

# Setup Zephyr workspace
ZEPHYR_WORKSPACE="$PROJECT_ROOT/zephyr-workspace"

if [ ! -d "$ZEPHYR_WORKSPACE" ]; then
    echo -e "${YELLOW}⚠ Initializing Zephyr workspace...${NC}"
    west init -m https://github.com/zephyrproject-rtos/zephyr --mr main "$ZEPHYR_WORKSPACE"
    cd "$ZEPHYR_WORKSPACE"
    west update
    echo -e "${GREEN}✓ Zephyr workspace initialized${NC}"
else
    echo -e "${GREEN}✓ Zephyr workspace already exists${NC}"
    cd "$ZEPHYR_WORKSPACE"
    echo "Updating Zephyr workspace..."
    west update
fi

# Set ZEPHYR_BASE
export ZEPHYR_BASE="$ZEPHYR_WORKSPACE/zephyr"
echo "ZEPHYR_BASE=$ZEPHYR_BASE" >> "$PROJECT_ROOT/.env.zephyr"

echo ""
echo -e "${GREEN}=== Zephyr Workspace Setup Complete ===${NC}"
echo ""
echo "Environment variables saved to: $PROJECT_ROOT/.env.zephyr"
echo ""
echo "To use Zephyr, source the environment:"
echo "  source $PROJECT_ROOT/.env.zephyr"
echo ""
echo "Or export manually:"
echo "  export ZEPHYR_SDK_INSTALL_DIR=$ZEPHYR_SDK_DIR"
echo "  export ZEPHYR_BASE=$ZEPHYR_BASE"

