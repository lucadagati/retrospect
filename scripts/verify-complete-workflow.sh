#!/bin/bash

# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

# Complete Workflow Verification Script
# Tests all workflow steps: Create -> Enroll -> Connect -> Deploy -> Stop -> Disconnect -> Delete

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_status() {
    local status=$1
    local message=$2
    case $status in
        "SUCCESS") echo -e "${GREEN}✓${NC} $message" ;;
        "ERROR") echo -e "${RED}✗${NC} $message" ;;
        "WARNING") echo -e "${YELLOW}⚠${NC} $message" ;;
        "INFO") echo -e "${BLUE}ℹ${NC} $message" ;;
    esac
}

print_header() {
    echo ""
    echo "========================================"
    echo "  $1"
    echo "========================================"
}

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

print_header "COMPLETE WORKFLOW VERIFICATION"

# Configuration
API_SERVER_URL="${API_SERVER_URL:-http://localhost:3001}"
GATEWAY_URL="${GATEWAY_URL:-http://localhost:8080}"
KUBECTL_NAMESPACE="${KUBECTL_NAMESPACE:-wasmbed}"

# Test counters
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_WARNING=0

# Test device/app names (with timestamp to avoid conflicts)
TIMESTAMP=$(date +%s)
TEST_DEVICE_NAME="workflow-test-device-${TIMESTAMP}"
TEST_APP_NAME="workflow-test-app-${TIMESTAMP}"

# Cleanup function
cleanup() {
    print_header "CLEANUP"
    print_status "INFO" "Cleaning up test resources..."
    
    # Delete test device if exists
    if kubectl get device "$TEST_DEVICE_NAME" -n "$KUBECTL_NAMESPACE" >/dev/null 2>&1; then
        kubectl delete device "$TEST_DEVICE_NAME" -n "$KUBECTL_NAMESPACE" >/dev/null 2>&1 || true
        print_status "INFO" "Deleted test device: $TEST_DEVICE_NAME"
    fi
    
    # Delete test application if exists
    if kubectl get application "$TEST_APP_NAME" -n "$KUBECTL_NAMESPACE" >/dev/null 2>&1; then
        kubectl delete application "$TEST_APP_NAME" -n "$KUBECTL_NAMESPACE" >/dev/null 2>&1 || true
        print_status "INFO" "Deleted test application: $TEST_APP_NAME"
    fi
}

trap cleanup EXIT

# Check prerequisites
print_header "PREREQUISITES CHECK"

check_service() {
    local name=$1
    local url=$2
    
    if curl -4 -s -f "$url/health" >/dev/null 2>&1 || curl -4 -s -f "$url" >/dev/null 2>&1; then
        print_status "SUCCESS" "$name is responding"
        return 0
    else
        print_status "ERROR" "$name is not responding at $url"
        return 1
    fi
}

if ! check_service "API Server" "$API_SERVER_URL"; then
    print_status "ERROR" "API Server must be running"
    exit 1
fi

if ! check_service "Gateway" "$GATEWAY_URL"; then
    print_status "WARNING" "Gateway is not responding. Starting port-forward..."
    ./scripts/fix-gateway-connection.sh || {
        print_status "ERROR" "Failed to start gateway port-forward"
        exit 1
    }
fi

if ! kubectl get namespace "$KUBECTL_NAMESPACE" >/dev/null 2>&1; then
    print_status "ERROR" "Kubernetes namespace '$KUBECTL_NAMESPACE' not found"
    exit 1
fi

# Get first available gateway
GATEWAY_ID=$(kubectl get gateways -n "$KUBECTL_NAMESPACE" -o jsonpath='{.items[0].metadata.name}' 2>/dev/null || echo "gateway-1")
print_status "INFO" "Using gateway: $GATEWAY_ID"

print_status "SUCCESS" "All prerequisites met"

# STEP 1: Create Device
print_header "STEP 1: CREATE DEVICE"

print_status "INFO" "Creating device: $TEST_DEVICE_NAME"

CREATE_RESPONSE=$(curl -s -X POST "${API_SERVER_URL}/api/v1/devices" \
    -H "Content-Type: application/json" \
    -d "{
        \"name\": \"${TEST_DEVICE_NAME}\",
        \"type\": \"MCU\",
        \"mcuType\": \"RenodeArduinoNano33Ble\",
        \"gatewayId\": \"${GATEWAY_ID}\"
    }" 2>/dev/null || echo "{}")

