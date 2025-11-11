#!/bin/bash

# Script per testare tutti gli endpoint API della dashboard

API_BASE="http://localhost:3001"
PASSED=0
FAILED=0
MISSING=0

echo "=== TEST COMPLETO API ENDPOINTS ==="
echo ""

test_endpoint() {
    local method=$1
    local endpoint=$2
    local data=$3
    local description=$4
    
    echo -n "Testing $method $endpoint ... "
    
    if [ "$method" = "GET" ]; then
        response=$(curl -s -w "\n%{http_code}" "$API_BASE$endpoint" 2>&1)
    elif [ "$method" = "POST" ]; then
        response=$(curl -s -w "\n%{http_code}" -X POST -H "Content-Type: application/json" -d "$data" "$API_BASE$endpoint" 2>&1)
    elif [ "$method" = "PUT" ]; then
        response=$(curl -s -w "\n%{http_code}" -X PUT -H "Content-Type: application/json" -d "$data" "$API_BASE$endpoint" 2>&1)
    elif [ "$method" = "DELETE" ]; then
        response=$(curl -s -w "\n%{http_code}" -X DELETE "$API_BASE$endpoint" 2>&1)
    fi
    
    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n-1)
    
    if [ "$http_code" = "200" ] || [ "$http_code" = "201" ] || [ "$http_code" = "204" ]; then
        echo "✅ PASSED ($http_code)"
        ((PASSED++))
        return 0
    elif [ "$http_code" = "404" ]; then
        echo "❌ MISSING ($http_code)"
        ((MISSING++))
        echo "   Description: $description"
        return 1
    else
        echo "⚠️  FAILED ($http_code)"
        echo "   Response: $body"
        ((FAILED++))
        return 1
    fi
}

# DeviceManagement.js endpoints
echo "=== DeviceManagement.js Endpoints ==="
test_endpoint "GET" "/api/v1/devices" "" "Lista dispositivi"
test_endpoint "POST" "/api/v1/devices" '{"name":"test-device","type":"MCU","architecture":"ARM_CORTEX_M","mcuType":"RenodeArduinoNano33Ble","publicKey":"auto-generated","gatewayId":"gateway-1","qemuEnabled":true}' "Crea dispositivo"
test_endpoint "GET" "/api/v1/devices" "" "Lista dispositivi (verifica creazione)"
DEVICE_ID=$(curl -s "$API_BASE/api/v1/devices" | jq -r '.devices[0].id // .devices[0].device_id // empty' 2>/dev/null)
if [ -n "$DEVICE_ID" ]; then
    # Get gateway info for enrollment
    GATEWAY_ID=$(curl -s "$API_BASE/api/v1/gateways" | jq -r '.gateways[0].id // .gateways[0].gateway_id // "gateway-1"' 2>/dev/null)
    GATEWAY_NAME=$(curl -s "$API_BASE/api/v1/gateways" | jq -r '.gateways[0].name // .gateways[0].gateway_id // "gateway-1"' 2>/dev/null)
    test_endpoint "POST" "/api/v1/devices/$DEVICE_ID/enroll" "{\"gatewayId\":\"$GATEWAY_ID\",\"gatewayName\":\"$GATEWAY_NAME\"}" "Enroll dispositivo"
    test_endpoint "POST" "/api/v1/devices/$DEVICE_ID/connect" '{}' "Connetti dispositivo"
    test_endpoint "POST" "/api/v1/devices/$DEVICE_ID/disconnect" '{}' "Disconnetti dispositivo"
    test_endpoint "POST" "/api/v1/devices/$DEVICE_ID/renode/start" '{}' "Avvia Renode"
    test_endpoint "POST" "/api/v1/devices/$DEVICE_ID/renode/stop" '{}' "Ferma Renode"
    test_endpoint "DELETE" "/api/v1/devices/$DEVICE_ID" "" "Elimina dispositivo"
else
    echo "⚠️  Cannot test device-specific endpoints: no device ID found"
fi
test_endpoint "GET" "/api/v1/renode/devices" "" "Lista dispositivi Renode"
test_endpoint "GET" "/api/v1/gateways" "" "Lista gateway"

echo ""

# GatewayManagement.js endpoints
echo "=== GatewayManagement.js Endpoints ==="
test_endpoint "GET" "/api/v1/gateways" "" "Lista gateway"
test_endpoint "POST" "/api/v1/gateways" '{"name":"test-gateway","endpoint":"127.0.0.1"}' "Crea gateway"
GATEWAY_ID=$(curl -s "$API_BASE/api/v1/gateways" | jq -r '.gateways[0].id // .gateways[0].gateway_id // empty' 2>/dev/null)
if [ -n "$GATEWAY_ID" ]; then
    test_endpoint "PUT" "/api/v1/gateways/$GATEWAY_ID" '{"heartbeatInterval":"30s"}' "Aggiorna configurazione gateway"
    test_endpoint "POST" "/api/v1/gateways/$GATEWAY_ID/toggle" '{"enabled":true}' "Toggle gateway"
    test_endpoint "DELETE" "/api/v1/gateways/$GATEWAY_ID" "" "Elimina gateway"
else
    echo "⚠️  Cannot test gateway-specific endpoints: no gateway ID found"
fi

echo ""

