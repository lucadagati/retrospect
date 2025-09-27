#!/bin/bash

# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

set -e

# Wasmbed Platform - Application Management Script
# This script provides application management operations

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
    echo "Wasmbed Application Management Script"
    echo ""
    echo "Usage: $0 [COMMAND] [OPTIONS]"
    echo ""
    echo "Commands:"
    echo "  list                    List all applications"
    echo "  create <name>           Create a new application"
    echo "  delete <name>           Delete an application"
    echo "  status <name>           Show application status"
    echo "  deploy <name>           Force application deployment"
    echo "  stop <name>             Stop application deployment"
    echo "  restart <name>          Restart application deployment"
    echo "  logs <name>             Show application logs"
    echo ""
    echo "Examples:"
    echo "  $0 list"
    echo "  $0 create my-app"
    echo "  $0 status test-app-1"
    echo "  $0 deploy test-app-1"
}

list_applications() {
    print_status "INFO" "Listing all applications..."
    kubectl get applications -n wasmbed -o wide
}

create_application() {
    local app_name=$1
    if [ -z "$app_name" ]; then
        print_status "ERROR" "Application name is required"
        exit 1
    fi
    
    print_status "INFO" "Creating application: $app_name"
    
    # Create a simple WASM bytecode (base64 encoded "hello world")
    wasm_bytes=$(echo -n "hello world" | base64)
    
    cat <<EOF | kubectl apply -f -
apiVersion: wasmbed.github.io/v1alpha1
kind: Application
metadata:
  name: $app_name
  namespace: wasmbed
spec:
  name: "$app_name"
  description: "Test application created by script"
  wasmBytes: "$wasm_bytes"
  targetDevices:
    deviceNames: ["test-device-1"]
EOF
    
    print_status "SUCCESS" "Application $app_name created successfully"
}

delete_application() {
    local app_name=$1
    if [ -z "$app_name" ]; then
        print_status "ERROR" "Application name is required"
        exit 1
    fi
    
    print_status "INFO" "Deleting application: $app_name"
    kubectl delete application "$app_name" -n wasmbed
    print_status "SUCCESS" "Application $app_name deleted successfully"
}

show_application_status() {
    local app_name=$1
    if [ -z "$app_name" ]; then
        print_status "ERROR" "Application name is required"
        exit 1
    fi
    
    print_status "INFO" "Application status for: $app_name"
    kubectl get application "$app_name" -n wasmbed -o yaml
}

deploy_application() {
    local app_name=$1
    if [ -z "$app_name" ]; then
        print_status "ERROR" "Application name is required"
        exit 1
    fi
    
    print_status "INFO" "Forcing deployment for application: $app_name"
    
    # Patch application to trigger deployment
    kubectl patch application "$app_name" -n wasmbed --type='merge' -p='{"status":{"phase":"Creating"}}'
    
    print_status "SUCCESS" "Application $app_name deployment triggered"
}

stop_application() {
    local app_name=$1
    if [ -z "$app_name" ]; then
        print_status "ERROR" "Application name is required"
        exit 1
    fi
    
    print_status "INFO" "Stopping application: $app_name"
    
    # Patch application to stopping state
    kubectl patch application "$app_name" -n wasmbed --type='merge' -p='{"status":{"phase":"Stopping"}}'
    
    print_status "SUCCESS" "Application $app_name stop triggered"
}

restart_application() {
    local app_name=$1
    if [ -z "$app_name" ]; then
        print_status "ERROR" "Application name is required"
        exit 1
    fi
    
    print_status "INFO" "Restarting application: $app_name"
    
    # First stop, then restart
    kubectl patch application "$app_name" -n wasmbed --type='merge' -p='{"status":{"phase":"Stopping"}}'
    sleep 2
    kubectl patch application "$app_name" -n wasmbed --type='merge' -p='{"status":{"phase":"Creating"}}'
    
    print_status "SUCCESS" "Application $app_name restart triggered"
}

show_application_logs() {
    local app_name=$1
    if [ -z "$app_name" ]; then
        print_status "ERROR" "Application name is required"
        exit 1
    fi
    
    print_status "INFO" "Application logs for: $app_name"
    
    # Get application events
    kubectl get events -n wasmbed --field-selector involvedObject.name="$app_name" --sort-by='.lastTimestamp'
}

# Main script logic
case "$1" in
    "list")
        list_applications
        ;;
    "create")
        create_application "$2"
        ;;
    "delete")
        delete_application "$2"
        ;;
    "status")
        show_application_status "$2"
        ;;
    "deploy")
        deploy_application "$2"
        ;;
    "stop")
        stop_application "$2"
        ;;
    "restart")
        restart_application "$2"
        ;;
    "logs")
        show_application_logs "$2"
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