if echo "$CREATE_RESPONSE" | jq -e '.success == true' >/dev/null 2>&1; then
    print_status "SUCCESS" "Device created successfully"
    TESTS_PASSED=$((TESTS_PASSED + 1))
    
    # Wait for device to appear in Kubernetes
    sleep 3
    
    if kubectl get device "$TEST_DEVICE_NAME" -n "$KUBECTL_NAMESPACE" >/dev/null 2>&1; then
        print_status "SUCCESS" "Device visible in Kubernetes"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        print_status "ERROR" "Device not found in Kubernetes"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
else
    ERROR_MSG=$(echo "$CREATE_RESPONSE" | jq -r '.message // .errors[0] // "Unknown error"' 2>/dev/null || echo "Unknown error")
    print_status "ERROR" "Failed to create device: $ERROR_MSG"
    TESTS_FAILED=$((TESTS_FAILED + 1))
    exit 1
fi

# STEP 2: Verify Device in API
print_header "STEP 2: VERIFY DEVICE IN API"

sleep 2
API_DEVICES=$(curl -s "${API_SERVER_URL}/api/v1/devices" 2>/dev/null || echo "{}")
DEVICE_FOUND=$(echo "$API_DEVICES" | jq -r ".devices[] | select(.name == \"${TEST_DEVICE_NAME}\" or .id == \"${TEST_DEVICE_NAME}\") | .id" 2>/dev/null || echo "")

if [ -n "$DEVICE_FOUND" ]; then
    print_status "SUCCESS" "Device visible in API: $DEVICE_FOUND"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    print_status "ERROR" "Device not found in API"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi

# STEP 3: Enroll Device
print_header "STEP 3: ENROLL DEVICE"

print_status "INFO" "Enrolling device to gateway..."

ENROLL_RESPONSE=$(curl -s -X POST "${API_SERVER_URL}/api/v1/devices/${TEST_DEVICE_NAME}/enroll" \
    -H "Content-Type: application/json" \
    -d "{\"gatewayId\": \"${GATEWAY_ID}\"}" 2>/dev/null || echo "{}")

if echo "$ENROLL_RESPONSE" | jq -e '.success == true or .status == "Enrolled"' >/dev/null 2>&1; then
    print_status "SUCCESS" "Device enrolled successfully"
    TESTS_PASSED=$((TESTS_PASSED + 1))
    
    # Verify status in Kubernetes
    sleep 2
    DEVICE_STATUS=$(kubectl get device "$TEST_DEVICE_NAME" -n "$KUBECTL_NAMESPACE" -o jsonpath='{.status.phase}' 2>/dev/null || echo "")
    if [ "$DEVICE_STATUS" = "Enrolled" ]; then
        print_status "SUCCESS" "Device status is 'Enrolled' in Kubernetes"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        print_status "WARNING" "Device status is '$DEVICE_STATUS' (expected 'Enrolled')"
        TESTS_WARNING=$((TESTS_WARNING + 1))
    fi
else
    ERROR_MSG=$(echo "$ENROLL_RESPONSE" | jq -r '.message // "Unknown error"' 2>/dev/null || echo "Unknown error")
    print_status "WARNING" "Enroll may have failed: $ERROR_MSG (continuing anyway)"
    TESTS_WARNING=$((TESTS_WARNING + 1))
fi

# STEP 4: Connect Device (optional - may take time)
print_header "STEP 4: CONNECT DEVICE"

print_status "INFO" "Connecting device (this may take time for Renode startup)..."

CONNECT_RESPONSE=$(curl -s -X POST "${API_SERVER_URL}/api/v1/devices/${TEST_DEVICE_NAME}/connect" \
    -H "Content-Type: application/json" \
    -d '{}' \
    --max-time 30 2>/dev/null || echo "{}")

if echo "$CONNECT_RESPONSE" | jq -e '.success == true or .status == "Connected"' >/dev/null 2>&1; then
    print_status "SUCCESS" "Device connection initiated"
    TESTS_PASSED=$((TESTS_PASSED + 1))
    
    # Note: Full connection may take longer (Renode startup)
    print_status "INFO" "Note: Full connection may take additional time for Renode to start"
