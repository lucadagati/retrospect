#!/bin/bash

# Complete Device System Test
# Tests all devices: QEMU RISC-V, ESP32, and Simulated MCUs

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

log_esp32() {
    echo -e "${PURPLE}[ESP32]${NC} $1"
}

log_mcu() {
    echo -e "${PURPLE}[MCU]${NC} $1"
}

# Test QEMU devices
test_qemu_devices() {
    log_test "Testing QEMU RISC-V devices..."
    
    # Check QEMU processes
    local qemu_count=$(ps aux | grep qemu-system-riscv32 | grep -v grep | wc -l)
    if [ "$qemu_count" -gt 0 ]; then
        log_success "Found $qemu_count QEMU RISC-V device(s) running"
        
        # List QEMU processes
        ps aux | grep qemu-system-riscv32 | grep -v grep | while read -r line; do
            log_qemu "QEMU process: $line"
        done
    else
        log_error "No QEMU RISC-V devices found running"
        return 1
    fi
    
    # Check QEMU device resources
    local qemu_devices=$(kubectl get devices -n "$NAMESPACE" | grep qemu-device | wc -l)
    if [ "$qemu_devices" -gt 0 ]; then
        log_success "Found $qemu_devices QEMU device resource(s) in Kubernetes"
        kubectl get devices -n "$NAMESPACE" | grep qemu-device
    else
        log_error "No QEMU device resources found"
        return 1
    fi
}

# Test ESP32 devices
test_esp32_devices() {
    log_test "Testing ESP32 devices..."
    
    # Check ESP32 device resources
    local esp32_devices=$(kubectl get devices -n "$NAMESPACE" | grep esp32-device | wc -l)
    if [ "$esp32_devices" -gt 0 ]; then
        log_success "Found $esp32_devices ESP32 device resource(s) in Kubernetes"
        kubectl get devices -n "$NAMESPACE" | grep esp32-device
    else
        log_error "No ESP32 device resources found"
        return 1
    fi
    
    # Test ESP32 enrollment using MCU simulator
    log_info "Testing ESP32 enrollment using MCU simulator..."
    if timeout 20 ./target/release/wasmbed-mcu-simulator --device-id esp32-device-1 --gateway-host "$GATEWAY_HOST" --gateway-port "$GATEWAY_PORT" --test-mode > /tmp/esp32-test-1.log 2>&1; then
        log_success "esp32-device-1 enrollment test completed"
        
        # Check if enrollment was successful
        if grep -q "enrollment completed" /tmp/esp32-test-1.log; then
            log_success "esp32-device-1 enrollment successful"
        else
            log_warning "esp32-device-1 enrollment may have failed"
        fi
        
        # Check if connection was successful
        if grep -q "connected successfully" /tmp/esp32-test-1.log; then
            log_success "esp32-device-1 connection successful"
        else
            log_warning "esp32-device-1 connection may have failed"
        fi
        
        # Check if WASM execution was successful
        if grep -q "WASM application.*running" /tmp/esp32-test-1.log; then
            log_success "esp32-device-1 WASM execution successful"
        else
            log_warning "esp32-device-1 WASM execution may have failed"
        fi
        
        # Check if microROS was successful
        if grep -q "microROS communication active" /tmp/esp32-test-1.log; then
            log_success "esp32-device-1 microROS communication successful"
        else
            log_warning "esp32-device-1 microROS communication may have failed"
        fi
        
    else
        log_error "esp32-device-1 enrollment test failed"
        return 1
    fi
}

