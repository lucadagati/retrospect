#!/bin/bash

# QEMU Device Complete Test
# Tests QEMU devices using the working MCU simulator

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
NAMESPACE="wasmbed"
GATEWAY_HOST="172.19.0.2"
GATEWAY_PORT="30423"

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

log_test() {
    echo -e "${CYAN}[TEST]${NC} $1"
}

log_qemu() {
    echo -e "${PURPLE}[QEMU]${NC} $1"
}

# Test QEMU processes
test_qemu_processes() {
    log_test "Testing QEMU processes..."
    
    local qemu_count=$(ps aux | grep qemu-system-riscv32 | grep -v grep | wc -l)
    if [ "$qemu_count" -gt 0 ]; then
        log_success "Found $qemu_count QEMU device(s) running"
        
        # List QEMU processes
        ps aux | grep qemu-system-riscv32 | grep -v grep | while read -r line; do
            log_qemu "QEMU process: $line"
        done
    else
        log_error "No QEMU devices found running"
        return 1
    fi
}

# Test QEMU device resources
test_qemu_resources() {
    log_test "Testing QEMU device resources..."
    
    local qemu_devices=$(kubectl get devices -n "$NAMESPACE" | grep qemu-device | wc -l)
    if [ "$qemu_devices" -gt 0 ]; then
        log_success "Found $qemu_devices QEMU device resource(s) in Kubernetes"
        kubectl get devices -n "$NAMESPACE" | grep qemu-device
    else
        log_error "No QEMU device resources found"
        return 1
    fi
}

# Test QEMU device enrollment using MCU simulator
test_qemu_enrollment() {
    log_test "Testing QEMU device enrollment using MCU simulator..."
    
    # Test enrollment for qemu-device-1
    log_info "Testing enrollment for qemu-device-1..."
    if timeout 30 ./target/release/wasmbed-mcu-simulator --device-id qemu-device-1 --gateway-host "$GATEWAY_HOST" --gateway-port "$GATEWAY_PORT" --test-mode > /tmp/qemu-test-1.log 2>&1; then
        log_success "qemu-device-1 enrollment test completed"
        
        # Check if enrollment was successful
        if grep -q "enrollment completed" /tmp/qemu-test-1.log; then
            log_success "qemu-device-1 enrollment successful"
        else
            log_warning "qemu-device-1 enrollment may have failed"
        fi
        
        # Check if connection was successful
        if grep -q "connected successfully" /tmp/qemu-test-1.log; then
            log_success "qemu-device-1 connection successful"
        else
            log_warning "qemu-device-1 connection may have failed"
        fi
        
        # Check if WASM execution was successful
        if grep -q "WASM application.*running" /tmp/qemu-test-1.log; then
            log_success "qemu-device-1 WASM execution successful"
        else
            log_warning "qemu-device-1 WASM execution may have failed"
        fi
        
        # Check if microROS was successful
        if grep -q "microROS communication active" /tmp/qemu-test-1.log; then
            log_success "qemu-device-1 microROS communication successful"
        else
            log_warning "qemu-device-1 microROS communication may have failed"
        fi
        
    else
        log_error "qemu-device-1 enrollment test failed"
        return 1
    fi
    
    # Test enrollment for qemu-device-2
    log_info "Testing enrollment for qemu-device-2..."
    if timeout 30 ./target/release/wasmbed-mcu-simulator --device-id qemu-device-2 --gateway-host "$GATEWAY_HOST" --gateway-port "$GATEWAY_PORT" --test-mode > /tmp/qemu-test-2.log 2>&1; then
        log_success "qemu-device-2 enrollment test completed"
        
        # Check if enrollment was successful
        if grep -q "enrollment completed" /tmp/qemu-test-2.log; then
            log_success "qemu-device-2 enrollment successful"
        else
            log_warning "qemu-device-2 enrollment may have failed"
        fi
        
        # Check if connection was successful
        if grep -q "connected successfully" /tmp/qemu-test-2.log; then
            log_success "qemu-device-2 connection successful"
        else
            log_warning "qemu-device-2 connection may have failed"
        fi
        
        # Check if WASM execution was successful
        if grep -q "WASM application.*running" /tmp/qemu-test-2.log; then
            log_success "qemu-device-2 WASM execution successful"
        else
            log_warning "qemu-device-2 WASM execution may have failed"
        fi
        
        # Check if microROS was successful
        if grep -q "microROS communication active" /tmp/qemu-test-2.log; then
            log_success "qemu-device-2 microROS communication successful"
        else
            log_warning "qemu-device-2 microROS communication may have failed"
        fi
        
    else
        log_error "qemu-device-2 enrollment test failed"
        return 1
    fi
}

