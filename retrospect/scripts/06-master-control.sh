#!/bin/bash

# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

set -e

# Wasmbed Platform - Main Management Script
# This script provides a unified interface for all platform operations

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

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

show_help() {
    echo "Wasmbed Platform Management Script"
    echo ""
    echo "Usage: $0 [COMMAND] [OPTIONS]"
    echo ""
    echo "Essential Commands:"
    echo "  clean       Clean up all resources (k3d cluster, processes, build artifacts)"
    echo "  build       Build all Rust components"
    echo "  deploy      Deploy complete platform (k3d cluster + services)"
    echo "  stop        Stop all services gracefully"
    echo "  status      Check system status and health"
    echo "  restart     Stop and restart all services"
    echo ""
    echo "Examples:"
    echo "  $0 deploy    # Deploy the complete platform"
    echo "  $0 status    # Check if everything is running"
    echo "  $0 stop      # Stop all services"
    echo "  $0 clean     # Clean everything and start fresh"
    echo ""
    echo "Quick Start:"
    echo "  1. $0 deploy    # Deploy the platform"
    echo "  2. $0 status    # Verify deployment"
    echo "  3. Open http://localhost:30453 in browser"
    echo ""
}

# Main command dispatcher
case "${1:-help}" in
    "clean")
        print_status "INFO" "Cleaning Wasmbed platform..."
        "$SCRIPT_DIR/00-cleanup-environment.sh"
        ;;
    "build")
        print_status "INFO" "Building Wasmbed platform..."
        "$SCRIPT_DIR/01-build-components.sh"
        ;;
    "deploy")
        print_status "INFO" "Deploying Wasmbed platform..."
        "$SCRIPT_DIR/02-deploy-infrastructure.sh"
        ;;
    "stop")
        print_status "INFO" "Stopping Wasmbed platform..."
        "$SCRIPT_DIR/05-stop-services.sh"
        ;;
    "status")
        print_status "INFO" "Checking Wasmbed platform status..."
        "$SCRIPT_DIR/03-check-system-status.sh"
        ;;
    "restart")
        print_status "INFO" "Restarting Wasmbed platform..."
        "$SCRIPT_DIR/05-stop-services.sh"
        sleep 2
        "$SCRIPT_DIR/02-deploy-infrastructure.sh"
        ;;
    "help"|"-h"|"--help")
        show_help
        ;;
    *)
        print_status "ERROR" "Unknown command: $1"
        echo ""
        show_help
        exit 1
        ;;
esac
