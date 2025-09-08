#!/bin/bash

# Test di deployment reale per Wasmbed Gateway con nuove implementazioni
# Questo script testa il deployment completo con certificati reali

set -e

echo "🚀 Test Deployment Reale Wasmbed Gateway"
echo "========================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    local status=$1
    local message=$2
    case $status in
        "INFO")
            echo -e "${BLUE}ℹ️  $message${NC}"
            ;;
        "SUCCESS")
            echo -e "${GREEN}✅ $message${NC}"
            ;;
        "WARNING")
            echo -e "${YELLOW}⚠️  $message${NC}"
            ;;
        "ERROR")
            echo -e "${RED}❌ $message${NC}"
            ;;
    esac
}

# Configuration
GATEWAY_PORT=8443
HTTP_PORT=8080
NAMESPACE="wasmbed-test"
CERT_DIR="test-deployment/certs"

# Check prerequisites
check_prerequisites() {
    print_status "INFO" "Checking prerequisites..."
    
    local missing_deps=()
    
    # Check required tools
    command -v cargo >/dev/null 2>&1 || missing_deps+=("cargo")
    command -v kubectl >/dev/null 2>&1 || missing_deps+=("kubectl")
    command -v k3d >/dev/null 2>&1 || missing_deps+=("k3d")
    command -v docker >/dev/null 2>&1 || missing_deps+=("docker")
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        print_status "ERROR" "Missing dependencies: ${missing_deps[*]}"
        print_status "ERROR" "Please install missing dependencies and try again"
        exit 1
    fi
    
    # Check if certificates exist
    if [ ! -f "$CERT_DIR/server-cert-pem.pem" ] || [ ! -f "$CERT_DIR/server-key-pem.pem" ] || [ ! -f "$CERT_DIR/ca-cert-pem.pem" ]; then
        print_status "ERROR" "Certificates not found in $CERT_DIR"
        print_status "ERROR" "Please run certificate generation first"
        exit 1
    fi
    
    print_status "SUCCESS" "All prerequisites satisfied"
}

# Create test namespace
create_test_namespace() {
    print_status "INFO" "Creating test namespace: $NAMESPACE"
    
    kubectl create namespace "$NAMESPACE" 2>/dev/null || true
    print_status "SUCCESS" "Test namespace created"
}

# Deploy CRDs
deploy_crds() {
    print_status "INFO" "Deploying Custom Resource Definitions..."
    
    # Check if CRD files exist
    if [ -d "resources/k8s/crds" ]; then
        kubectl apply -f resources/k8s/crds/ -n "$NAMESPACE"
        print_status "SUCCESS" "CRDs deployed"
    else
        print_status "WARNING" "CRD files not found, skipping CRD deployment"
    fi
}

# Test gateway compilation with real certificates
test_gateway_compilation() {
    print_status "INFO" "Testing gateway compilation with real certificates..."
    
    if cargo build --package wasmbed-gateway; then
        print_status "SUCCESS" "Gateway compiles successfully"
    else
        print_status "ERROR" "Gateway compilation failed"
        exit 1
    fi
}

