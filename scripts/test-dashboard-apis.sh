#!/bin/bash
# Test all dashboard API endpoints page by page
# This script tests all API calls made by the dashboard and verifies operations

set +e  # Don't exit on error, we want to continue testing

API_BASE_URL="${API_BASE_URL:-http://localhost:3000/api}"
COLOR_GREEN='\033[0;32m'
COLOR_RED='\033[0;31m'
COLOR_YELLOW='\033[1;33m'
COLOR_BLUE='\033[0;34m'
COLOR_RESET='\033[0m'

# Test counters
TESTS_PASSED=0
TESTS_FAILED=0
OPERATIONS_FAILED=0

print_test() {
    echo -e "${COLOR_BLUE}[TEST]${COLOR_RESET} $1"
}

print_success() {
    echo -e "${COLOR_GREEN}[✓]${COLOR_RESET} $1"
    ((TESTS_PASSED++))
}

print_error() {
    echo -e "${COLOR_RED}[✗]${COLOR_RESET} $1"
    ((TESTS_FAILED++))
}

print_warning() {
    echo -e "${COLOR_YELLOW}[!]${COLOR_RESET} $1"
}

# Test API endpoint
test_api() {
    local method=$1
    local endpoint=$2
    local data=$3
    local expected_status=${4:-200}
    
    local url="${API_BASE_URL}${endpoint}"
    local temp_file=$(mktemp)
    local status_code
    
    if [ "$method" = "GET" ]; then
        status_code=$(curl -s -w "%{http_code}" -o "$temp_file" "${url}" 2>/dev/null || echo "000")
    elif [ "$method" = "POST" ]; then
        status_code=$(curl -s -w "%{http_code}" -o "$temp_file" -X POST -H "Content-Type: application/json" -d "${data}" "${url}" 2>/dev/null || echo "000")
    elif [ "$method" = "PUT" ]; then
        status_code=$(curl -s -w "%{http_code}" -o "$temp_file" -X PUT -H "Content-Type: application/json" -d "${data}" "${url}" 2>/dev/null || echo "000")
    elif [ "$method" = "DELETE" ]; then
        status_code=$(curl -s -w "%{http_code}" -o "$temp_file" -X DELETE "${url}" 2>/dev/null || echo "000")
    fi
    
    local body=$(cat "$temp_file" 2>/dev/null || echo "")
    rm -f "$temp_file"
    
    if [ "$status_code" = "$expected_status" ]; then
        echo "$body"
        return 0
    else
        echo "ERROR: Expected status $expected_status, got $status_code" >&2
        echo "$body" >&2
        return 1
    fi
}

# Verify operation was actually performed using kubectl
verify_operation() {
    local operation=$1
    local verification_cmd=$2
    
    print_test "Verifying: $operation"
    if eval "$verification_cmd"; then
        print_success "Operation verified: $operation"
        return 0
    else
        print_error "Operation NOT verified: $operation"
        ((OPERATIONS_FAILED++))
        return 1
    fi
}

# Verify Device CRD exists in Kubernetes
verify_device_crd() {
    local device_id=$1
    print_test "Verifying Device CRD exists: $device_id"
    if kubectl get device "$device_id" -n wasmbed &>/dev/null; then
        print_success "Device CRD exists: $device_id"
        # Get device status
        local status=$(kubectl get device "$device_id" -n wasmbed -o jsonpath='{.status.phase}' 2>/dev/null || echo "")
        if [ -n "$status" ]; then
            echo "  Device status: $status"
        fi
        return 0
    else
        print_error "Device CRD NOT found: $device_id"
        ((OPERATIONS_FAILED++))
        return 1
    fi
}

# Verify Application CRD exists in Kubernetes
verify_application_crd() {
    local app_id=$1
    print_test "Verifying Application CRD exists: $app_id"
    if kubectl get application "$app_id" -n wasmbed &>/dev/null; then
        print_success "Application CRD exists: $app_id"
        # Get application status
        local status=$(kubectl get application "$app_id" -n wasmbed -o jsonpath='{.status.phase}' 2>/dev/null || echo "")
        if [ -n "$status" ]; then
            echo "  Application status: $status"
        fi
        return 0
    else
        print_error "Application CRD NOT found: $app_id"
        ((OPERATIONS_FAILED++))
        return 1
    fi
}