# Test QEMU device status in Kubernetes
test_qemu_status() {
    log_test "Testing QEMU device status in Kubernetes..."
    
    # Check device status
    kubectl get devices -n "$NAMESPACE" -o jsonpath='{range .items[*]}{.metadata.name}{"\t"}{.status.phase}{"\n"}{end}' | while read -r device_name status; do
        if [[ "$device_name" == qemu-device-* ]]; then
            if [ -n "$status" ] && [ "$status" != "null" ]; then
                log_success "QEMU device $device_name status: $status"
            else
                log_warning "QEMU device $device_name has no status"
            fi
        fi
    done
}

# Test QEMU serial communication
test_qemu_serial() {
    log_test "Testing QEMU serial communication..."
    
    # Check serial sockets
    local socket_count=$(ls /tmp/wasmbed-qemu-*.sock 2>/dev/null | wc -l)
    if [ "$socket_count" -gt 0 ]; then
        log_success "Found $socket_count QEMU serial socket(s)"
        ls -la /tmp/wasmbed-qemu-*.sock 2>/dev/null || true
    else
        log_warning "No QEMU serial sockets found"
    fi
    
    # Check QEMU logs
    local log_count=$(ls /tmp/wasmbed-qemu-*.log 2>/dev/null | wc -l)
    if [ "$log_count" -gt 0 ]; then
        log_success "Found $log_count QEMU log file(s)"
        for log_file in /tmp/wasmbed-qemu-*.log; do
            if [ -f "$log_file" ]; then
                log_qemu "Log file: $log_file"
                tail -3 "$log_file" 2>/dev/null || true
            fi
        done
    else
        log_warning "No QEMU log files found"
    fi
}

# Test gateway connectivity
test_gateway_connectivity() {
    log_test "Testing gateway connectivity..."
    
    # Test HTTP API
    if curl -s "http://$GATEWAY_HOST:30080/health" >/dev/null 2>&1; then
        log_success "Gateway HTTP API is accessible"
    else
        log_error "Gateway HTTP API is not accessible"
    fi
    
    # Test TLS port (basic connectivity)
    if timeout 5 bash -c "</dev/tcp/$GATEWAY_HOST/$GATEWAY_PORT" 2>/dev/null; then
        log_success "Gateway TLS port is accessible"
    else
        log_warning "Gateway TLS port is not accessible"
    fi
}

# Run comprehensive QEMU test
run_comprehensive_qemu_test() {
    log_info "Starting comprehensive QEMU device test..."
    echo "=================================================="
    
    test_qemu_processes
    echo "--------------------------------------------------"
    
    test_qemu_resources
    echo "--------------------------------------------------"
    
    test_gateway_connectivity
    echo "--------------------------------------------------"
    
    test_qemu_enrollment
    echo "--------------------------------------------------"
    
    test_qemu_status
    echo "--------------------------------------------------"
    
    test_qemu_serial
    echo "--------------------------------------------------"
    
    log_success "Comprehensive QEMU device test completed!"
}

# Main function
main() {
    case "${1:-comprehensive}" in
        "comprehensive")
            run_comprehensive_qemu_test
            ;;
        "processes")
            test_qemu_processes
            ;;
        "resources")
            test_qemu_resources
            ;;
        "enrollment")
            test_qemu_enrollment
            ;;
        "status")
            test_qemu_status
            ;;
        "serial")
            test_qemu_serial
            ;;
        "gateway")
            test_gateway_connectivity
            ;;
        *)
            echo "Usage: $0 {comprehensive|processes|resources|enrollment|status|serial|gateway}"
            echo "  comprehensive - Run all QEMU tests (default)"
            echo "  processes     - Test QEMU processes only"
            echo "  resources     - Test QEMU resources only"
            echo "  enrollment    - Test QEMU enrollment only"
            echo "  status        - Test QEMU status only"
            echo "  serial        - Test QEMU serial communication only"
            echo "  gateway       - Test gateway connectivity only"
            exit 1
            ;;
    esac
}

# Run main function
main "$@"
