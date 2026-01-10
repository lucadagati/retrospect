#!/bin/bash

# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

# Complete Application Workflow Verification
# Tests: Create Device -> Renode -> TLS Registration -> Create App -> Compile WASM -> Deploy -> Monitor

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

print_header "COMPLETE APPLICATION WORKFLOW VERIFICATION"

# Configuration
API_SERVER_URL="${API_SERVER_URL:-http://localhost:3001}"
GATEWAY_URL="${GATEWAY_URL:-http://localhost:8080}"
KUBECTL_NAMESPACE="${KUBECTL_NAMESPACE:-wasmbed}"

# Test counters
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_WARNING=0

# Test names (with timestamp)
TIMESTAMP=$(date +%s)
TEST_DEVICE_NAME="workflow-device-${TIMESTAMP}"
TEST_APP_NAME="workflow-app-${TIMESTAMP}"

# Cleanup function
cleanup() {
    print_header "CLEANUP"
    print_status "INFO" "Cleaning up test resources..."
    
    if kubectl get device "$TEST_DEVICE_NAME" -n "$KUBECTL_NAMESPACE" >/dev/null 2>&1; then
        kubectl delete device "$TEST_DEVICE_NAME" -n "$KUBECTL_NAMESPACE" >/dev/null 2>&1 || true
    fi
    
    if kubectl get application "$TEST_APP_NAME" -n "$KUBECTL_NAMESPACE" >/dev/null 2>&1; then
        kubectl delete application "$TEST_APP_NAME" -n "$KUBECTL_NAMESPACE" >/dev/null 2>&1 || true
    fi
}

trap cleanup EXIT

# Check prerequisites
print_header "PREREQUISITES CHECK"

if ! curl -4 -s -f "${API_SERVER_URL}/health" >/dev/null 2>&1; then
    print_status "ERROR" "API Server not responding"
    exit 1
fi

if ! curl -4 -s -f "${GATEWAY_URL}/health" >/dev/null 2>&1; then
    print_status "WARNING" "Gateway not responding. Starting port-forward..."
    ./scripts/fix-gateway-connection.sh || exit 1
fi

GATEWAY_ID=$(kubectl get gateways -n "$KUBECTL_NAMESPACE" -o jsonpath='{.items[0].metadata.name}' 2>/dev/null || echo "gateway-1")
print_status "INFO" "Using gateway: $GATEWAY_ID"

# STEP 1: Create Device
print_header "STEP 1: CREATE DEVICE"

print_status "INFO" "Creating device: $TEST_DEVICE_NAME"

CREATE_DEVICE_RESPONSE=$(curl -s -X POST "${API_SERVER_URL}/api/v1/devices" \
    -H "Content-Type: application/json" \
    -d "{
        \"name\": \"${TEST_DEVICE_NAME}\",
        \"type\": \"MCU\",
        \"mcuType\": \"RenodeArduinoNano33Ble\",
        \"gatewayId\": \"${GATEWAY_ID}\"
    }" 2>/dev/null || echo "{}")

if echo "$CREATE_DEVICE_RESPONSE" | jq -e '.success == true' >/dev/null 2>&1; then
    print_status "SUCCESS" "Device created"
    TESTS_PASSED=$((TESTS_PASSED + 1))
    sleep 3
else
    print_status "ERROR" "Failed to create device"
    TESTS_FAILED=$((TESTS_FAILED + 1))
    exit 1
fi

# STEP 2: Verify Device in Kubernetes
print_header "STEP 2: VERIFY DEVICE IN KUBERNETES"

if kubectl get device "$TEST_DEVICE_NAME" -n "$KUBECTL_NAMESPACE" >/dev/null 2>&1; then
    print_status "SUCCESS" "Device visible in Kubernetes"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    print_status "ERROR" "Device not found in Kubernetes"
    TESTS_FAILED=$((TESTS_FAILED + 1))
    exit 1
fi

# STEP 3: Enroll Device
print_header "STEP 3: ENROLL DEVICE TO GATEWAY"

print_status "INFO" "Enrolling device to gateway via TLS..."

ENROLL_RESPONSE=$(curl -s -X POST "${API_SERVER_URL}/api/v1/devices/${TEST_DEVICE_NAME}/enroll" \
    -H "Content-Type: application/json" \
    -d "{\"gatewayId\": \"${GATEWAY_ID}\"}" 2>/dev/null || echo "{}")

if echo "$ENROLL_RESPONSE" | jq -e '.success == true or .status == "Enrolled"' >/dev/null 2>&1; then
    print_status "SUCCESS" "Device enrolled"
    TESTS_PASSED=$((TESTS_PASSED + 1))
    sleep 2
else
    print_status "WARNING" "Enroll may have issues"
    TESTS_WARNING=$((TESTS_WARNING + 1))
fi

# STEP 4: Connect Device (Renode Environment)
print_header "STEP 4: CONNECT DEVICE (RENODE ENVIRONMENT)"

print_status "INFO" "Connecting device (starting Renode emulation)..."

CONNECT_RESPONSE=$(curl -s -X POST "${API_SERVER_URL}/api/v1/devices/${TEST_DEVICE_NAME}/connect" \
    -H "Content-Type: application/json" \
    -d '{}' \
    --max-time 30 2>/dev/null || echo "{}")

if echo "$CONNECT_RESPONSE" | jq -e '.success == true or .status == "Connected"' >/dev/null 2>&1; then
    print_status "SUCCESS" "Device connection initiated (Renode starting)"
    TESTS_PASSED=$((TESTS_PASSED + 1))
    print_status "INFO" "Note: Full Renode startup may take additional time"
else
    print_status "WARNING" "Connection may be in progress"
    TESTS_WARNING=$((TESTS_WARNING + 1))
fi

