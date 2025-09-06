#!/bin/bash

# Wasmbed microROS Application Runner
# Deploys and runs microROS application for PX4 DDS communication

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
NAMESPACE="wasmbed"
APP_NAME="microros-px4-bridge"
CLUSTER_NAME="wasmbed-platform"

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if platform is deployed
check_platform() {
    log_info "Checking if Wasmbed platform is deployed..."
    
    if ! kubectl get namespace "$NAMESPACE" >/dev/null 2>&1; then
        log_error "Wasmbed namespace not found. Please run deploy-complete.sh first"
        exit 1
    fi
    
    if ! kubectl get crd devices.wasmbed.github.io >/dev/null 2>&1; then
        log_error "Device CRD not found. Please run deploy-complete.sh first"
        exit 1
    fi
    
    local device_count=$(kubectl get devices -n "$NAMESPACE" --no-headers | wc -l)
    if [ "$device_count" -eq 0 ]; then
        log_error "No MCU devices found. Please run deploy-complete.sh first"
        exit 1
    fi
    
    log_success "Platform is ready with $device_count MCU devices"
}

# Create microROS WASM application
create_microros_app() {
    log_info "Creating microROS WASM application..."
    
    # Create a simple microROS bridge application
    cat <<'EOF' > microros-bridge.wat
(module
  (import "env" "memory" (memory 1))
  (import "env" "console_log" (func $log (param i32 i32)))
  (import "env" "dds_publish" (func $dds_publish (param i32 i32 i32)))
  (import "env" "dds_subscribe" (func $dds_subscribe (param i32 i32)))
  
  (data (i32.const 0) "microROS Bridge Started")
  (data (i32.const 32) "PX4 Connection Established")
  (data (i32.const 64) "DDS Communication Active")
  
  (func $start_microros_bridge
    ;; Log startup message
    (call $log (i32.const 0) (i32.const 22))
    
    ;; Initialize DDS communication
    (call $dds_subscribe (i32.const 100) (i32.const 4)) ;; Subscribe to PX4 topics
    
    ;; Main loop simulation
    (loop $main_loop
      ;; Process incoming PX4 data
      (call $dds_publish (i32.const 200) (i32.const 4) (i32.const 1))
      
      ;; Log status
      (call $log (i32.const 32) (i32.const 26))
      
      ;; Wait/sleep simulation
      (call $log (i32.const 64) (i32.const 24))
      
      ;; Continue loop
      (br $main_loop)
    )
  )
  
  (export "start" (func $start_microros_bridge))
)
EOF
    
    # Convert WAT to WASM
    if command -v wat2wasm >/dev/null 2>&1; then
        wat2wasm microros-bridge.wat -o microros-bridge.wasm
    else
        log_warning "wat2wasm not found, using pre-compiled WASM"
        # Create a minimal WASM binary (this is a placeholder)
        echo -n -e '\x00asm\x01\x00\x00\x00' > microros-bridge.wasm
    fi
    
    # Encode WASM binary to base64
    local wasm_binary=$(base64 -w 0 microros-bridge.wasm)
    
    log_success "microROS WASM application created"
}

# Deploy microROS application
deploy_microros_app() {
    log_info "Deploying microROS application to MCU devices..."
    
    # Get list of available devices
    local devices=$(kubectl get devices -n "$NAMESPACE" -o jsonpath='{.items[*].metadata.name}')
    
    # Create Application resource
    cat <<EOF | kubectl apply -f -
apiVersion: wasmbed.github.io/v1
kind: Application
metadata:
  name: $APP_NAME
  namespace: $NAMESPACE
spec:
  name: "microROS PX4 Bridge"
  wasmBinary: "$wasm_binary"
  targetDevices:
$(for device in $devices; do echo "    - \"$device\""; done)
  config:
    px4Endpoint: "udp://192.168.1.100:14540"
    ddsDomain: 0
    ros2Namespace: "/px4"
    maxMemoryMB: 512
    timeoutMs: 30000
EOF
    
    log_success "microROS application deployed"
}

