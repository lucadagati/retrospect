#!/bin/bash

# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

# Quick Setup Script - Installa prerequisiti e crea virtual environment

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=== Quick Setup - Prerequisiti ===${NC}"
echo ""

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

# Check if running as root (for apt-get)
if [ "$EUID" -eq 0 ]; then
    SUDO=""
else
    SUDO="sudo"
fi

# Install system prerequisites
echo -e "${YELLOW}Installing system prerequisites...${NC}"
$SUDO apt-get update
$SUDO apt-get install -y python3-venv python3-pip cmake build-essential git wget
echo -e "${GREEN}✓ System prerequisites installed${NC}"
echo ""

# Create virtual environment
if [ ! -d ".venv" ]; then
    echo -e "${YELLOW}Creating virtual environment...${NC}"
    python3 -m venv .venv
    echo -e "${GREEN}✓ Virtual environment created${NC}"
else
    echo -e "${GREEN}✓ Virtual environment already exists${NC}"
fi

# Activate and install west
echo -e "${YELLOW}Installing west in virtual environment...${NC}"
source .venv/bin/activate
pip install --upgrade pip
pip install west
echo -e "${GREEN}✓ west installed${NC}"
echo ""

# Verify installation
echo -e "${BLUE}=== Verification ===${NC}"
west --version
python3 --version
echo ""

echo -e "${GREEN}=== Quick Setup Complete ===${NC}"
echo ""
echo "Virtual environment: $PROJECT_ROOT/.venv"
echo ""
echo "To activate virtual environment:"
echo "  source .venv/bin/activate"
echo ""
echo "Next steps:"
echo "  1. source .venv/bin/activate"
echo "  2. ./scripts/setup-zephyr-workspace.sh"
echo "  3. ./scripts/setup-wamr-integration.sh"