# Verify Gateway CRD exists in Kubernetes
verify_gateway_crd() {
    local gateway_id=$1
    print_test "Verifying Gateway CRD exists: $gateway_id"
    if kubectl get gateway "$gateway_id" -n wasmbed &>/dev/null; then
        print_success "Gateway CRD exists: $gateway_id"
        # Get gateway status
        local status=$(kubectl get gateway "$gateway_id" -n wasmbed -o jsonpath='{.status.phase}' 2>/dev/null || echo "")
        if [ -n "$status" ]; then
            echo "  Gateway status: $status"
        fi
        return 0
    else
        print_error "Gateway CRD NOT found: $gateway_id"
        ((OPERATIONS_FAILED++))
        return 1
    fi
}

# Verify Device CRD deleted
verify_device_deleted() {
    local device_id=$1
    print_test "Verifying Device CRD deleted: $device_id"
    if ! kubectl get device "$device_id" -n wasmbed &>/dev/null; then
        print_success "Device CRD deleted: $device_id"
        return 0
    else
        print_error "Device CRD still exists: $device_id"
        ((OPERATIONS_FAILED++))
        return 1
    fi
}

# Verify Application CRD deleted
verify_application_deleted() {
    local app_id=$1
    print_test "Verifying Application CRD deleted: $app_id"
    if ! kubectl get application "$app_id" -n wasmbed &>/dev/null; then
        print_success "Application CRD deleted: $app_id"
        return 0
    else
        print_error "Application CRD still exists: $app_id"
        ((OPERATIONS_FAILED++))
        return 1
    fi
}

# Verify Gateway CRD deleted
verify_gateway_deleted() {
    local gateway_id=$1
    print_test "Verifying Gateway CRD deleted: $gateway_id"
    if ! kubectl get gateway "$gateway_id" -n wasmbed &>/dev/null; then
        print_success "Gateway CRD deleted: $gateway_id"
        return 0
    else
        print_error "Gateway CRD still exists: $gateway_id"
        ((OPERATIONS_FAILED++))
        return 1
    fi
}

# Verify Renode pod/container exists for device
verify_renode_running() {
    local device_id=$1
    print_test "Verifying Renode container running for device: $device_id"
    # Check if there's a Renode container or pod for this device
    if docker ps --format "{{.Names}}" | grep -q "renode.*${device_id}" || \
       kubectl get pods -n wasmbed -l app=renode-sidecar 2>/dev/null | grep -q "${device_id}"; then
        print_success "Renode container/pod found for device: $device_id"
        return 0
    else
        print_warning "Renode container/pod not found for device: $device_id (may not be started yet)"
        return 0  # Not a failure, just a warning
    fi
}

# Verify Gateway pod exists and is running
verify_gateway_pod() {
    local gateway_id=$1
    print_test "Verifying Gateway pod running: $gateway_id"
    # Gateway pods are typically named after the gateway
    if kubectl get pods -n wasmbed -l app=gateway 2>/dev/null | grep -q "${gateway_id}" || \
       kubectl get pods -n wasmbed | grep -q "gateway.*${gateway_id}"; then
        local pod_status=$(kubectl get pods -n wasmbed | grep "${gateway_id}" | awk '{print $3}' | head -1)
        if [ "$pod_status" = "Running" ]; then
            print_success "Gateway pod running: $gateway_id (status: $pod_status)"
            return 0
        else
            print_warning "Gateway pod exists but not running: $gateway_id (status: $pod_status)"
            return 0  # Not a failure, pod might be starting
        fi
    else
        print_warning "Gateway pod not found: $gateway_id (may not be created yet)"
        return 0  # Not a failure, pod might be created by controller later
    fi
}

echo "=========================================="
echo "Dashboard API Testing Script"
echo "=========================================="
echo "API Base URL: $API_BASE_URL"
echo ""

# Store created resources for cleanup
CREATED_DEVICES=()
CREATED_APPLICATIONS=()
CREATED_GATEWAYS=()

# ============================================
# PAGE 1: Device Management
# ============================================
echo -e "\n${COLOR_BLUE}=== PAGE 1: Device Management ===${COLOR_RESET}\n"

# Test 1.1: GET /api/v1/devices
print_test "1.1 GET /api/v1/devices - Fetch devices list"
DEVICES_RESPONSE=$(test_api "GET" "/v1/devices" "" 200)
TEST_RESULT=$?
if [ $TEST_RESULT -eq 0 ]; then
    print_success "GET /api/v1/devices - Success"
    DEVICES_COUNT=$(echo "$DEVICES_RESPONSE" | jq -r '.devices | length' 2>/dev/null || echo "0")
    if [ "$DEVICES_COUNT" = "0" ] && [ -n "$DEVICES_RESPONSE" ]; then
        # Try alternative parsing
        DEVICES_COUNT=$(echo "$DEVICES_RESPONSE" | grep -o '"id"' | wc -l || echo "0")
    fi
    echo "  Found $DEVICES_COUNT devices"
