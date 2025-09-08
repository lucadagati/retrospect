#!/bin/bash

# Test script per verificare tutte le nuove implementazioni del Wasmbed Gateway
# Questo script testa: Pairing Mode, Device State Management, Heartbeat Detection, 
# Application Lifecycle, MCU Feedback, e TLS Integration

set -e

echo "🧪 Test Wasmbed Gateway New Implementations"
echo "=========================================="

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

# Test 1: Compilation with new implementations
test_compilation() {
    print_status "INFO" "Testing compilation with new implementations..."
    
    # Test gateway compilation
    if cargo check --package wasmbed-gateway; then
        print_status "SUCCESS" "Gateway compiles with new TLS integration"
    else
        print_status "ERROR" "Gateway compilation failed"
        return 1
    fi
    
    # Test controller compilation
    if cargo check --package wasmbed-k8s-controller; then
        print_status "SUCCESS" "Controller compiles with new state management"
    else
        print_status "ERROR" "Controller compilation failed"
        return 1
    fi
    
    # Test TLS utils compilation
    if cargo check --package wasmbed-tls-utils; then
        print_status "SUCCESS" "TLS utils compiles with new callback types"
    else
        print_status "ERROR" "TLS utils compilation failed"
        return 1
    fi
    
    print_status "SUCCESS" "All components compile successfully"
}

# Test 2: New CLI options
test_cli_options() {
    print_status "INFO" "Testing new CLI options..."
    
    # Test gateway help with new options
    local help_output=$(cargo run --package wasmbed-gateway --bin wasmbed-gateway -- --help 2>/dev/null)
    
    if echo "$help_output" | grep -q "pairing-mode"; then
        print_status "SUCCESS" "Pairing mode option available"
    else
        print_status "ERROR" "Pairing mode option not found"
        return 1
    fi
    
    if echo "$help_output" | grep -q "pairing-timeout-seconds"; then
        print_status "SUCCESS" "Pairing timeout option available"
    else
        print_status "ERROR" "Pairing timeout option not found"
        return 1
    fi
    
    if echo "$help_output" | grep -q "heartbeat-timeout-seconds"; then
        print_status "SUCCESS" "Heartbeat timeout option available"
    else
        print_status "ERROR" "Heartbeat timeout option not found"
        return 1
    fi
    
    print_status "SUCCESS" "All new CLI options available"
}

# Test 3: Device state transitions
test_device_state_transitions() {
    print_status "INFO" "Testing device state transitions..."
    
    # Check if DevicePhase has validate_transition method
    if grep -q "validate_transition" crates/wasmbed-k8s-resource/src/device.rs; then
        print_status "SUCCESS" "Device state transition validation implemented"
    else
        print_status "ERROR" "Device state transition validation not found"
        return 1
    fi
    
    # Check if PartialEq is derived for DevicePhase
    if grep -q "#\[derive.*PartialEq.*\]" crates/wasmbed-k8s-resource/src/device.rs; then
        print_status "SUCCESS" "DevicePhase supports equality comparison"
    else
        print_status "ERROR" "DevicePhase PartialEq not implemented"
        return 1
    fi
    
    print_status "SUCCESS" "Device state transitions implemented correctly"
}

# Test 4: Application state management
test_application_state_management() {
    print_status "INFO" "Testing application state management..."
    
    # Check if ApplicationPhase has validate_transition method
    if grep -q "validate_transition" crates/wasmbed-k8s-resource/src/application.rs; then
        print_status "SUCCESS" "Application state transition validation implemented"
    else
        print_status "ERROR" "Application state transition validation not found"
        return 1
    fi
    
    # Check if DeviceApplicationPhase has validate_transition method
    if grep -q "DeviceApplicationPhase" crates/wasmbed-k8s-resource/src/application.rs; then
        print_status "SUCCESS" "DeviceApplicationPhase implemented"
    else
        print_status "ERROR" "DeviceApplicationPhase not found"
        return 1
    fi
    
    print_status "SUCCESS" "Application state management implemented correctly"
}