# ApplicationManagement.js endpoints
echo "=== ApplicationManagement.js Endpoints ==="
test_endpoint "GET" "/api/v1/applications" "" "Lista applicazioni"
# Get a device ID for targetDevices
TARGET_DEVICE_ID=$(curl -s "$API_BASE/api/v1/devices" | jq -r '.devices[0].id // .devices[0].device_id // "test-device"' 2>/dev/null)
test_endpoint "POST" "/api/v1/applications" "{\"name\":\"test-app\",\"description\":\"Test app\",\"wasmBytes\":\"dGVzdA==\",\"targetDevices\":{\"deviceNames\":[\"$TARGET_DEVICE_ID\"]}}" "Crea applicazione"
APP_ID=$(curl -s "$API_BASE/api/v1/applications" | jq -r '.applications[0].id // .applications[0].app_id // empty' 2>/dev/null)
if [ -n "$APP_ID" ]; then
    test_endpoint "POST" "/api/v1/applications/$APP_ID/deploy" '{}' "Deploy applicazione"
    test_endpoint "POST" "/api/v1/applications/$APP_ID/stop" '{}' "Ferma applicazione"
    test_endpoint "DELETE" "/api/v1/applications/$APP_ID" "" "Elimina applicazione"
else
    echo "⚠️  Cannot test application-specific endpoints: no application ID found"
fi

echo ""

# Dashboard.js endpoints
echo "=== Dashboard.js Endpoints ==="
test_endpoint "GET" "/api/v1/devices" "" "Statistiche dispositivi"
test_endpoint "GET" "/api/v1/applications" "" "Statistiche applicazioni"
test_endpoint "GET" "/api/v1/gateways" "" "Statistiche gateway"

echo ""

# Monitoring.js endpoints
echo "=== Monitoring.js Endpoints ==="
test_endpoint "GET" "/api/v1/devices" "" "Metriche dispositivi"
test_endpoint "GET" "/api/v1/applications" "" "Metriche applicazioni"
test_endpoint "GET" "/api/v1/gateways" "" "Metriche gateway"
test_endpoint "GET" "/api/v1/infrastructure/health" "" "Health check infrastruttura"
test_endpoint "GET" "/api/v1/infrastructure/status" "" "Status infrastruttura"
test_endpoint "GET" "/api/v1/infrastructure/logs" "" "Log infrastruttura"
test_endpoint "GET" "/api/v1/logs" "" "Log generali"

echo ""

# NetworkTopology.js endpoints
echo "=== NetworkTopology.js Endpoints ==="
test_endpoint "GET" "/api/v1/status" "" "Status sistema"
test_endpoint "GET" "/api/v1/gateways" "" "Topologia gateway"
test_endpoint "GET" "/api/v1/devices" "" "Topologia dispositivi"

echo ""

# GuidedDeployment.js endpoints
echo "=== GuidedDeployment.js Endpoints ==="
test_endpoint "GET" "/api/v1/devices" "" "Dispositivi disponibili"
test_endpoint "POST" "/api/v1/compile" '{"code":"fn main() {}","language":"rust"}' "Compila codice"
TARGET_DEVICE_ID_DEPLOY=$(curl -s "$API_BASE/api/v1/devices" | jq -r '.devices[0].id // .devices[0].device_id // "test-device"' 2>/dev/null)
test_endpoint "POST" "/api/v1/applications" "{\"name\":\"test-deploy\",\"wasmBytes\":\"dGVzdA==\",\"targetDevices\":{\"deviceNames\":[\"$TARGET_DEVICE_ID_DEPLOY\"]}}" "Deploy applicazione"

echo ""

# InitialConfiguration.js endpoints
echo "=== InitialConfiguration.js Endpoints ==="
test_endpoint "GET" "/api/v1/status" "" "Verifica infrastruttura"
test_endpoint "GET" "/api/v1/devices" "" "Verifica dispositivi"
test_endpoint "GET" "/api/v1/applications" "" "Verifica applicazioni"
test_endpoint "GET" "/api/v1/gateways" "" "Verifica gateway"
test_endpoint "POST" "/api/v1/terminal/execute" '{"command":"echo test"}' "Esecuzione comandi"
test_endpoint "POST" "/api/v1/gateways" '{"name":"test-gateway-2","endpoint":"127.0.0.1"}' "Deploy gateway"
test_endpoint "POST" "/api/v1/devices" '{"name":"test-device-2","type":"MCU","architecture":"ARM_CORTEX_M","mcuType":"RenodeArduinoNano33Ble","publicKey":"auto-generated","gatewayId":"gateway-1","qemuEnabled":true}' "Deploy dispositivi"

echo ""

# Terminal.js endpoints
echo "=== Terminal.js Endpoints ==="
test_endpoint "POST" "/api/v1/terminal/execute" '{"command":"kubectl get pods -n wasmbed"}' "Esecuzione comandi"
test_endpoint "GET" "/api/v1/devices" "" "Legacy devices"
test_endpoint "GET" "/api/v1/applications" "" "Legacy applications"
test_endpoint "GET" "/api/v1/gateways" "" "Legacy gateways"
test_endpoint "GET" "/api/v1/pods" "" "Legacy pods"
test_endpoint "GET" "/api/v1/services" "" "Legacy services"
test_endpoint "GET" "/api/v1/metrics" "" "Legacy metrics"
test_endpoint "GET" "/api/v1/logs" "" "Legacy logs"

echo ""
echo "=== RISULTATI ==="
echo "✅ Passed: $PASSED"
echo "⚠️  Failed: $FAILED"
echo "❌ Missing: $MISSING"
echo "Total: $((PASSED + FAILED + MISSING))"
echo ""

if [ $MISSING -gt 0 ] || [ $FAILED -gt 0 ]; then
    exit 1
else
    exit 0
fi