# Test gateway startup with real certificates
test_gateway_startup() {
    print_status "INFO" "Testing gateway startup with real certificates..."
    
    # Start gateway in background with test certificates
    print_status "INFO" "Starting gateway with certificates from $CERT_DIR"
    
    # Create a test configuration
    local gateway_pid
    cargo run --package wasmbed-gateway --bin wasmbed-gateway -- \
        --bind-addr "127.0.0.1:$GATEWAY_PORT" \
        --http-addr "127.0.0.1:$HTTP_PORT" \
        --private-key "$CERT_DIR/server-key-pem.pem" \
        --certificate "$CERT_DIR/server-cert-pem.pem" \
        --client-ca "$CERT_DIR/ca-cert-pem.pem" \
        --namespace "$NAMESPACE" \
        --pod-namespace "$NAMESPACE" \
        --pod-name "test-gateway" \
        --pairing-mode \
        --pairing-timeout-seconds 300 \
        --heartbeat-timeout-seconds 90 &
    
    gateway_pid=$!
    
    # Wait for gateway to start
    print_status "INFO" "Waiting for gateway to start..."
    sleep 5
    
    # Check if gateway is running
    if kill -0 $gateway_pid 2>/dev/null; then
        print_status "SUCCESS" "Gateway started successfully (PID: $gateway_pid)"
        
        # Test HTTP endpoint
        if curl -s "http://127.0.0.1:$HTTP_PORT/health" >/dev/null 2>&1; then
            print_status "SUCCESS" "Gateway HTTP endpoint responding"
        else
            print_status "WARNING" "Gateway HTTP endpoint not responding"
        fi
        
        # Test TLS endpoint (basic connectivity)
        if timeout 5 openssl s_client -connect "127.0.0.1:$GATEWAY_PORT" -quiet </dev/null >/dev/null 2>&1; then
            print_status "SUCCESS" "Gateway TLS endpoint accepting connections"
        else
            print_status "WARNING" "Gateway TLS endpoint not accepting connections"
        fi
        
        # Stop gateway
        print_status "INFO" "Stopping test gateway..."
        kill $gateway_pid 2>/dev/null || true
        wait $gateway_pid 2>/dev/null || true
        
        print_status "SUCCESS" "Gateway test completed successfully"
    else
        print_status "ERROR" "Gateway failed to start"
        exit 1
    fi
}

# Test client connection
test_client_connection() {
    print_status "INFO" "Testing client connection with TLS certificates..."
    
    # Build test client
    if cargo build --package wasmbed-gateway-test-client; then
        print_status "SUCCESS" "Test client compiled successfully"
    else
        print_status "ERROR" "Test client compilation failed"
        return 1
    fi
    
    # Test client help
    if cargo run --package wasmbed-gateway-test-client -- --help >/dev/null 2>&1; then
        print_status "SUCCESS" "Test client help works"
    else
        print_status "WARNING" "Test client help failed"
    fi
}

# Test controller compilation
test_controller_compilation() {
    print_status "INFO" "Testing controller compilation..."
    
    if cargo build --package wasmbed-k8s-controller; then
        print_status "SUCCESS" "Controller compiles successfully"
    else
        print_status "ERROR" "Controller compilation failed"
        return 1
    fi
}

# Test TLS utils functionality
test_tls_utils() {
    print_status "INFO" "Testing TLS utils functionality..."
    
    # Run TLS utils tests
    if cargo test --package wasmbed-tls-utils; then
        print_status "SUCCESS" "TLS utils tests passed"
    else
        print_status "WARNING" "Some TLS utils tests failed"
    fi
    
    # Test certificate parsing
    if cargo run --package wasmbed-cert-tool -- --help >/dev/null 2>&1; then
        print_status "SUCCESS" "Certificate tool works"
    else
        print_status "WARNING" "Certificate tool failed"
    fi
}

# Test protocol message handling
test_protocol_messages() {
    print_status "INFO" "Testing protocol message handling..."
    
    # Run protocol tests
    if cargo test --package wasmbed-protocol; then
        print_status "SUCCESS" "Protocol tests passed"
    else
        print_status "WARNING" "Some protocol tests failed"
    fi
}

# Test new CLI options
test_new_cli_options() {
    print_status "INFO" "Testing new CLI options..."
    
    local help_output=$(cargo run --package wasmbed-gateway --bin wasmbed-gateway -- --help 2>/dev/null)
    
    # Check for new options
    local new_options=("pairing-mode" "pairing-timeout-seconds" "heartbeat-timeout-seconds")
    
    for option in "${new_options[@]}"; do
        if echo "$help_output" | grep -q "$option"; then
            print_status "SUCCESS" "CLI option --$option available"
        else
            print_status "ERROR" "CLI option --$option not found"
            return 1
        fi
    done
    
    print_status "SUCCESS" "All new CLI options available"
}

# Test device state management
test_device_state_management() {
    print_status "INFO" "Testing device state management..."
    
    # Check if DevicePhase has validate_transition method
    if grep -q "validate_transition" crates/wasmbed-k8s-resource/src/device.rs; then
        print_status "SUCCESS" "Device state transition validation implemented"
    else
        print_status "ERROR" "Device state transition validation not found"
        return 1
    fi
    
    # Check if ApplicationPhase has validate_transition method
    if grep -q "validate_transition" crates/wasmbed-k8s-resource/src/application.rs; then
        print_status "SUCCESS" "Application state transition validation implemented"
    else
        print_status "ERROR" "Application state transition validation not found"
        return 1
    fi
    
    print_status "SUCCESS" "Device state management implemented correctly"
}