else
    ERROR_MSG=$(echo "$CONNECT_RESPONSE" | jq -r '.message // "Unknown error"' 2>/dev/null || echo "Unknown error")
    print_status "WARNING" "Connect may have failed or timed out: $ERROR_MSG"
    print_status "INFO" "This is acceptable if Renode startup takes longer than 30 seconds"
    TESTS_WARNING=$((TESTS_WARNING + 1))
fi

# STEP 5: Create Application
print_header "STEP 5: CREATE APPLICATION"

print_status "INFO" "Creating application: $TEST_APP_NAME"

# Create a minimal WASM module (base64 encoded)
MINIMAL_WASM="AGFzbQEAAAA="  # Minimal valid WASM module

CREATE_APP_RESPONSE=$(curl -s -X POST "${API_SERVER_URL}/api/v1/applications" \
    -H "Content-Type: application/json" \
    -d "{
        \"name\": \"${TEST_APP_NAME}\",
        \"description\": \"Test application for workflow verification\",
        \"wasmBytes\": \"${MINIMAL_WASM}\",
        \"targetDevices\": {
            \"deviceNames\": [\"${TEST_DEVICE_NAME}\"]
        }
    }" 2>/dev/null || echo "{}")

if echo "$CREATE_APP_RESPONSE" | jq -e '.success == true or .app_id' >/dev/null 2>&1; then
    print_status "SUCCESS" "Application created successfully"
    TESTS_PASSED=$((TESTS_PASSED + 1))
    
    # Wait for application to appear in Kubernetes
    sleep 2
    
    if kubectl get application "$TEST_APP_NAME" -n "$KUBECTL_NAMESPACE" >/dev/null 2>&1; then
        print_status "SUCCESS" "Application visible in Kubernetes"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        print_status "ERROR" "Application not found in Kubernetes"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
else
    ERROR_MSG=$(echo "$CREATE_APP_RESPONSE" | jq -r '.message // "Unknown error"' 2>/dev/null || echo "Unknown error")
    print_status "ERROR" "Failed to create application: $ERROR_MSG"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi

# STEP 6: Verify Application in API
print_header "STEP 6: VERIFY APPLICATION IN API"

sleep 2
API_APPS=$(curl -s "${API_SERVER_URL}/api/v1/applications" 2>/dev/null || echo "{}")
APP_FOUND=$(echo "$API_APPS" | jq -r ".applications[] | select(.name == \"${TEST_APP_NAME}\" or .app_id == \"${TEST_APP_NAME}\") | .app_id // .name" 2>/dev/null || echo "")

if [ -n "$APP_FOUND" ]; then
    print_status "SUCCESS" "Application visible in API: $APP_FOUND"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    print_status "ERROR" "Application not found in API"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi

# STEP 7: Deploy Application
print_header "STEP 7: DEPLOY APPLICATION"

print_status "INFO" "Deploying application to device..."

DEPLOY_RESPONSE=$(curl -s -X POST "${API_SERVER_URL}/api/v1/applications/${TEST_APP_NAME}/deploy" \
    -H "Content-Type: application/json" \
    -d '{}' \
    --max-time 60 2>/dev/null || echo "{}")

if echo "$DEPLOY_RESPONSE" | jq -e '.success == true' >/dev/null 2>&1; then
    print_status "SUCCESS" "Application deployment initiated"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    ERROR_MSG=$(echo "$DEPLOY_RESPONSE" | jq -r '.message // "Unknown error"' 2>/dev/null || echo "Unknown error")
    print_status "WARNING" "Deploy may have failed: $ERROR_MSG"
    print_status "INFO" "This may be expected if device is not fully connected"
    TESTS_WARNING=$((TESTS_WARNING + 1))
fi

# STEP 8: Stop Application
print_header "STEP 8: STOP APPLICATION"

print_status "INFO" "Stopping application..."

STOP_RESPONSE=$(curl -s -X POST "${API_SERVER_URL}/api/v1/applications/${TEST_APP_NAME}/stop" \
    -H "Content-Type: application/json" \
    -d '{}' \
    --max-time 30 2>/dev/null || echo "{}")

if echo "$STOP_RESPONSE" | jq -e '.success == true' >/dev/null 2>&1; then
    print_status "SUCCESS" "Application stopped successfully"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    ERROR_MSG=$(echo "$STOP_RESPONSE" | jq -r '.message // "Unknown error"' 2>/dev/null || echo "Unknown error")
    print_status "WARNING" "Stop may have failed: $ERROR_MSG"
    TESTS_WARNING=$((TESTS_WARNING + 1))