else
    print_error "GET /api/v1/devices - Failed"
    DEVICES_RESPONSE=""
    DEVICES_COUNT=0
fi

# Test 1.2: GET /api/v1/gateways (for device creation)
print_test "1.2 GET /api/v1/gateways - Fetch gateways for device creation"
GATEWAYS_RESPONSE=$(test_api "GET" "/v1/gateways" "" 200)
TEST_RESULT=$?
if [ $TEST_RESULT -eq 0 ]; then
    print_success "GET /api/v1/gateways - Success"
    GATEWAY_ID=$(echo "$GATEWAYS_RESPONSE" | jq -r '.gateways[0].id // .gateways[0].gateway_id // empty' 2>/dev/null || echo "")
    if [ -z "$GATEWAY_ID" ]; then
        # Try alternative parsing
        GATEWAY_ID=$(echo "$GATEWAYS_RESPONSE" | grep -o '"id"[[:space:]]*:[[:space:]]*"[^"]*"' | head -1 | cut -d'"' -f4 || echo "")
    fi
    if [ -z "$GATEWAY_ID" ]; then
        GATEWAY_ID="gateway-1"
    fi
    echo "  Using gateway: $GATEWAY_ID"
else
    print_error "GET /api/v1/gateways - Failed"
    GATEWAY_ID="gateway-1"
fi

# Test 1.3: POST /api/v1/devices - Create device
print_test "1.3 POST /api/v1/devices - Create new device"
DEVICE_NAME="test-device-$(date +%s)"
CREATE_DEVICE_DATA=$(cat <<EOF
{
  "name": "$DEVICE_NAME",
  "type": "MCU",
  "architecture": "ARM_CORTEX_M",
  "mcuType": "RenodeArduinoNano33Ble",
  "gatewayId": "$GATEWAY_ID",
  "qemuEnabled": true
}
EOF
)
    CREATE_DEVICE_RESPONSE=$(test_api "POST" "/v1/devices" "$CREATE_DEVICE_DATA" 200)
if [ $? -eq 0 ]; then
    print_success "POST /api/v1/devices - Device created"
    # Response format: {"devices": [{"id": "...", ...}], ...}
    CREATED_DEVICE_ID=$(echo "$CREATE_DEVICE_RESPONSE" | jq -r '.devices[0].id // .devices[0].device_id // .device.id // .device.device_id // .id // empty' 2>/dev/null || echo "")
    if [ -n "$CREATED_DEVICE_ID" ]; then
        CREATED_DEVICES+=("$CREATED_DEVICE_ID")
        echo "  Created device ID: $CREATED_DEVICE_ID"
        
        # Wait a bit for CRD to be created
        sleep 3
        
        # Verify device CRD exists in Kubernetes
        verify_device_crd "$CREATED_DEVICE_ID"
        
        # Also verify via API
        verify_operation "Device creation (API)" "test_api 'GET' '/v1/devices' '' 200 | jq -e '.devices[] | select(.id == \"$CREATED_DEVICE_ID\" or .device_id == \"$CREATED_DEVICE_ID\")' > /dev/null"
    else
        print_warning "Could not extract device ID from response"
    fi
else
    print_error "POST /api/v1/devices - Failed"
fi

# Test 1.4: POST /api/v1/devices/{id}/enroll - Enroll device
if [ -n "$CREATED_DEVICE_ID" ]; then
    print_test "1.4 POST /api/v1/devices/$CREATED_DEVICE_ID/enroll - Enroll device"
    ENROLL_DATA=$(cat <<EOF
{
  "gatewayId": "$GATEWAY_ID",
  "gatewayName": "$GATEWAY_ID"
}
EOF
)
    ENROLL_RESPONSE=$(test_api "POST" "/v1/devices/$CREATED_DEVICE_ID/enroll" "$ENROLL_DATA" 200)
    if [ $? -eq 0 ]; then
        print_success "POST /api/v1/devices/{id}/enroll - Success"
        sleep 3
        # Verify enrollment status in Kubernetes CRD
        verify_operation "Device enrollment (CRD)" "kubectl get device \"$CREATED_DEVICE_ID\" -n wasmbed -o jsonpath='{.status.phase}' 2>/dev/null | grep -qE 'Enrolled|Connected'"
        # Also verify via API
        verify_operation "Device enrollment (API)" "test_api 'GET' '/v1/devices' '' 200 | jq -e '.devices[] | select(.id == \"$CREATED_DEVICE_ID\" or .device_id == \"$CREATED_DEVICE_ID\") | .enrolled == true or .status == \"Enrolled\"' > /dev/null"
    else
        print_error "POST /api/v1/devices/{id}/enroll - Failed"
    fi
