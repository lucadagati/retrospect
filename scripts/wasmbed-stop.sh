#!/bin/bash

# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

set -e

# Wasmbed Platform - Stop Script
# This script stops all Wasmbed services

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    local status=$1
    local message=$2
    case $status in
        "SUCCESS")
            echo -e "${GREEN}✓ $message${NC}"
            ;;
        "ERROR")
            echo -e "${RED}✗ $message${NC}"
            ;;
        "WARNING")
            echo -e "${YELLOW}⚠ $message${NC}"
            ;;
        "INFO")
            echo -e "${BLUE}ℹ $message${NC}"
            ;;
    esac
}

print_status "INFO" "Stopping Wasmbed Platform services..."

# Stop services using saved PIDs
if [ -f ".wasmbed-pids" ]; then
    print_status "INFO" "Stopping services using saved PIDs..."
    PIDS=$(cat .wasmbed-pids)
    for pid in $PIDS; do
        if kill -0 "$pid" 2>/dev/null; then
            print_status "INFO" "Stopping process $pid"
            kill "$pid" 2>/dev/null || true
        fi
    done
    rm -f .wasmbed-pids
    print_status "SUCCESS" "Services stopped using saved PIDs"
else
    print_status "INFO" "No saved PIDs found, stopping by process name..."
fi

# Stop any remaining Wasmbed processes
print_status "INFO" "Stopping any remaining Wasmbed processes..."
pkill -f "wasmbed-" 2>/dev/null || true
pkill -f "wasmbed_" 2>/dev/null || true

# Stop k3d cluster
print_status "INFO" "Stopping k3d cluster..."
k3d cluster delete wasmbed-test 2>/dev/null || true

print_status "SUCCESS" "All Wasmbed services stopped successfully!"
print_status "INFO" "System is now clean"