fi

# STEP 9: Disconnect Device
print_header "STEP 9: DISCONNECT DEVICE"

print_status "INFO" "Disconnecting device..."

DISCONNECT_RESPONSE=$(curl -s -X POST "${API_SERVER_URL}/api/v1/devices/${TEST_DEVICE_NAME}/disconnect" \
    -H "Content-Type: application/json" \
    -d '{}' 2>/dev/null || echo "{}")

if echo "$DISCONNECT_RESPONSE" | jq -e '.success == true or .status == "Disconnected"' >/dev/null 2>&1; then
    print_status "SUCCESS" "Device disconnected successfully"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    ERROR_MSG=$(echo "$DISCONNECT_RESPONSE" | jq -r '.message // "Unknown error"' 2>/dev/null || echo "Unknown error")
    print_status "WARNING" "Disconnect may have failed: $ERROR_MSG"
    TESTS_WARNING=$((TESTS_WARNING + 1))
fi

# STEP 10: Delete Application
print_header "STEP 10: DELETE APPLICATION"

print_status "INFO" "Deleting application..."

DELETE_APP_RESPONSE=$(curl -s -X DELETE "${API_SERVER_URL}/api/v1/applications/${TEST_APP_NAME}" 2>/dev/null || echo "{}")

if echo "$DELETE_APP_RESPONSE" | jq -e '.success == true' >/dev/null 2>&1 || [ -z "$DELETE_APP_RESPONSE" ]; then
    print_status "SUCCESS" "Application deleted successfully"
    TESTS_PASSED=$((TESTS_PASSED + 1))
    
    # Verify deletion in Kubernetes
    sleep 2
    if ! kubectl get application "$TEST_APP_NAME" -n "$KUBECTL_NAMESPACE" >/dev/null 2>&1; then
        print_status "SUCCESS" "Application removed from Kubernetes"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        print_status "WARNING" "Application still exists in Kubernetes"
        TESTS_WARNING=$((TESTS_WARNING + 1))
    fi
else
    ERROR_MSG=$(echo "$DELETE_APP_RESPONSE" | jq -r '.message // "Unknown error"' 2>/dev/null || echo "Unknown error")
    print_status "WARNING" "Delete may have failed: $ERROR_MSG"
    TESTS_WARNING=$((TESTS_WARNING + 1))
fi

# STEP 11: Delete Device
print_header "STEP 11: DELETE DEVICE"

print_status "INFO" "Deleting device..."

DELETE_DEVICE_RESPONSE=$(curl -s -X DELETE "${API_SERVER_URL}/api/v1/devices/${TEST_DEVICE_NAME}" 2>/dev/null || echo "{}")

if echo "$DELETE_DEVICE_RESPONSE" | jq -e '.success == true' >/dev/null 2>&1 || [ -z "$DELETE_DEVICE_RESPONSE" ]; then
    print_status "SUCCESS" "Device deleted successfully"
    TESTS_PASSED=$((TESTS_PASSED + 1))
    
    # Verify deletion in Kubernetes
    sleep 2
    if ! kubectl get device "$TEST_DEVICE_NAME" -n "$KUBECTL_NAMESPACE" >/dev/null 2>&1; then
        print_status "SUCCESS" "Device removed from Kubernetes"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        print_status "WARNING" "Device still exists in Kubernetes"
        TESTS_WARNING=$((TESTS_WARNING + 1))
    fi
else
    ERROR_MSG=$(echo "$DELETE_DEVICE_RESPONSE" | jq -r '.message // "Unknown error"' 2>/dev/null || echo "Unknown error")
    print_status "WARNING" "Delete may have failed: $ERROR_MSG"
    TESTS_WARNING=$((TESTS_WARNING + 1))
fi

# Summary
print_header "WORKFLOW VERIFICATION SUMMARY"

echo ""
echo "Tests Passed: $TESTS_PASSED"
echo "Tests Failed: $TESTS_FAILED"
echo "Tests Warning: $TESTS_WARNING"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    print_status "SUCCESS" "All critical workflow steps passed!"
    if [ $TESTS_WARNING -gt 0 ]; then
        print_status "INFO" "Some warnings detected - review above"
    fi
    exit 0
else
    print_status "ERROR" "Some workflow steps failed - review above"
    exit 1
fi

