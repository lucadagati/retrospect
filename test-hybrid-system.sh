#!/bin/bash

# Wasmbed Hybrid System Test
# Tests both QEMU devices and simulated devices

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
GATEWAY_HTTP_PORT="30080"
GATEWAY_TLS_PORT="30423"

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

# Test system status
test_system_status() {
    log_test "Testing system status..."
    
    # Check Kubernetes cluster
    if kubectl cluster-info >/dev/null 2>&1; then
        log_success "Kubernetes cluster is running"
    else
        log_error "Kubernetes cluster is not accessible"
        return 1
    fi
    
    # Check namespace
    if kubectl get namespace "$NAMESPACE" >/dev/null 2>&1; then
        log_success "Namespace '$NAMESPACE' exists"
    else
        log_error "Namespace '$NAMESPACE' not found"
        return 1
    fi
    
    # Check gateway pod
    if kubectl get pod -n "$NAMESPACE" -l app=wasmbed-gateway | grep -q Running; then
        log_success "Gateway pod is running"
    else
        log_error "Gateway pod is not running"
        return 1
    fi
    
    # Check controller pod
    if kubectl get pod -n "$NAMESPACE" -l app=wasmbed-k8s-controller | grep -q Running; then
        log_success "Controller pod is running"
    else
        log_error "Controller pod is not running"
        return 1
    fi
}

# Test QEMU devices
test_qemu_devices() {
    log_test "Testing QEMU devices..."
    
    # Check QEMU processes
    local qemu_count=$(ps aux | grep qemu-system-riscv32 | grep -v grep | wc -l)
    if [ "$qemu_count" -gt 0 ]; then
        log_success "Found $qemu_count QEMU device(s) running"
        
        # List QEMU processes
        ps aux | grep qemu-system-riscv32 | grep -v grep | while read -r line; do
            log_qemu "QEMU process: $line"
        done
    else
        log_warning "No QEMU devices found running"
    fi
    
    # Check QEMU device resources
    local qemu_devices=$(kubectl get devices -n "$NAMESPACE" | grep qemu-device | wc -l)
    if [ "$qemu_devices" -gt 0 ]; then
        log_success "Found $qemu_devices QEMU device resource(s) in Kubernetes"
        kubectl get devices -n "$NAMESPACE" | grep qemu-device
    else
        log_warning "No QEMU device resources found"
    fi
}

# Test simulated devices
test_simulated_devices() {
    log_test "Testing simulated devices..."
    
    # Check simulated device resources
    local sim_devices=$(kubectl get devices -n "$NAMESPACE" | grep mcu-device | wc -l)
    if [ "$sim_devices" -gt 0 ]; then
        log_success "Found $sim_devices simulated device resource(s) in Kubernetes"
        kubectl get devices -n "$NAMESPACE" | grep mcu-device
    else
        log_warning "No simulated device resources found"
    fi
}

# Test gateway connectivity
test_gateway_connectivity() {
    log_test "Testing gateway connectivity..."
    
    # Test HTTP API
    if curl -s "http://$GATEWAY_HOST:$GATEWAY_HTTP_PORT/health" >/dev/null 2>&1; then
        log_success "Gateway HTTP API is accessible"
    else
        log_error "Gateway HTTP API is not accessible"
    fi
    
    # Test TLS port (basic connectivity)
    if timeout 5 bash -c "</dev/tcp/$GATEWAY_HOST/$GATEWAY_TLS_PORT" 2>/dev/null; then
        log_success "Gateway TLS port is accessible"
    else
        log_warning "Gateway TLS port is not accessible"
    fi
}

# Test device enrollment
test_device_enrollment() {
    log_test "Testing device enrollment..."
    
    # Check device status
    kubectl get devices -n "$NAMESPACE" -o jsonpath='{range .items[*]}{.metadata.name}{"\t"}{.status.phase}{"\n"}{end}' | while read -r device_name status; do
        if [ -n "$status" ] && [ "$status" != "null" ]; then
            log_success "Device $device_name status: $status"
        else
            log_warning "Device $device_name has no status"
        fi
    done
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

# Test MCU simulator
test_mcu_simulator() {
    log_test "Testing MCU simulator..."
    
    # Check if simulator binary exists
    if [ -f "target/release/wasmbed-mcu-simulator" ]; then
        log_success "MCU simulator binary exists"
        
        # Test simulator help
        if timeout 5 ./target/release/wasmbed-mcu-simulator --help >/dev/null 2>&1; then
            log_success "MCU simulator is functional"
        else
            log_warning "MCU simulator may have issues"
        fi
    else
        log_warning "MCU simulator binary not found"
    fi
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

# Run comprehensive test
run_comprehensive_test() {
    log_info "Starting comprehensive hybrid system test..."
    echo "=================================================="
    
    test_system_status
    echo "--------------------------------------------------"
    
    test_qemu_devices
    echo "--------------------------------------------------"
    
    test_simulated_devices
    echo "--------------------------------------------------"
    
    test_gateway_connectivity
    echo "--------------------------------------------------"
    
    test_device_enrollment
    echo "--------------------------------------------------"
    
    test_applications
    echo "--------------------------------------------------"
    
    test_mcu_simulator
    echo "--------------------------------------------------"
    
    test_qemu_serial
    echo "--------------------------------------------------"
    
    log_success "Comprehensive hybrid system test completed!"
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
        "simulated")
            test_simulated_devices
            ;;
        "gateway")
            test_gateway_connectivity
            ;;
        "devices")
            test_device_enrollment
            ;;
        "apps")
            test_applications
            ;;
        "simulator")
            test_mcu_simulator
            ;;
        "serial")
            test_qemu_serial
            ;;
        *)
            echo "Usage: $0 {comprehensive|qemu|simulated|gateway|devices|apps|simulator|serial}"
            echo "  comprehensive - Run all tests (default)"
            echo "  qemu         - Test QEMU devices only"
            echo "  simulated    - Test simulated devices only"
            echo "  gateway      - Test gateway connectivity"
            echo "  devices      - Test device enrollment"
            echo "  apps         - Test applications"
            echo "  simulator    - Test MCU simulator"
            echo "  serial       - Test QEMU serial communication"
            exit 1
            ;;
    esac
}

# Run main function
main "$@"