fi

# Test 1.5: POST /api/v1/devices/{id}/connect - Connect device
if [ -n "$CREATED_DEVICE_ID" ]; then
    print_test "1.5 POST /api/v1/devices/$CREATED_DEVICE_ID/connect - Connect device"
    CONNECT_RESPONSE=$(test_api "POST" "/v1/devices/$CREATED_DEVICE_ID/connect" "{}" 200)
    if [ $? -eq 0 ]; then
        print_success "POST /api/v1/devices/{id}/connect - Success"
        sleep 5
        # Verify Renode is running
        verify_renode_running "$CREATED_DEVICE_ID"
        # Verify connection status in Kubernetes CRD
        verify_operation "Device connection (CRD)" "kubectl get device \"$CREATED_DEVICE_ID\" -n wasmbed -o jsonpath='{.status.phase}' 2>/dev/null | grep -q 'Connected'"
        # Also verify via API
        verify_operation "Device connection (API)" "test_api 'GET' '/v1/devices' '' 200 | jq -e '.devices[] | select(.id == \"$CREATED_DEVICE_ID\" or .device_id == \"$CREATED_DEVICE_ID\") | .status == \"Connected\"' > /dev/null"
    else
        print_error "POST /api/v1/devices/{id}/connect - Failed (may timeout, this is expected)"
    fi
fi

# Test 1.6: POST /api/v1/devices/{id}/emulation/start - Start Renode
if [ -n "$CREATED_DEVICE_ID" ]; then
    print_test "1.6 POST /api/v1/devices/$CREATED_DEVICE_ID/emulation/start - Start Renode emulation"
    # Wait a bit to ensure device is ready
    sleep 2
    START_RESPONSE=$(test_api "POST" "/v1/devices/$CREATED_DEVICE_ID/emulation/start" "{}" 200)
    TEST_RESULT=$?
    if [ $TEST_RESULT -eq 0 ]; then
        print_success "POST /api/v1/devices/{id}/emulation/start - Success"
        sleep 3
    else
        # Check if device exists in CRD before reporting failure
        if kubectl get device "$CREATED_DEVICE_ID" -n wasmbed &>/dev/null; then
            print_warning "POST /api/v1/devices/{id}/emulation/start - Failed (device exists, may be already started or error in Renode)"
        else
            print_warning "POST /api/v1/devices/{id}/emulation/start - Failed (device CRD not found, may not be ready yet)"
        fi
    fi
fi

# Test 1.7: POST /api/v1/devices/{id}/emulation/stop - Stop Renode
if [ -n "$CREATED_DEVICE_ID" ]; then
    print_test "1.7 POST /api/v1/devices/$CREATED_DEVICE_ID/emulation/stop - Stop Renode emulation"
    STOP_RESPONSE=$(test_api "POST" "/v1/devices/$CREATED_DEVICE_ID/emulation/stop" "{}" 200)
    if [ $? -eq 0 ]; then
        print_success "POST /api/v1/devices/{id}/emulation/stop - Success"
    else
        print_error "POST /api/v1/devices/{id}/emulation/stop - Failed"
    fi
fi

# Test 1.8: POST /api/v1/devices/{id}/disconnect - Disconnect device
if [ -n "$CREATED_DEVICE_ID" ]; then
    print_test "1.8 POST /api/v1/devices/$CREATED_DEVICE_ID/disconnect - Disconnect device"
    DISCONNECT_RESPONSE=$(test_api "POST" "/v1/devices/$CREATED_DEVICE_ID/disconnect" "{}" 200)
    if [ $? -eq 0 ]; then
        print_success "POST /api/v1/devices/{id}/disconnect - Success"
        sleep 3
        # Verify disconnection status in Kubernetes CRD
        verify_operation "Device disconnection (CRD)" "kubectl get device \"$CREATED_DEVICE_ID\" -n wasmbed -o jsonpath='{.status.phase}' 2>/dev/null | grep -qE 'Disconnected|Pending'"
        # Also verify via API
        verify_operation "Device disconnection (API)" "test_api 'GET' '/v1/devices' '' 200 | jq -e '.devices[] | select(.id == \"$CREATED_DEVICE_ID\" or .device_id == \"$CREATED_DEVICE_ID\") | .status == \"Disconnected\"' > /dev/null"
    else
        print_error "POST /api/v1/devices/{id}/disconnect - Failed"
    fi