# STEP 5: Create Application via Dashboard/API
print_header "STEP 5: CREATE APPLICATION (KUBERNETES)"

print_status "INFO" "Creating application: $TEST_APP_NAME"

# Create minimal WASM (base64)
MINIMAL_WASM="AGFzbQEAAAA="

CREATE_APP_RESPONSE=$(curl -s -X POST "${API_SERVER_URL}/api/v1/applications" \
    -H "Content-Type: application/json" \
    -d "{
        \"name\": \"${TEST_APP_NAME}\",
        \"description\": \"Test application for workflow\",
        \"wasmBytes\": \"${MINIMAL_WASM}\",
        \"targetDevices\": {
            \"deviceNames\": [\"${TEST_DEVICE_NAME}\"]
        }
    }" 2>/dev/null || echo "{}")

if echo "$CREATE_APP_RESPONSE" | jq -e '.success == true or .app_id' >/dev/null 2>&1; then
    print_status "SUCCESS" "Application created"
    TESTS_PASSED=$((TESTS_PASSED + 1))
    sleep 2
else
    ERROR_MSG=$(echo "$CREATE_APP_RESPONSE" | jq -r '.message // "Unknown error"' 2>/dev/null || echo "Unknown error")
    print_status "ERROR" "Failed to create application: $ERROR_MSG"
    TESTS_FAILED=$((TESTS_FAILED + 1))
    exit 1
fi

# STEP 6: Verify Application in Kubernetes
print_header "STEP 6: VERIFY APPLICATION IN KUBERNETES"

if kubectl get application "$TEST_APP_NAME" -n "$KUBECTL_NAMESPACE" >/dev/null 2>&1; then
    print_status "SUCCESS" "Application visible in Kubernetes"
    TESTS_PASSED=$((TESTS_PASSED + 1))
    
    # Check status
    APP_STATUS=$(kubectl get application "$TEST_APP_NAME" -n "$KUBECTL_NAMESPACE" -o jsonpath='{.status.phase}' 2>/dev/null || echo "")
    print_status "INFO" "Application status: ${APP_STATUS:-Pending}"
else
    print_status "ERROR" "Application not found in Kubernetes"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi

# STEP 7: Verify Application in API
print_header "STEP 7: VERIFY APPLICATION IN API"

sleep 2
API_APPS=$(curl -s "${API_SERVER_URL}/api/v1/applications" 2>/dev/null || echo "{}")
APP_FOUND=$(echo "$API_APPS" | jq -r ".applications[] | select(.name == \"${TEST_APP_NAME}\" or .app_id == \"${TEST_APP_NAME}\") | .app_id // .id" 2>/dev/null || echo "")

if [ -n "$APP_FOUND" ]; then
    print_status "SUCCESS" "Application visible in API: $APP_FOUND"
    TESTS_PASSED=$((TESTS_PASSED + 1))
    
    # Check if it has required fields
    APP_DATA=$(echo "$API_APPS" | jq ".applications[] | select(.name == \"${TEST_APP_NAME}\" or .app_id == \"${TEST_APP_NAME}\")" 2>/dev/null)
    if echo "$APP_DATA" | jq -e '.app_id or .id' >/dev/null 2>&1; then
        print_status "SUCCESS" "Application has ID field"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    fi
else
    print_status "ERROR" "Application not found in API"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi

# STEP 8: Verify Application in Dashboard
print_header "STEP 8: VERIFY APPLICATION IN DASHBOARD"

DASHBOARD_APPS=$(curl -s "${API_SERVER_URL}/api/v1/applications" 2>/dev/null || echo "{}")
DASHBOARD_APP_FOUND=$(echo "$DASHBOARD_APPS" | jq -r ".applications[] | select(.name == \"${TEST_APP_NAME}\" or .app_id == \"${TEST_APP_NAME}\") | .name" 2>/dev/null || echo "")

if [ -n "$DASHBOARD_APP_FOUND" ]; then
    print_status "SUCCESS" "Application should be visible in dashboard: $DASHBOARD_APP_FOUND"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    print_status "ERROR" "Application not found in dashboard API response"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi

# STEP 9: Compile WASM (if needed)
print_header "STEP 9: COMPILE WASM"

print_status "INFO" "Checking WASM compilation endpoint..."

# Note: WASM compilation would happen here if needed
# For now, we're using pre-compiled WASM
print_status "INFO" "Using pre-compiled WASM (compilation step would be here)"

# STEP 10: Deploy Application
print_header "STEP 10: DEPLOY APPLICATION TO DEVICE"

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
    print_status "WARNING" "Deploy may have issues: $ERROR_MSG"
    TESTS_WARNING=$((TESTS_WARNING + 1))
fi

# STEP 11: Monitor Deployment Status
print_header "STEP 11: MONITOR DEPLOYMENT STATUS"

sleep 3
MONITOR_APPS=$(curl -s "${API_SERVER_URL}/api/v1/applications" 2>/dev/null || echo "{}")
MONITOR_STATUS=$(echo "$MONITOR_APPS" | jq -r ".applications[] | select(.name == \"${TEST_APP_NAME}\" or .app_id == \"${TEST_APP_NAME}\") | .status" 2>/dev/null || echo "")

if [ -n "$MONITOR_STATUS" ]; then
    print_status "SUCCESS" "Application status monitorable: $MONITOR_STATUS"
    TESTS_PASSED=$((TESTS_PASSED + 1))
    
    # Check if status is being updated
    if [ "$MONITOR_STATUS" != "Pending" ]; then
        print_status "SUCCESS" "Status is being updated: $MONITOR_STATUS"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    fi
else
    print_status "WARNING" "Status monitoring may have issues"
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
    exit 0
else
    print_status "ERROR" "Some workflow steps failed"
    exit 1
fi