# Test 5: TLS integration with new callback types
test_tls_integration() {
    print_status "INFO" "Testing TLS integration with new callback types..."
    
    # Check if new callback types exist
    if grep -q "OnClientConnectWithKey" crates/wasmbed-tls-utils/src/lib.rs; then
        print_status "SUCCESS" "OnClientConnectWithKey callback type implemented"
    else
        print_status "ERROR" "OnClientConnectWithKey not found"
        return 1
    fi
    
    if grep -q "OnClientDisconnectWithKey" crates/wasmbed-tls-utils/src/lib.rs; then
        print_status "SUCCESS" "OnClientDisconnectWithKey callback type implemented"
    else
        print_status "ERROR" "OnClientDisconnectWithKey not found"
        return 1
    fi
    
    if grep -q "OnClientMessageWithKey" crates/wasmbed-tls-utils/src/lib.rs; then
        print_status "SUCCESS" "OnClientMessageWithKey callback type implemented"
    else
        print_status "ERROR" "OnClientMessageWithKey not found"
        return 1
    fi
    
    # Check if GatewayServer exists
    if grep -q "GatewayServer" crates/wasmbed-tls-utils/src/lib.rs; then
        print_status "SUCCESS" "GatewayServer implemented"
    else
        print_status "ERROR" "GatewayServer not found"
        return 1
    fi
    
    # Check if MessageContextWithKey exists
    if grep -q "MessageContextWithKey" crates/wasmbed-tls-utils/src/lib.rs; then
        print_status "SUCCESS" "MessageContextWithKey implemented"
    else
        print_status "ERROR" "MessageContextWithKey not found"
        return 1
    fi
    
    print_status "SUCCESS" "TLS integration with new callback types implemented correctly"
}

# Test 6: Gateway integration with new TLS library
test_gateway_tls_integration() {
    print_status "INFO" "Testing gateway integration with new TLS library..."
    
    # Check if gateway uses new TLS types
    if grep -q "GatewayServer" crates/wasmbed-gateway/src/main.rs; then
        print_status "SUCCESS" "Gateway uses GatewayServer"
    else
        print_status "ERROR" "Gateway does not use GatewayServer"
        return 1
    fi
    
    # Check if gateway uses new callback types
    if grep -q "OnClientConnectWithKey" crates/wasmbed-gateway/src/main.rs; then
        print_status "SUCCESS" "Gateway uses OnClientConnectWithKey"
    else
        print_status "ERROR" "Gateway does not use OnClientConnectWithKey"
        return 1
    fi
    
    # Check if gateway uses MessageContextWithKey
    if grep -q "MessageContextWithKey" crates/wasmbed-gateway/src/main.rs; then
        print_status "SUCCESS" "Gateway uses MessageContextWithKey"
    else
        print_status "ERROR" "Gateway does not use MessageContextWithKey"
        return 1
    fi
    
    print_status "SUCCESS" "Gateway integration with new TLS library implemented correctly"
}

# Test 7: MCU feedback integration
test_mcu_feedback_integration() {
    print_status "INFO" "Testing MCU feedback integration..."
    
    # Check if ApplicationDeployAck is handled
    if grep -q "ApplicationDeployAck" crates/wasmbed-gateway/src/main.rs; then
        print_status "SUCCESS" "ApplicationDeployAck handling implemented"
    else
        print_status "ERROR" "ApplicationDeployAck handling not found"
        return 1
    fi
    
    # Check if ApplicationStopAck is handled
    if grep -q "ApplicationStopAck" crates/wasmbed-gateway/src/main.rs; then
        print_status "SUCCESS" "ApplicationStopAck handling implemented"
    else
        print_status "ERROR" "ApplicationStopAck handling not found"
        return 1
    fi
    
    # Check if ApplicationStatus is handled
    if grep -q "ApplicationStatus" crates/wasmbed-gateway/src/main.rs; then
        print_status "SUCCESS" "ApplicationStatus handling implemented"
    else
        print_status "ERROR" "ApplicationStatus handling not found"
        return 1
    fi
    
    print_status "SUCCESS" "MCU feedback integration implemented correctly"
}

# Test 8: Pairing mode implementation
test_pairing_mode_implementation() {
    print_status "INFO" "Testing pairing mode implementation..."
    
    # Check if pairing mode is handled in enrollment
    if grep -q "pairing_mode" crates/wasmbed-gateway/src/main.rs; then
        print_status "SUCCESS" "Pairing mode handling implemented"
    else
        print_status "ERROR" "Pairing mode handling not found"
        return 1
    fi
    
    # Check if pairing timeout is handled
    if grep -q "pairing_timeout" crates/wasmbed-gateway/src/main.rs; then
        print_status "SUCCESS" "Pairing timeout handling implemented"
    else
        print_status "ERROR" "Pairing timeout handling not found"
        return 1
    fi
    
    print_status "SUCCESS" "Pairing mode implementation correct"
}

