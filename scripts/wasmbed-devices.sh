#!/bin/bash

# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

set -e

# Wasmbed Platform - Device Management Script
# This script provides device management operations

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
    echo "Wasmbed Device Management Script"
    echo ""
    echo "Usage: $0 [COMMAND] [OPTIONS]"
    echo ""
    echo "Commands:"
    echo "  list                    List all devices"
    echo "  create <name>           Create a new device"
    echo "  delete <name>           Delete a device"
    echo "  status <name>           Show device status"
    echo "  enroll <name>           Force device enrollment"
    echo "  connect <name>          Simulate device connection"
    echo "  disconnect <name>       Simulate device disconnection"
    echo "  logs <name>             Show device logs"
    echo ""
    echo "Examples:"
    echo "  $0 list"
    echo "  $0 create my-device"
    echo "  $0 status test-device-1"
    echo "  $0 enroll test-device-1"
}

list_devices() {
    print_status "INFO" "Listing all devices..."
    kubectl get devices -n wasmbed -o wide
}

create_device() {
    local device_name=$1
    if [ -z "$device_name" ]; then
        print_status "ERROR" "Device name is required"
        exit 1
    fi
    
    print_status "INFO" "Creating device: $device_name"
    
    cat <<EOF | kubectl apply -f -
apiVersion: wasmbed.github.io/v0
kind: Device
metadata:
  name: $device_name
  namespace: wasmbed
spec:
  publicKey: "device-$device_name-public-key-$(date +%s)"
EOF
    
    print_status "SUCCESS" "Device $device_name created successfully"
}

delete_device() {
    local device_name=$1
    if [ -z "$device_name" ]; then
        print_status "ERROR" "Device name is required"
        exit 1
    fi
    
    print_status "INFO" "Deleting device: $device_name"
    kubectl delete device "$device_name" -n wasmbed
    print_status "SUCCESS" "Device $device_name deleted successfully"
}

show_device_status() {
    local device_name=$1
    if [ -z "$device_name" ]; then
        print_status "ERROR" "Device name is required"
        exit 1
    fi
    
    print_status "INFO" "Device status for: $device_name"
    kubectl get device "$device_name" -n wasmbed -o yaml
}

enroll_device() {
    local device_name=$1
    if [ -z "$device_name" ]; then
        print_status "ERROR" "Device name is required"
        exit 1
    fi
    
    print_status "INFO" "Forcing enrollment for device: $device_name"
    
    # Patch device to trigger enrollment
    kubectl patch device "$device_name" -n wasmbed --type='merge' -p='{"status":{"phase":"Pending"}}'
    
    print_status "SUCCESS" "Device $device_name enrollment triggered"
}

connect_device() {
    local device_name=$1
    if [ -z "$device_name" ]; then
        print_status "ERROR" "Device name is required"
        exit 1
    fi
    
    print_status "INFO" "Simulating connection for device: $device_name"
    
    # Patch device to connected state
    kubectl patch device "$device_name" -n wasmbed --type='merge' -p='{"status":{"phase":"Connected","connectedSince":"'$(date -u +%Y-%m-%dT%H:%M:%SZ)'","lastHeartbeat":"'$(date -u +%Y-%m-%dT%H:%M:%SZ)'"}}'
    
    print_status "SUCCESS" "Device $device_name connection simulated"
}

disconnect_device() {
    local device_name=$1
    if [ -z "$device_name" ]; then
        print_status "ERROR" "Device name is required"
        exit 1
    fi
    
    print_status "INFO" "Simulating disconnection for device: $device_name"
    
    # Patch device to disconnected state
    kubectl patch device "$device_name" -n wasmbed --type='merge' -p='{"status":{"phase":"Disconnected"}}'
    
    print_status "SUCCESS" "Device $device_name disconnection simulated"
}

show_device_logs() {
    local device_name=$1
    if [ -z "$device_name" ]; then
        print_status "ERROR" "Device name is required"
        exit 1
    fi
    
    print_status "INFO" "Device logs for: $device_name"
    
    # Get device events
    kubectl get events -n wasmbed --field-selector involvedObject.name="$device_name" --sort-by='.lastTimestamp'
}

# Main script logic
case "$1" in
    "list")
        list_devices
        ;;
    "create")
        create_device "$2"
        ;;
    "delete")
        delete_device "$2"
        ;;
    "status")
        show_device_status "$2"
        ;;
    "enroll")
        enroll_device "$2"
        ;;
    "connect")
        connect_device "$2"
        ;;
    "disconnect")
        disconnect_device "$2"
        ;;
    "logs")
        show_device_logs "$2"
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
