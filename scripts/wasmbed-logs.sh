#!/bin/bash

# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

set -e

# Wasmbed Platform - Log Management Script
# This script manages logs and debugging for the Wasmbed platform

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

show_help() {
    echo "Wasmbed Log Management Script"
    echo ""
    echo "Usage: $0 [COMMAND] [OPTIONS]"
    echo ""
    echo "Commands:"
    echo "  show                    Show recent logs"
    echo "  follow                  Follow logs in real-time"
    echo "  errors                  Show only error logs"
    echo "  events                  Show Kubernetes events"
    echo "  controller <name>       Show controller logs"
    echo "  service <name>           Show service logs"
    echo "  debug                   Enable debug logging"
    echo "  clean                   Clean up log files"
    echo ""
    echo "Examples:"
    echo "  $0 show"
    echo "  $0 follow"
    echo "  $0 controller device"
    echo "  $0 service gateway"
    echo "  $0 debug"
}

show_logs() {
    print_status "INFO" "Showing recent logs..."
    echo ""
    
    # Show Kubernetes events
    print_status "INFO" "Recent Kubernetes Events:"
    kubectl get events -n wasmbed --sort-by='.lastTimestamp' | tail -20
    echo ""
    
    # Show system logs if available
    if [ -f "/var/log/wasmbed.log" ]; then
        print_status "INFO" "System Logs:"
        tail -20 /var/log/wasmbed.log
    fi
}

follow_logs() {
    print_status "INFO" "Following logs in real-time (Press Ctrl+C to stop)..."
    echo ""
    
    # Follow Kubernetes events
    kubectl get events -n wasmbed --sort-by='.lastTimestamp' -w
}

show_errors() {
    print_status "INFO" "Showing error logs..."
    echo ""
    
    # Show error events
    kubectl get events -n wasmbed --sort-by='.lastTimestamp' | grep -i error
    echo ""
    
    # Show failed pods
    print_status "INFO" "Failed Pods:"
    kubectl get pods -n wasmbed --field-selector=status.phase=Failed
}

show_events() {
    print_status "INFO" "Showing Kubernetes events..."
    echo ""
    
    kubectl get events -n wasmbed --sort-by='.lastTimestamp'
}

show_controller_logs() {
    local controller_name=$1
    if [ -z "$controller_name" ]; then
        print_status "ERROR" "Controller name is required"
        echo "Available controllers: device, application, gateway"
        exit 1
    fi
    
    print_status "INFO" "Showing logs for $controller_name controller..."
    echo ""
    
    case $controller_name in
        "device")
            if pgrep -f "wasmbed-device-controller" >/dev/null; then
                print_status "SUCCESS" "Device Controller is running"
                # In a real deployment, we would show actual logs
                print_status "INFO" "Device Controller logs would be shown here"
            else
                print_status "ERROR" "Device Controller is not running"
            fi
            ;;
        "application")
            if pgrep -f "wasmbed-application-controller" >/dev/null; then
                print_status "SUCCESS" "Application Controller is running"
                print_status "INFO" "Application Controller logs would be shown here"
            else
                print_status "ERROR" "Application Controller is not running"
            fi
            ;;
        "gateway")
            if pgrep -f "wasmbed-gateway-controller" >/dev/null; then
                print_status "SUCCESS" "Gateway Controller is running"
                print_status "INFO" "Gateway Controller logs would be shown here"
            else
                print_status "ERROR" "Gateway Controller is not running"
            fi
            ;;
        *)
            print_status "ERROR" "Unknown controller: $controller_name"
            echo "Available controllers: device, application, gateway"
            exit 1
            ;;
    esac
}

show_service_logs() {
    local service_name=$1
    if [ -z "$service_name" ]; then
        print_status "ERROR" "Service name is required"
        echo "Available services: gateway, infrastructure, dashboard"
        exit 1
    fi
    
    print_status "INFO" "Showing logs for $service_name service..."
    echo ""
    
    case $service_name in
        "gateway")
            if pgrep -f "wasmbed-gateway" >/dev/null; then
                print_status "SUCCESS" "Gateway service is running"
                print_status "INFO" "Gateway service logs would be shown here"
            else
                print_status "ERROR" "Gateway service is not running"
            fi
            ;;
        "infrastructure")
            if pgrep -f "wasmbed-infrastructure" >/dev/null; then
                print_status "SUCCESS" "Infrastructure service is running"
                print_status "INFO" "Infrastructure service logs would be shown here"
            else
                print_status "ERROR" "Infrastructure service is not running"
            fi
            ;;
        "dashboard")
            if pgrep -f "wasmbed-dashboard" >/dev/null; then
                print_status "SUCCESS" "Dashboard service is running"
                print_status "INFO" "Dashboard service logs would be shown here"
            else
                print_status "ERROR" "Dashboard service is not running"
            fi
            ;;
        *)
            print_status "ERROR" "Unknown service: $service_name"
            echo "Available services: gateway, infrastructure, dashboard"
            exit 1
            ;;
    esac
}

enable_debug_logging() {
    print_status "INFO" "Enabling debug logging..."
    echo ""
    
    # Set environment variables for debug logging
    export RUST_LOG=debug
    export WASMBED_LOG_LEVEL=debug
    
    print_status "SUCCESS" "Debug logging enabled"
    print_status "INFO" "Restart services to apply debug logging"
    print_status "INFO" "Environment variables set:"
    echo "  RUST_LOG=debug"
    echo "  WASMBED_LOG_LEVEL=debug"
}

clean_logs() {
    print_status "INFO" "Cleaning up log files..."
    
    # Clean up log files
    find . -name "*.log" -type f -delete 2>/dev/null || true
    
    # Clean up journal logs if available
    if command -v journalctl >/dev/null 2>&1; then
        print_status "INFO" "Cleaning journal logs..."
        journalctl --vacuum-time=1d 2>/dev/null || true
    fi
    
    print_status "SUCCESS" "Log files cleaned up"
}

# Main script logic
case "$1" in
    "show")
        show_logs
        ;;
    "follow")
        follow_logs
        ;;
    "errors")
        show_errors
        ;;
    "events")
        show_events
        ;;
    "controller")
        show_controller_logs "$2"
        ;;
    "service")
        show_service_logs "$2"
        ;;
    "debug")
        enable_debug_logging
        ;;
    "clean")
        clean_logs
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