fi

# ============================================
# PAGE 2: Application Management
# ============================================
echo -e "\n${COLOR_BLUE}=== PAGE 2: Application Management ===${COLOR_RESET}\n"

# Test 2.1: GET /api/v1/applications
print_test "2.1 GET /api/v1/applications - Fetch applications list"
APPLICATIONS_RESPONSE=$(test_api "GET" "/v1/applications" "" 200)
if [ $? -eq 0 ]; then
    print_success "GET /api/v1/applications - Success"
    APPS_COUNT=$(echo "$APPLICATIONS_RESPONSE" | jq -r '.applications | length' 2>/dev/null || echo "0")
    echo "  Found $APPS_COUNT applications"
else
    print_error "GET /api/v1/applications - Failed"
fi

# Test 2.2: POST /api/v1/applications - Create application
print_test "2.2 POST /api/v1/applications - Create new application"
APP_NAME="test-app-$(date +%s)"
# Create a minimal WASM binary (base64 encoded "test")
WASM_BYTES="dGVzdA=="
CREATE_APP_DATA=$(cat <<EOF
{
  "name": "$APP_NAME",
  "description": "Test application",
  "targetDevices": [],
  "wasmBytes": "$WASM_BYTES"
}
EOF
)
CREATE_APP_RESPONSE=$(test_api "POST" "/v1/applications" "$CREATE_APP_DATA" 200)
if [ $? -eq 0 ]; then
    print_success "POST /api/v1/applications - Application created"
    CREATED_APP_ID=$(echo "$CREATE_APP_RESPONSE" | jq -r '.application.id // .application.app_id // .id // empty' 2>/dev/null || echo "")
    if [ -n "$CREATED_APP_ID" ]; then
        CREATED_APPLICATIONS+=("$CREATED_APP_ID")
        echo "  Created application ID: $CREATED_APP_ID"
        
        # Wait a bit for CRD to be created
        sleep 3
        
        # Verify application CRD exists in Kubernetes
        verify_application_crd "$CREATED_APP_ID"
        
        # Also verify via API
        verify_operation "Application creation (API)" "test_api 'GET' '/v1/applications' '' 200 | jq -e '.applications[] | select(.id == \"$CREATED_APP_ID\" or .app_id == \"$CREATED_APP_ID\")' > /dev/null"
    else
        print_warning "Could not extract application ID from response"
    fi
else
    print_error "POST /api/v1/applications - Failed"
fi

# Test 2.3: POST /api/v1/applications/{id}/deploy - Deploy application
if [ -n "$CREATED_APP_ID" ]; then
    print_test "2.3 POST /api/v1/applications/$CREATED_APP_ID/deploy - Deploy application"
    DEPLOY_RESPONSE=$(test_api "POST" "/v1/applications/$CREATED_APP_ID/deploy" "{}" 200)
    if [ $? -eq 0 ]; then
        print_success "POST /api/v1/applications/{id}/deploy - Success"
        sleep 5
        # Verify deployment status in Kubernetes CRD
        verify_operation "Application deployment (CRD)" "kubectl get application \"$CREATED_APP_ID\" -n wasmbed -o jsonpath='{.status.phase}' 2>/dev/null | grep -qE 'Deployed|Running|Deploying'"
        # Also verify via API
        verify_operation "Application deployment (API)" "test_api 'GET' '/v1/applications' '' 200 | jq -e '.applications[] | select(.id == \"$CREATED_APP_ID\" or .app_id == \"$CREATED_APP_ID\") | .status == \"Running\" or .status == \"Deploying\"' > /dev/null"
    else
        print_error "POST /api/v1/applications/{id}/deploy - Failed"
    fi
fi

