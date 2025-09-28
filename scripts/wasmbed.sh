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
    echo "Commands:"
    echo "  clean       Clean up all resources"
    echo "  build       Build all components"
    echo "  deploy      Deploy complete platform"
    echo "  stop        Stop all services"
    echo "  status      Check system status"
    echo "  restart     Restart all services"
    echo "  logs        Show system logs"
    echo "  test        Run platform tests"
    echo ""
    echo "Examples:"
    echo "  $0 deploy"
    echo "  $0 status"
    echo "  $0 stop"
    echo ""
}

# Main command dispatcher
case "${1:-help}" in
    "clean")
        print_status "INFO" "Cleaning Wasmbed platform..."
        "$SCRIPT_DIR/wasmbed-clean.sh"
        ;;
    "build")
        print_status "INFO" "Building Wasmbed platform..."
        "$SCRIPT_DIR/wasmbed-build.sh"
        ;;
    "deploy")
        print_status "INFO" "Deploying Wasmbed platform..."
        "$SCRIPT_DIR/wasmbed-deploy.sh"
        ;;
    "stop")
        print_status "INFO" "Stopping Wasmbed platform..."
        "$SCRIPT_DIR/wasmbed-stop.sh"
        ;;
    "status")
        print_status "INFO" "Checking Wasmbed platform status..."
        "$SCRIPT_DIR/wasmbed-status.sh"
        ;;
    "restart")
        print_status "INFO" "Restarting Wasmbed platform..."
        "$SCRIPT_DIR/wasmbed-stop.sh"
        sleep 2
        "$SCRIPT_DIR/wasmbed-deploy.sh"
        ;;
    "logs")
        print_status "INFO" "Showing Wasmbed platform logs..."
        "$SCRIPT_DIR/wasmbed-logs.sh" "${@:2}"
        ;;
    "test")
        print_status "INFO" "Running Wasmbed platform tests..."
        "$SCRIPT_DIR/wasmbed-test-complete.sh"
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