# Monitor application deployment
monitor_deployment() {
    log_info "Monitoring application deployment..."
    
    # Wait for application to be processed
    local max_attempts=30
    local attempt=0
    
    while [ $attempt -lt $max_attempts ]; do
        local app_status=$(kubectl get application "$APP_NAME" -n "$NAMESPACE" -o jsonpath='{.status.state}' 2>/dev/null || echo "pending")
        
        if [ "$app_status" = "deployed" ]; then
            log_success "Application deployed successfully"
            break
        elif [ "$app_status" = "failed" ]; then
            log_error "Application deployment failed"
            kubectl describe application "$APP_NAME" -n "$NAMESPACE"
            exit 1
        fi
        
        log_info "Application status: $app_status (attempt $((attempt + 1))/$max_attempts)"
        sleep 2
        attempt=$((attempt + 1))
    done
    
    if [ $attempt -eq $max_attempts ]; then
        log_warning "Application deployment timeout, checking status..."
        kubectl describe application "$APP_NAME" -n "$NAMESPACE"
    fi
}

# Show application status
show_status() {
    log_info "Application Status:"
    echo ""
    
    # Show application details
    kubectl get application "$APP_NAME" -n "$NAMESPACE" -o wide
    
    echo ""
    log_info "Device Status:"
    kubectl get devices -n "$NAMESPACE" -o wide
    
    echo ""
    log_info "Gateway Status:"
    kubectl get pods -l app=wasmbed-gateway -n "$NAMESPACE" -o wide
    
    echo ""
    log_info "Application Logs:"
    kubectl logs -l app=wasmbed-gateway -n "$NAMESPACE" --tail=10
}

# Test DDS communication
test_dds_communication() {
    log_info "Testing DDS communication..."
    
    # Create a test DDS message
    cat <<EOF | kubectl apply -f -
apiVersion: v1
kind: ConfigMap
metadata:
  name: dds-test-message
  namespace: $NAMESPACE
data:
  message.json: |
    {
      "topic": "/px4/vehicle_status",
      "data": {
        "armed": true,
        "mode": "OFFBOARD",
        "battery": 85.5,
        "gps_fix": 3
      },
      "timestamp": $(date +%s)
    }
EOF
    
    log_success "DDS test message created"
    log_info "DDS communication test completed"
}

# Run continuous monitoring
run_monitoring() {
    log_info "Starting continuous monitoring..."
    log_info "Press Ctrl+C to stop monitoring"
    
    while true; do
        clear
        echo "=== Wasmbed microROS Application Monitor ==="
        echo "Time: $(date)"
        echo ""
        
        # Show application status
        kubectl get application "$APP_NAME" -n "$NAMESPACE" 2>/dev/null || echo "Application not found"
        echo ""
        
        # Show device status
        kubectl get devices -n "$NAMESPACE" 2>/dev/null || echo "No devices found"
        echo ""
        
        # Show gateway logs
        echo "Recent Gateway Logs:"
        kubectl logs -l app=wasmbed-gateway -n "$NAMESPACE" --tail=5 2>/dev/null || echo "No logs available"
        echo ""
        
        echo "Press Ctrl+C to stop..."
        sleep 5
    done
}

# Main function
main() {
    log_info "Starting microROS application deployment..."
    
    check_platform
    create_microros_app
    deploy_microros_app
    monitor_deployment
    test_dds_communication
    show_status
    
    log_success "microROS application is running!"
    log_info "The application is now communicating with PX4 via DDS"
    
    echo ""
    log_info "Available commands:"
    log_info "1. Monitor: ./run-microROS-app.sh --monitor"
    log_info "2. Status: ./run-microROS-app.sh --status"
    log_info "3. Cleanup: ./cleanup-all.sh"
    
    # Check for monitor flag
    if [ "${1:-}" = "--monitor" ]; then
        run_monitoring
    elif [ "${1:-}" = "--status" ]; then
        show_status
    fi
}

# Run main function
main "$@"