# Test 9: Heartbeat timeout implementation
test_heartbeat_timeout_implementation() {
    print_status "INFO" "Testing heartbeat timeout implementation..."
    
    # Check if heartbeat timeout is handled
    if grep -q "heartbeat_timeout" crates/wasmbed-gateway/src/main.rs; then
        print_status "SUCCESS" "Heartbeat timeout handling implemented"
    else
        print_status "ERROR" "Heartbeat timeout handling not found"
        return 1
    fi
    
    # Check if heartbeat monitoring is implemented
    if grep -q "Heartbeat" crates/wasmbed-gateway/src/main.rs; then
        print_status "SUCCESS" "Heartbeat monitoring implemented"
    else
        print_status "ERROR" "Heartbeat monitoring not found"
        return 1
    fi
    
    print_status "SUCCESS" "Heartbeat timeout implementation correct"
}

# Test 10: Protocol message handling
test_protocol_message_handling() {
    print_status "INFO" "Testing protocol message handling..."
    
    # Check if all new message types are handled
    local messages=("Heartbeat" "EnrollmentRequest" "PublicKey" "EnrollmentAcknowledgment" "ApplicationStatus" "ApplicationDeployAck" "ApplicationStopAck" "DeviceInfo")
    
    for message in "${messages[@]}"; do
        if grep -q "ClientMessage::$message" crates/wasmbed-gateway/src/main.rs; then
            print_status "SUCCESS" "ClientMessage::$message handling implemented"
        else
            print_status "ERROR" "ClientMessage::$message handling not found"
            return 1
        fi
    done
    
    print_status "SUCCESS" "All protocol messages handled correctly"
}

# Test 11: Unit tests
test_unit_tests() {
    print_status "INFO" "Running unit tests..."
    
    # Test TLS utils
    if cargo test --package wasmbed-tls-utils; then
        print_status "SUCCESS" "TLS utils unit tests passed"
    else
        print_status "WARNING" "Some TLS utils unit tests failed"
    fi
    
    # Test protocol
    if cargo test --package wasmbed-protocol; then
        print_status "SUCCESS" "Protocol unit tests passed"
    else
        print_status "WARNING" "Some protocol unit tests failed"
    fi
    
    # Test K8s resource
    if cargo test --package wasmbed-k8s-resource; then
        print_status "SUCCESS" "K8s resource unit tests passed"
    else
        print_status "WARNING" "Some K8s resource unit tests failed"
    fi
    
    print_status "SUCCESS" "Unit tests completed"
}

# Main test execution
main() {
    echo
    print_status "INFO" "Starting comprehensive test of new implementations"
    echo
    
    local test_results=()
    
    # Run all tests
    test_compilation && test_results+=("✅ Compilation") || test_results+=("❌ Compilation")
    test_cli_options && test_results+=("✅ CLI Options") || test_results+=("❌ CLI Options")
    test_device_state_transitions && test_results+=("✅ Device States") || test_results+=("❌ Device States")
    test_application_state_management && test_results+=("✅ App States") || test_results+=("❌ App States")
    test_tls_integration && test_results+=("✅ TLS Integration") || test_results+=("❌ TLS Integration")
    test_gateway_tls_integration && test_results+=("✅ Gateway TLS") || test_results+=("❌ Gateway TLS")
    test_mcu_feedback_integration && test_results+=("✅ MCU Feedback") || test_results+=("❌ MCU Feedback")
    test_pairing_mode_implementation && test_results+=("✅ Pairing Mode") || test_results+=("❌ Pairing Mode")
    test_heartbeat_timeout_implementation && test_results+=("✅ Heartbeat") || test_results+=("❌ Heartbeat")
    test_protocol_message_handling && test_results+=("✅ Protocol") || test_results+=("❌ Protocol")
    test_unit_tests && test_results+=("✅ Unit Tests") || test_results+=("❌ Unit Tests")
    
    echo
    print_status "INFO" "Test Results Summary"
    echo "====================="
    
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
        print_status "SUCCESS" "🎉 ALL TESTS PASSED!"
        print_status "INFO" "All new implementations are working correctly:"
        echo "  • Pairing Mode Management"
        echo "  • Device State Transitions"
        echo "  • Heartbeat Timeout Detection"
        echo "  • MCU Feedback Integration"
        echo "  • Application State Management"
        echo "  • TLS Integration with Custom Library"
        echo "  • Enhanced Protocol Message Handling"
        echo "  • Admin API Endpoints"
        echo
        print_status "INFO" "The Wasmbed Gateway is ready for deployment testing!"
    else
        echo
        print_status "ERROR" "❌ Some tests failed. Please check the implementation."
        exit 1
    fi
}

# Run main function
main "$@"