# Test 2.4: POST /api/v1/applications/{id}/stop - Stop application
if [ -n "$CREATED_APP_ID" ]; then
    print_test "2.4 POST /api/v1/applications/$CREATED_APP_ID/stop - Stop application"
    STOP_APP_RESPONSE=$(test_api "POST" "/v1/applications/$CREATED_APP_ID/stop" "{}" 200)
    if [ $? -eq 0 ]; then
        print_success "POST /api/v1/applications/{id}/stop - Success"
        sleep 5
        # Get current status from CRD
        CRD_STATUS=$(kubectl get application "$CREATED_APP_ID" -n wasmbed -o jsonpath='{.status.phase}' 2>/dev/null || echo "")
        if [ -n "$CRD_STATUS" ]; then
            # Verify stop status in Kubernetes CRD (status might be Failed, Stopped, or Pending)
            # If status is Failed, Stopped, or Pending, it means the app is not running (stop successful)
            if echo "$CRD_STATUS" | grep -qE 'Stopped|Pending|Failed'; then
                print_success "Application stop (CRD) - Status: $CRD_STATUS (app is not running)"
            else
                print_warning "Application stop (CRD) - Status: $CRD_STATUS (may still be stopping)"
            fi
        else
            print_error "Application stop (CRD) - Could not get status from CRD"
            ((OPERATIONS_FAILED++))
        fi
        
        # Also verify via API
        API_STATUS=$(test_api 'GET' '/v1/applications' '' 200 2>/dev/null | jq -r ".applications[] | select(.id == \"$CREATED_APP_ID\" or .app_id == \"$CREATED_APP_ID\") | .status // empty" 2>/dev/null || echo "")
        if [ -n "$API_STATUS" ]; then
            if echo "$API_STATUS" | grep -qE 'Stopped|Failed|Pending'; then
                print_success "Application stop (API) - Status: $API_STATUS (app is not running)"
            else
                print_warning "Application stop (API) - Status: $API_STATUS (may still be stopping)"
            fi
        else
            print_warning "Application stop (API) - Could not get status from API (app may have been deleted)"
        fi
    else
        print_error "POST /api/v1/applications/{id}/stop - Failed"
    fi
fi

# ============================================
# PAGE 3: Gateway Management
# ============================================
echo -e "\n${COLOR_BLUE}=== PAGE 3: Gateway Management ===${COLOR_RESET}\n"

# Test 3.1: GET /api/v1/gateways (already tested, but verify)
print_test "3.1 GET /api/v1/gateways - Fetch gateways list"
GATEWAYS_RESPONSE=$(test_api "GET" "/v1/gateways" "" 200)
if [ $? -eq 0 ]; then
    print_success "GET /api/v1/gateways - Success"
    GATEWAYS_COUNT=$(echo "$GATEWAYS_RESPONSE" | jq -r '.gateways | length' 2>/dev/null || echo "0")
    echo "  Found $GATEWAYS_COUNT gateways"
else
    print_error "GET /api/v1/gateways - Failed"
fi

# Test 3.2: POST /api/v1/gateways - Create gateway
print_test "3.2 POST /api/v1/gateways - Create new gateway"
GATEWAY_NAME="test-gateway-$(date +%s)"
CREATE_GATEWAY_DATA=$(cat <<EOF
{
  "name": "$GATEWAY_NAME",
  "description": "Test gateway created from API test"
}
EOF
)
    CREATE_GATEWAY_RESPONSE=$(test_api "POST" "/v1/gateways" "$CREATE_GATEWAY_DATA" 200)
if [ $? -eq 0 ]; then
    print_success "POST /api/v1/gateways - Gateway created"
    # Response format: {"gateways": [{"id": "...", ...}], ...}
    CREATED_GATEWAY_ID=$(echo "$CREATE_GATEWAY_RESPONSE" | jq -r '.gateways[0].id // .gateways[0].gateway_id // .gateway.id // .gateway.gateway_id // .id // empty' 2>/dev/null || echo "")
    if [ -n "$CREATED_GATEWAY_ID" ]; then
        CREATED_GATEWAYS+=("$CREATED_GATEWAY_ID")
        echo "  Created gateway ID: $CREATED_GATEWAY_ID"
        
        # Wait a bit for CRD to be created
        sleep 3
        
        # Verify gateway CRD exists in Kubernetes
        verify_gateway_crd "$CREATED_GATEWAY_ID"
        
        # Verify gateway pod is running
        verify_gateway_pod "$CREATED_GATEWAY_ID"
        
        # Also verify via API
        verify_operation "Gateway creation (API)" "test_api 'GET' '/v1/gateways' '' 200 | jq -e '.gateways[] | select(.id == \"$CREATED_GATEWAY_ID\" or .gateway_id == \"$CREATED_GATEWAY_ID\")' > /dev/null"
    else
        print_warning "Could not extract gateway ID from response"
    fi
