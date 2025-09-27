#!/bin/bash

# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

set -e

# Wasmbed Platform - Main Management Script
# This script provides access to all Wasmbed management operations

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

print_banner() {
    echo -e "${CYAN}"
    echo "╔══════════════════════════════════════════════════════════════╗"
    echo "║                                                              ║"
    echo "║                    WASMBED PLATFORM                         ║"
    echo "║                  Management Console                         ║"
    echo "║                                                              ║"
    echo "╚══════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

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
        "HEADER")
            echo -e "${PURPLE}▶ $message${NC}"
            ;;
    esac
}

show_help() {
    print_banner
    echo ""
    echo "Wasmbed Platform Management Console"
    echo ""
    echo "Usage: $0 [COMMAND] [OPTIONS]"
    echo ""
    echo "System Commands:"
    echo "  clean                   Clean up all Wasmbed resources"
    echo "  build                   Build all Wasmbed components"
    echo "  deploy                  Deploy the complete platform"
    echo "  stop                    Stop all services"
    echo "  status                  Check system status"
    echo "  restart                 Restart all services"
    echo ""
    echo "Resource Management:"
    echo "  devices [cmd]           Manage devices (list, create, delete, etc.)"
    echo "  applications [cmd]      Manage applications (list, create, delete, etc.)"
    echo "  monitor [cmd]           Monitor system (overview, health, logs, etc.)"
    echo ""
    echo "Testing Commands:"
    echo "  test                    Run complete workflow tests"
    echo "  test-devices            Test device workflows"
    echo "  test-applications       Test application workflows"
    echo "  test-gateways           Test gateway workflows"
    echo ""
    echo "Examples:"
    echo "  $0 deploy               Deploy the complete platform"
    echo "  $0 devices list         List all devices"
    echo "  $0 applications create my-app"
    echo "  $0 monitor health       Check system health"
    echo "  $0 test                 Run all tests"
    echo ""
    echo "For detailed help on specific commands:"
    echo "  $0 devices help"
    echo "  $0 applications help"
    echo "  $0 monitor help"
}

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check prerequisites
check_prerequisites() {
    print_status "INFO" "Checking prerequisites..."
    
    local missing=()
    
    if ! command_exists k3d; then
        missing+=("k3d")
    fi
    
    if ! command_exists kubectl; then
        missing+=("kubectl")
    fi
    
    if ! command_exists cargo; then
        missing+=("cargo")
    fi
    
    if [ ${#missing[@]} -gt 0 ]; then
        print_status "ERROR" "Missing prerequisites: ${missing[*]}"
        print_status "INFO" "Please install the missing tools and try again"
        exit 1
    fi
    
    print_status "SUCCESS" "All prerequisites are available"
}

# System commands
clean_system() {
    print_status "HEADER" "Cleaning Wasmbed Platform..."
    ./scripts/clean.sh
}

build_system() {
    print_status "HEADER" "Building Wasmbed Platform..."
    ./scripts/build.sh
}

deploy_system() {
    print_status "HEADER" "Deploying Wasmbed Platform..."
    check_prerequisites
    ./scripts/deploy.sh
}

stop_system() {
    print_status "HEADER" "Stopping Wasmbed Platform..."
    ./scripts/stop.sh
}

show_status() {
    print_status "HEADER" "Checking Wasmbed Platform Status..."
    ./scripts/status.sh
}

restart_system() {
    print_status "HEADER" "Restarting Wasmbed Platform..."
    ./scripts/stop.sh
    sleep 2
    ./scripts/deploy.sh
}

# Resource management commands
manage_devices() {
    shift # Remove 'devices' from arguments
    ./scripts/devices.sh "$@"
}

manage_applications() {
    shift # Remove 'applications' from arguments
    ./scripts/applications.sh "$@"
}

manage_monitoring() {
    shift # Remove 'monitor' from arguments
    ./scripts/monitor.sh "$@"
}

# Testing commands
run_tests() {
    print_status "HEADER" "Running Complete Workflow Tests..."
    ./scripts/test-complete-workflows.sh
}

test_devices() {
    print_status "HEADER" "Testing Device Workflows..."
    print_status "INFO" "Creating test device..."
    ./scripts/devices.sh create test-device-$(date +%s)
    
    print_status "INFO" "Listing devices..."
    ./scripts/devices.sh list
    
    print_status "INFO" "Checking device status..."
    ./scripts/devices.sh status test-device-1
}

test_applications() {
    print_status "HEADER" "Testing Application Workflows..."
    print_status "INFO" "Creating test application..."
    ./scripts/applications.sh create test-app-$(date +%s)
    
    print_status "INFO" "Listing applications..."
    ./scripts/applications.sh list
    
    print_status "INFO" "Checking application status..."
    ./scripts/applications.sh status test-app-1
}

test_gateways() {
    print_status "HEADER" "Testing Gateway Workflows..."
    print_status "INFO" "Checking gateway status..."
    ./scripts/monitor.sh gateways
    
    print_status "INFO" "Checking gateway health..."
    ./scripts/monitor.sh health
}

# Main script logic
case "$1" in
    "clean")
        clean_system
        ;;
    "build")
        build_system
        ;;
    "deploy")
        deploy_system
        ;;
    "stop")
        stop_system
        ;;
    "status")
        show_status
        ;;
    "restart")
        restart_system
        ;;
    "devices")
        manage_devices "$@"
        ;;
    "applications")
        manage_applications "$@"
        ;;
    "monitor")
        manage_monitoring "$@"
        ;;
    "test")
        run_tests
        ;;
    "test-devices")
        test_devices
        ;;
    "test-applications")
        test_applications
        ;;
    "test-gateways")
        test_gateways
        ;;
    "help"|"-h"|"--help"|"")
        show_help
        ;;
    *)
        print_status "ERROR" "Unknown command: $1"
        echo ""
        show_help
        exit 1
        ;;
esac