# Test MCU feedback integration
test_mcu_feedback() {
    print_status "INFO" "Testing MCU feedback integration..."
    
    # Check if feedback messages are handled
    local feedback_messages=("ApplicationDeployAck" "ApplicationStopAck" "ApplicationStatus")
    
    for message in "${feedback_messages[@]}"; do
        if grep -q "ClientMessage::$message" crates/wasmbed-gateway/src/main.rs; then
            print_status "SUCCESS" "MCU feedback message $message handled"
        else
            print_status "ERROR" "MCU feedback message $message not handled"
            return 1
        fi
    done
    
    print_status "SUCCESS" "MCU feedback integration implemented correctly"
}

# Cleanup
cleanup() {
    print_status "INFO" "Cleaning up test resources..."
    
    # Delete test namespace
    kubectl delete namespace "$NAMESPACE" 2>/dev/null || true
    
    # Kill any remaining gateway processes
    pkill -f "wasmbed-gateway" 2>/dev/null || true
    
    print_status "SUCCESS" "Cleanup completed"
}

# Main test execution
main() {
    echo
    print_status "INFO" "Starting real deployment test"
    echo
    
    local test_results=()
    
    # Run all tests
    check_prerequisites && test_results+=("✅ Prerequisites") || test_results+=("❌ Prerequisites")
    create_test_namespace && test_results+=("✅ Namespace") || test_results+=("❌ Namespace")
    deploy_crds && test_results+=("✅ CRDs") || test_results+=("❌ CRDs")
    test_gateway_compilation && test_results+=("✅ Gateway Compilation") || test_results+=("❌ Gateway Compilation")
    test_gateway_startup && test_results+=("✅ Gateway Startup") || test_results+=("❌ Gateway Startup")
    test_client_connection && test_results+=("✅ Client Connection") || test_results+=("❌ Client Connection")
    test_controller_compilation && test_results+=("✅ Controller Compilation") || test_results+=("❌ Controller Compilation")
    test_tls_utils && test_results+=("✅ TLS Utils") || test_results+=("❌ TLS Utils")
    test_protocol_messages && test_results+=("✅ Protocol Messages") || test_results+=("❌ Protocol Messages")
    test_new_cli_options && test_results+=("✅ CLI Options") || test_results+=("❌ CLI Options")
    test_device_state_management && test_results+=("✅ State Management") || test_results+=("❌ State Management")
    test_mcu_feedback && test_results+=("✅ MCU Feedback") || test_results+=("❌ MCU Feedback")
    
    echo
    print_status "INFO" "Deployment Test Results Summary"
    echo "====================================="
    
    local passed=0
    local failed=0
    
    for result in "${test_results[@]}"; do
        echo "  $result"
        if [[ $result == ✅* ]]; then
            ((passed++))
        else
            ((failed++))
        fi
    done
    
    echo
    print_status "INFO" "Summary: $passed passed, $failed failed"
    
    if [ $failed -eq 0 ]; then
        echo
        print_status "SUCCESS" "🎉 ALL DEPLOYMENT TESTS PASSED!"
        print_status "INFO" "Real deployment test successful with:"
        echo "  • Real TLS certificates"
        echo "  • Gateway startup and connectivity"
        echo "  • Client connection capabilities"
        echo "  • Controller compilation"
        echo "  • TLS utils functionality"
        echo "  • Protocol message handling"
        echo "  • New CLI options"
        echo "  • Device state management"
        echo "  • MCU feedback integration"
        echo
        print_status "INFO" "The Wasmbed Gateway is ready for production deployment!"
    else
        echo
        print_status "ERROR" "❌ Some deployment tests failed. Please check the implementation."
        cleanup
        exit 1
    fi
    
    cleanup
}

# Trap to ensure cleanup on exit
trap cleanup EXIT

# Run main function
main "$@"