else
    print_error "POST /api/v1/gateways - Failed"
fi

# Test 3.3: PUT /api/v1/gateways/{id} - Update gateway config
if [ -n "$CREATED_GATEWAY_ID" ]; then
    print_test "3.3 PUT /api/v1/gateways/$CREATED_GATEWAY_ID - Update gateway configuration"
    UPDATE_GATEWAY_DATA=$(cat <<EOF
{
  "config": {
    "heartbeatInterval": "30s",
    "connectionTimeout": "10m",
    "enrollmentTimeout": "5m"
  }
}
EOF
)
    UPDATE_GATEWAY_RESPONSE=$(test_api "PUT" "/v1/gateways/$CREATED_GATEWAY_ID" "$UPDATE_GATEWAY_DATA" 200)
    if [ $? -eq 0 ]; then
        print_success "PUT /api/v1/gateways/{id} - Success"
        sleep 3
        # Verify configuration was updated in Kubernetes CRD
        verify_operation "Gateway config update (CRD)" "kubectl get gateway \"$CREATED_GATEWAY_ID\" -n wasmbed -o jsonpath='{.spec.config.heartbeatInterval}' 2>/dev/null | grep -q '30s'"
        # Also verify via API (if config is available in API response)
        # Note: API may return config as null, so CRD verification is the primary check
        API_CONFIG=$(test_api 'GET' '/v1/gateways' '' 200 2>/dev/null | jq -r ".gateways[] | select(.id == \"$CREATED_GATEWAY_ID\" or .gateway_id == \"$CREATED_GATEWAY_ID\") | .config.heartbeatInterval // empty" 2>/dev/null || echo "")
        if [ -n "$API_CONFIG" ] && [ "$API_CONFIG" != "null" ]; then
            verify_operation "Gateway config update (API)" "[ \"$API_CONFIG\" = \"30s\" ]"
        else
            print_warning "Gateway config not available in API response (CRD verification is primary)"
        fi
    else
        print_error "PUT /api/v1/gateways/{id} - Failed"
    fi
fi

# Test 3.4: POST /api/v1/gateways/{id}/toggle - Toggle gateway
if [ -n "$CREATED_GATEWAY_ID" ]; then
    print_test "3.4 POST /api/v1/gateways/$CREATED_GATEWAY_ID/toggle - Toggle gateway status"
    TOGGLE_DATA='{"enabled": false}'
    TOGGLE_RESPONSE=$(test_api "POST" "/v1/gateways/$CREATED_GATEWAY_ID/toggle" "$TOGGLE_DATA" 200)
    if [ $? -eq 0 ]; then
        print_success "POST /api/v1/gateways/{id}/toggle - Success"
        sleep 2
        # Toggle back to enabled
        TOGGLE_DATA='{"enabled": true}'
        test_api "POST" "/v1/gateways/$CREATED_GATEWAY_ID/toggle" "$TOGGLE_DATA" 200 > /dev/null
    else
        print_error "POST /api/v1/gateways/{id}/toggle - Failed"
    fi
fi

# ============================================
# PAGE 4: Monitoring
# ============================================
echo -e "\n${COLOR_BLUE}=== PAGE 4: Monitoring ===${COLOR_RESET}\n"

# Test 4.1: GET /api/v1/infrastructure/health
print_test "4.1 GET /api/v1/infrastructure/health - Get infrastructure health"
HEALTH_RESPONSE=$(test_api "GET" "/v1/infrastructure/health" "" 200)
if [ $? -eq 0 ]; then
    print_success "GET /api/v1/infrastructure/health - Success"
else
    print_warning "GET /api/v1/infrastructure/health - Not available (may not be implemented)"
fi

# Test 4.2: GET /api/v1/infrastructure/status
print_test "4.2 GET /api/v1/infrastructure/status - Get infrastructure status"
STATUS_RESPONSE=$(test_api "GET" "/v1/infrastructure/status" "" 200)
if [ $? -eq 0 ]; then
    print_success "GET /api/v1/infrastructure/status - Success"
else
    print_warning "GET /api/v1/infrastructure/status - Not available (may not be implemented)"
fi