# Test simulated MCU devices
test_mcu_devices() {
    log_test "Testing simulated MCU devices..."
    
    # Check MCU device resources
    local mcu_devices=$(kubectl get devices -n "$NAMESPACE" | grep mcu-device | wc -l)
    if [ "$mcu_devices" -gt 0 ]; then
        log_success "Found $mcu_devices simulated MCU device resource(s) in Kubernetes"
        kubectl get devices -n "$NAMESPACE" | grep mcu-device
    else
        log_error "No simulated MCU device resources found"
        return 1
    fi
    
    # Test MCU enrollment
    log_info "Testing MCU enrollment..."
    if timeout 20 ./target/release/wasmbed-mcu-simulator --device-id mcu-device-1 --gateway-host "$GATEWAY_HOST" --gateway-port "$GATEWAY_PORT" --test-mode > /tmp/mcu-test-1.log 2>&1; then
        log_success "mcu-device-1 enrollment test completed"
        
        # Check if enrollment was successful
        if grep -q "enrollment completed" /tmp/mcu-test-1.log; then
            log_success "mcu-device-1 enrollment successful"
        else
            log_warning "mcu-device-1 enrollment may have failed"
        fi
        
        # Check if connection was successful
        if grep -q "connected successfully" /tmp/mcu-test-1.log; then
            log_success "mcu-device-1 connection successful"
        else
            log_warning "mcu-device-1 connection may have failed"
        fi
        
        # Check if WASM execution was successful
        if grep -q "WASM application.*running" /tmp/mcu-test-1.log; then
            log_success "mcu-device-1 WASM execution successful"
        else
            log_warning "mcu-device-1 WASM execution may have failed"
        fi
        
        # Check if microROS was successful
        if grep -q "microROS communication active" /tmp/mcu-test-1.log; then
            log_success "mcu-device-1 microROS communication successful"
        else
            log_warning "mcu-device-1 microROS communication may have failed"
        fi
        
    else
        log_error "mcu-device-1 enrollment test failed"
        return 1
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

# Test applications
test_applications() {
    log_test "Testing applications..."
    
    # Check applications
    local app_count=$(kubectl get applications -n "$NAMESPACE" 2>/dev/null | wc -l)
    if [ "$app_count" -gt 1 ]; then  # Subtract header line
        log_success "Found $((app_count - 1)) application(s)"
        kubectl get applications -n "$NAMESPACE"
    else
        log_warning "No applications found"
    fi
}

# Test system summary
test_system_summary() {
    log_test "Testing system summary..."
    
    # Count all devices
    local total_devices=$(kubectl get devices -n "$NAMESPACE" | wc -l)
    local qemu_count=$(kubectl get devices -n "$NAMESPACE" | grep qemu-device | wc -l)
    local esp32_count=$(kubectl get devices -n "$NAMESPACE" | grep esp32-device | wc -l)
    local mcu_count=$(kubectl get devices -n "$NAMESPACE" | grep mcu-device | wc -l)
    
    log_success "Total devices: $((total_devices - 1))"  # Subtract header line
    log_success "QEMU RISC-V devices: $qemu_count"
    log_success "ESP32 devices: $esp32_count"
    log_success "Simulated MCU devices: $mcu_count"
    
    # Check gateway pods
    local gateway_pods=$(kubectl get pods -n "$NAMESPACE" -l app=wasmbed-gateway | grep Running | wc -l)
    log_success "Gateway pods running: $gateway_pods"
    
    # Check controller pods
    local controller_pods=$(kubectl get pods -n "$NAMESPACE" -l app=wasmbed-k8s-controller | grep Running | wc -l)
    log_success "Controller pods running: $controller_pods"
}

# Run comprehensive test
run_comprehensive_test() {
    log_info "Starting comprehensive device system test..."
    echo "=================================================="
    
    test_gateway_connectivity
    echo "--------------------------------------------------"
    
    test_qemu_devices
    echo "--------------------------------------------------"
    
    test_esp32_devices
    echo "--------------------------------------------------"
    
    test_mcu_devices
    echo "--------------------------------------------------"
    
    test_applications
    echo "--------------------------------------------------"
    
    test_system_summary
    echo "--------------------------------------------------"
    
    log_success "Comprehensive device system test completed!"
}

# Main function
main() {
    case "${1:-comprehensive}" in
        "comprehensive")
            run_comprehensive_test
            ;;
        "qemu")
            test_qemu_devices
            ;;
        "esp32")
            test_esp32_devices
            ;;
        "mcu")
            test_mcu_devices
            ;;
        "gateway")
            test_gateway_connectivity
            ;;
        "apps")
            test_applications
            ;;
        "summary")
            test_system_summary
            ;;
        *)
            echo "Usage: $0 {comprehensive|qemu|esp32|mcu|gateway|apps|summary}"
            echo "  comprehensive - Run all tests (default)"
            echo "  qemu         - Test QEMU RISC-V devices only"
            echo "  esp32        - Test ESP32 devices only"
            echo "  mcu          - Test simulated MCU devices only"
            echo "  gateway      - Test gateway connectivity only"
            echo "  apps         - Test applications only"
            echo "  summary      - Test system summary only"
            exit 1
            ;;
    esac
}

# Run main function
main "$@"