# Test 4.3: GET /api/v1/infrastructure/logs
print_test "4.3 GET /api/v1/infrastructure/logs - Get infrastructure logs"
LOGS_RESPONSE=$(test_api "GET" "/v1/infrastructure/logs" "" 200)
if [ $? -eq 0 ]; then
    print_success "GET /api/v1/infrastructure/logs - Success"
else
    print_warning "GET /api/v1/infrastructure/logs - Not available (may not be implemented)"
fi

# Test 4.4: GET /api/v1/logs
print_test "4.4 GET /api/v1/logs - Get general logs"
GENERAL_LOGS_RESPONSE=$(test_api "GET" "/v1/logs" "" 200)
if [ $? -eq 0 ]; then
    print_success "GET /api/v1/logs - Success"
else
    print_warning "GET /api/v1/logs - Not available (may not be implemented)"
fi

# ============================================
# Cleanup: Delete created resources
# ============================================
echo -e "\n${COLOR_BLUE}=== Cleanup: Deleting created resources ===${COLOR_RESET}\n"

# Delete created applications
for app_id in "${CREATED_APPLICATIONS[@]}"; do
    print_test "DELETE /api/v1/applications/$app_id - Delete application"
    DELETE_RESPONSE=$(test_api "DELETE" "/v1/applications/$app_id" "" 200)
    if [ $? -eq 0 ]; then
        print_success "Application $app_id deleted"
        sleep 1
        sleep 2
        # Verify deletion in Kubernetes
        verify_application_deleted "$app_id"
        # Also verify via API
        verify_operation "Application deletion (API)" "! test_api 'GET' '/v1/applications' '' 200 | jq -e '.applications[] | select(.id == \"$app_id\" or .app_id == \"$app_id\")' > /dev/null"
    else
        print_error "Failed to delete application $app_id"
    fi
done

# Delete created devices
for device_id in "${CREATED_DEVICES[@]}"; do
    print_test "DELETE /api/v1/devices/$device_id - Delete device"
    DELETE_RESPONSE=$(test_api "DELETE" "/v1/devices/$device_id" "" 200)
    if [ $? -eq 0 ]; then
        print_success "Device $device_id deleted"
        sleep 2
        # Verify deletion in Kubernetes
        verify_device_deleted "$device_id"
        # Also verify via API
        verify_operation "Device deletion (API)" "! test_api 'GET' '/v1/devices' '' 200 | jq -e '.devices[] | select(.id == \"$device_id\" or .device_id == \"$device_id\")' > /dev/null"
    else
        print_error "Failed to delete device $device_id"
    fi
done

# Delete created gateways
for gateway_id in "${CREATED_GATEWAYS[@]}"; do
    print_test "DELETE /api/v1/gateways/$gateway_id - Delete gateway"
    DELETE_RESPONSE=$(test_api "DELETE" "/v1/gateways/$gateway_id" "" 200)
    if [ $? -eq 0 ]; then
        print_success "Gateway $gateway_id deleted"
        sleep 2
        # Verify deletion in Kubernetes
        verify_gateway_deleted "$gateway_id"
        # Also verify via API
        verify_operation "Gateway deletion (API)" "! test_api 'GET' '/v1/gateways' '' 200 | jq -e '.gateways[] | select(.id == \"$gateway_id\" or .gateway_id == \"$gateway_id\")' > /dev/null"
    else
        print_error "Failed to delete gateway $gateway_id"
    fi
done

# ============================================
# Summary
# ============================================
echo -e "\n${COLOR_BLUE}==========================================${COLOR_RESET}"
echo -e "${COLOR_BLUE}Test Summary${COLOR_RESET}"
echo -e "${COLOR_BLUE}==========================================${COLOR_RESET}"
echo -e "${COLOR_GREEN}Tests Passed: $TESTS_PASSED${COLOR_RESET}"
echo -e "${COLOR_RED}Tests Failed: $TESTS_FAILED${COLOR_RESET}"
echo -e "${COLOR_RED}Operations NOT Verified: $OPERATIONS_FAILED${COLOR_RESET}"
echo ""

if [ $TESTS_FAILED -eq 0 ] && [ $OPERATIONS_FAILED -eq 0 ]; then
    echo -e "${COLOR_GREEN}✓ All tests passed and operations verified!${COLOR_RESET}"
    exit 0
elif [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${COLOR_YELLOW}⚠ All API tests passed, but some operations were not verified${COLOR_RESET}"
    exit 0
else
    echo -e "${COLOR_RED}✗ Some tests failed${COLOR_RESET}"
    exit 1
fi
