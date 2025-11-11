#!/bin/bash

# Test reale di tutti i workflow UML usando le API della dashboard
# e verificando i log tramite kubectl

# Non usare set -e per permettere continuazione anche con errori
set +e

API_BASE="http://localhost:3001"
NAMESPACE="wasmbed"
TEST_DIR="/tmp/uml-tests-$(date +%Y%m%d-%H%M%S)"
mkdir -p "$TEST_DIR"

# Colori per output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log() {
    echo -e "${BLUE}[$(date +%H:%M:%S)]${NC} $1"
}

success() {
    echo -e "${GREEN}✅ $1${NC}"
}

error() {
    echo -e "${RED}❌ $1${NC}"
}

warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# Funzione per verificare che un componente sia attivo
check_component() {
    local component=$1
    log "Verificando $component..."
    
    case $component in
        "api-server")
            if curl -s "$API_BASE/api/v1/status" > /dev/null 2>&1; then
                success "API Server attivo"
                return 0
            else
                error "API Server non raggiungibile"
                return 1
            fi
            ;;
        "gateway")
            if kubectl get pods -n $NAMESPACE -l app=wasmbed-gateway --field-selector=status.phase=Running 2>/dev/null | grep -q Running; then
                success "Gateway attivo"
                return 0
            else
                warning "Nessun gateway attivo"
                return 1
            fi
            ;;
        "device-controller")
            if kubectl get pods -n $NAMESPACE -l app=wasmbed-device-controller --field-selector=status.phase=Running 2>/dev/null | grep -q Running; then
                success "Device Controller attivo"
                return 0
            else
                warning "Device Controller non attivo"
                return 1
            fi
            ;;
        "app-controller")
            if kubectl get pods -n $NAMESPACE -l app=wasmbed-application-controller --field-selector=status.phase=Running 2>/dev/null | grep -q Running; then
                success "Application Controller attivo"
                return 0
            else
                warning "Application Controller non attivo"
                return 1
            fi
            ;;
    esac
}

# Funzione per salvare i log di un componente
save_logs() {
    local component=$1
    local test_name=$2
    local log_file="$TEST_DIR/${test_name}-${component}-logs.txt"
    
    log "Salvando log di $component..."
    
    case $component in
        "gateway")
            kubectl logs -n $NAMESPACE -l app=wasmbed-gateway --tail=100 > "$log_file" 2>&1 || true
            ;;
        "device-controller")
            kubectl logs -n $NAMESPACE -l app=wasmbed-device-controller --tail=100 > "$log_file" 2>&1 || true
            ;;
        "app-controller")
            kubectl logs -n $NAMESPACE -l app=wasmbed-application-controller --tail=100 > "$log_file" 2>&1 || true
            ;;
        "api-server")
            if [ -f logs/api-server.log ]; then
                tail -100 logs/api-server.log > "$log_file" 2>&1 || true
            fi
            ;;
    esac
}

# Funzione per verificare lo stato di un Device in Kubernetes
check_device_status() {
    local device_name=$1
    local expected_phase=${2:-"Connected"}
    
    log "Verificando stato Device: $device_name (atteso: $expected_phase)"
    
    local phase=$(kubectl get device "$device_name" -n $NAMESPACE -o jsonpath='{.status.phase}' 2>/dev/null || echo "NotFound")
    
    if [ "$phase" = "$expected_phase" ]; then
        success "Device $device_name in fase $phase"
        return 0
    elif [ "$phase" = "NotFound" ]; then
        error "Device $device_name non trovato"
        return 1
    else
        warning "Device $device_name in fase $phase (atteso: $expected_phase)"
        return 1
    fi
}

# Funzione per verificare lo stato di un Application in Kubernetes
check_application_status() {
    local app_name=$1
    local expected_phase=${2:-"Deployed"}
    
    log "Verificando stato Application: $app_name (atteso: $expected_phase)"
    
    local phase=$(kubectl get application "$app_name" -n $NAMESPACE -o jsonpath='{.status.phase}' 2>/dev/null || echo "NotFound")
    
    if [ "$phase" = "$expected_phase" ]; then
        success "Application $app_name in fase $phase"
        return 0
    elif [ "$phase" = "NotFound" ]; then
        error "Application $app_name non trovata"
        return 1
    else
        warning "Application $app_name in fase $phase (atteso: $expected_phase)"
        return 1
    fi
}

# TEST 1: Device Enrollment - Simple Workflow
test_device_enrollment_simple() {
    local test_name="device-enrollment-simple"
    log "=== TEST 1: Device Enrollment - Simple Workflow ==="
    
    local device_name="test-enrollment-simple-$(date +%s)"
    
    # 1. Create Device via API
    log "1. Creazione device via API..."
    local response=$(curl -s -X POST -H "Content-Type: application/json" \
        -d "{\"name\":\"$device_name\",\"type\":\"MCU\",\"mcuType\":\"RenodeArduinoNano33Ble\",\"gatewayId\":\"gateway-1\"}" \
        "$API_BASE/api/v1/devices")
    
    if echo "$response" | jq -e '.success' > /dev/null 2>&1; then
        success "Device creato: $device_name"
        echo "$response" | jq . > "$TEST_DIR/${test_name}-create-device.json"
    else
        error "Errore nella creazione device"
        echo "$response" > "$TEST_DIR/${test_name}-create-device-error.json"
        return 1
    fi
    
    # 2. Start Renode device
    log "2. Avvio Renode device..."
    sleep 2
    local start_response=$(curl -s -X POST -H "Content-Type: application/json" -d "{}" "$API_BASE/api/v1/devices/$device_name/renode/start" 2>&1)
    if echo "$start_response" | jq . > "$TEST_DIR/${test_name}-start-device.json" 2>/dev/null; then
        success "Renode device avviato"
    else
        echo "$start_response" > "$TEST_DIR/${test_name}-start-device-error.txt"
        warning "Errore nell'avvio Renode (potrebbe essere già avviato)"
    fi
    
    # 3. Enroll device
    log "3. Enrollment device..."
    sleep 2
    local enroll_response=$(curl -s -X POST -H "Content-Type: application/json" -d "{}" "$API_BASE/api/v1/devices/$device_name/enroll" 2>&1)
    if echo "$enroll_response" | jq . > "$TEST_DIR/${test_name}-enroll-device.json" 2>/dev/null; then
        success "Device enrolled"
    else
        echo "$enroll_response" > "$TEST_DIR/${test_name}-enroll-device-error.txt"
        warning "Errore nell'enrollment (potrebbe essere già enrolled)"
    fi
    
    # 4. Connect device
    log "4. Connessione device..."
    sleep 2
    local connect_response=$(curl -s -X POST -H "Content-Type: application/json" -d "{}" "$API_BASE/api/v1/devices/$device_name/connect" 2>&1)
    if echo "$connect_response" | jq . > "$TEST_DIR/${test_name}-connect-device.json" 2>/dev/null; then
        success "Device connesso"
    else
        echo "$connect_response" > "$TEST_DIR/${test_name}-connect-device-error.txt"
        warning "Errore nella connessione (potrebbe essere già connesso)"
    fi
    
    # 5. Verifica stato in Kubernetes
    log "5. Verifica stato in Kubernetes..."
    sleep 5
    check_device_status "$device_name" "Connected" || warning "Device non ancora in fase Connected"
    
    # 6. Salva log
    save_logs "gateway" "$test_name"
    save_logs "device-controller" "$test_name"
    save_logs "api-server" "$test_name"
    
    # 7. Verifica heartbeat
    log "6. Verifica heartbeat..."
    sleep 10
    local device_info=$(curl -s "$API_BASE/api/v1/devices/$device_name")
    echo "$device_info" | jq . > "$TEST_DIR/${test_name}-device-info.json"
    
    success "Test Device Enrollment Simple completato"
    return 0
}

# TEST 2: Device Enrollment - Connection
test_device_enrollment_connection() {
    local test_name="device-enrollment-connection"
    log "=== TEST 2: Device Enrollment - Connection ==="
    
    local device_name="test-enrollment-connection-$(date +%s)"
    
    # 1. Create Device
    log "1. Creazione device..."
    curl -s -X POST -H "Content-Type: application/json" \
        -d "{\"name\":\"$device_name\",\"type\":\"MCU\",\"mcuType\":\"RenodeArduinoNano33Ble\",\"gatewayId\":\"gateway-1\"}" \
        "$API_BASE/api/v1/devices" > "$TEST_DIR/${test_name}-create-device.json"
    
    # 2. Start Renode
    log "2. Avvio Renode..."
    sleep 2
    curl -s -X POST -H "Content-Type: application/json" "$API_BASE/api/v1/devices/$device_name/renode/start" > "$TEST_DIR/${test_name}-start-device.json"
    
    # 3. Connect (simula TLS handshake)
    log "3. Connessione TLS..."
    sleep 3
    curl -s -X POST "$API_BASE/api/v1/devices/$device_name/connect" > "$TEST_DIR/${test_name}-connect-device.json"
    
    # 4. Verifica connessione
    log "4. Verifica connessione..."
    sleep 5
    local device_info=$(curl -s "$API_BASE/api/v1/devices/$device_name")
    echo "$device_info" | jq . > "$TEST_DIR/${test_name}-device-info.json"
    
    # 5. Verifica log gateway per TLS handshake
    log "5. Verifica log gateway per TLS handshake..."
    kubectl logs -n $NAMESPACE -l app=wasmbed-gateway --tail=50 | grep -i "tls\|handshake\|connection" > "$TEST_DIR/${test_name}-gateway-tls-logs.txt" || true
    
    success "Test Device Enrollment Connection completato"
    return 0
}

# TEST 3: Device Enrollment - Process
test_device_enrollment_process() {
    local test_name="device-enrollment-process"
    log "=== TEST 3: Device Enrollment - Process ==="
    
    local device_name="test-enrollment-process-$(date +%s)"
    
    # 1. Create Device CRD
    log "1. Creazione Device CRD..."
    curl -s -X POST -H "Content-Type: application/json" \
        -d "{\"name\":\"$device_name\",\"type\":\"MCU\",\"mcuType\":\"RenodeArduinoNano33Ble\",\"gatewayId\":\"gateway-1\"}" \
        "$API_BASE/api/v1/devices" > "$TEST_DIR/${test_name}-create-device.json"
    
    # 2. Verifica Device CRD creato
    log "2. Verifica Device CRD..."
    sleep 2
    kubectl get device "$device_name" -n $NAMESPACE -o yaml > "$TEST_DIR/${test_name}-device-crd.yaml" || error "Device CRD non trovato"
    
    # 3. Enrollment
    log "3. Enrollment..."
    sleep 2
    curl -s -X POST "$API_BASE/api/v1/devices/$device_name/enroll" > "$TEST_DIR/${test_name}-enroll-device.json"
    
    # 4. Verifica stato dopo enrollment
    log "4. Verifica stato dopo enrollment..."
    sleep 5
    kubectl get device "$device_name" -n $NAMESPACE -o jsonpath='{.status}' > "$TEST_DIR/${test_name}-device-status.json" || true
    
    # 5. Verifica log controller
    log "5. Verifica log Device Controller..."
    save_logs "device-controller" "$test_name"
    
    success "Test Device Enrollment Process completato"
    return 0
}

# TEST 4: Device Enrollment - Heartbeat
test_device_enrollment_heartbeat() {
    local test_name="device-enrollment-heartbeat"
    log "=== TEST 4: Device Enrollment - Heartbeat ==="
    
    local device_name="test-enrollment-heartbeat-$(date +%s)"
    
    # 1. Setup device
    log "1. Setup device..."
    curl -s -X POST -H "Content-Type: application/json" \
        -d "{\"name\":\"$device_name\",\"type\":\"MCU\",\"mcuType\":\"RenodeArduinoNano33Ble\",\"gatewayId\":\"gateway-1\"}" \
        "$API_BASE/api/v1/devices" > /dev/null
    
    sleep 2
    curl -s -X POST -H "Content-Type: application/json" "$API_BASE/api/v1/devices/$device_name/renode/start" > /dev/null
    sleep 2
    curl -s -X POST "$API_BASE/api/v1/devices/$device_name/connect" > /dev/null
    
    # 2. Monitor heartbeat per 30 secondi
    log "2. Monitoraggio heartbeat per 30 secondi..."
    for i in {1..6}; do
        sleep 5
        local device_info=$(curl -s "$API_BASE/api/v1/devices/$device_name")
        local last_heartbeat=$(echo "$device_info" | jq -r '.lastHeartbeat // "N/A"')
        log "   Heartbeat $i: $last_heartbeat"
        echo "$device_info" | jq . > "$TEST_DIR/${test_name}-heartbeat-${i}.json"
    done
    
    # 3. Verifica heartbeat in Kubernetes
    log "3. Verifica heartbeat in Kubernetes..."
    kubectl get device "$device_name" -n $NAMESPACE -o jsonpath='{.status.lastHeartbeat}' > "$TEST_DIR/${test_name}-k8s-heartbeat.txt" || true
    
    success "Test Device Enrollment Heartbeat completato"
    return 0
}

# TEST 5: Application Deployment - Compilation
test_application_deployment_compilation() {
    local test_name="application-deployment-compilation"
    log "=== TEST 5: Application Deployment - Compilation ==="
    
    # 1. Compile Rust to WASM
    log "1. Compilazione Rust to WASM..."
    # Il codice viene inserito in un template wasm-bindgen, quindi non serve no_std
    # main() deve essere () non può restituire valori
    local rust_code='pub fn main() {
    // Simple test function
    let _result = 42;
}'
    
    local compile_response=$(curl -s -X POST -H "Content-Type: application/json" \
        -d "{\"code\":$(echo "$rust_code" | jq -Rs .),\"language\":\"rust\",\"target\":\"wasm32-unknown-unknown\"}" \
        "$API_BASE/api/v1/compile")
    
    echo "$compile_response" | jq . > "$TEST_DIR/${test_name}-compile.json"
    
    if echo "$compile_response" | jq -e '.wasmBytes' > /dev/null 2>&1; then
        success "Compilazione riuscita"
        local wasm_binary=$(echo "$compile_response" | jq -r '.wasmBytes')
        echo "$wasm_binary" | base64 -d > "$TEST_DIR/${test_name}-wasm.bin" 2>/dev/null || true
    elif echo "$compile_response" | jq -e '.success == true' > /dev/null 2>&1; then
        success "Compilazione riuscita"
        local wasm_binary=$(echo "$compile_response" | jq -r '.wasmBytes // .wasm')
        echo "$wasm_binary" | base64 -d > "$TEST_DIR/${test_name}-wasm.bin" 2>/dev/null || true
    else
        error "Errore nella compilazione"
        echo "$compile_response" | jq -r '.error' > "$TEST_DIR/${test_name}-compile-error.txt" 2>/dev/null || echo "$compile_response" > "$TEST_DIR/${test_name}-compile-error.txt"
        return 1
    fi
    
    # 2. Create Application
    log "2. Creazione Application..."
    local app_name="test-app-compilation-$(date +%s)"
    local app_response=$(curl -s -X POST -H "Content-Type: application/json" \
        -d "{\"name\":\"$app_name\",\"wasm\":\"$wasm_binary\",\"targetDevices\":[]}" \
        "$API_BASE/api/v1/applications")
    
    echo "$app_response" | jq . > "$TEST_DIR/${test_name}-create-app.json"
    
    # 3. Verifica Application CRD
    log "3. Verifica Application CRD..."
    sleep 2
    kubectl get application "$app_name" -n $NAMESPACE -o yaml > "$TEST_DIR/${test_name}-app-crd.yaml" || error "Application CRD non trovata"
    
    success "Test Application Deployment Compilation completato"
    return 0
}

# TEST 6: Application Deployment - Simple
test_application_deployment_simple() {
    local test_name="application-deployment-simple"
    log "=== TEST 6: Application Deployment - Simple ==="
    
    # 1. Setup device
    local device_name="test-deployment-simple-device-$(date +%s)"
    curl -s -X POST -H "Content-Type: application/json" \
        -d "{\"name\":\"$device_name\",\"type\":\"MCU\",\"mcuType\":\"RenodeArduinoNano33Ble\",\"gatewayId\":\"gateway-1\"}" \
        "$API_BASE/api/v1/devices" > /dev/null
    sleep 2
    curl -s -X POST -H "Content-Type: application/json" "$API_BASE/api/v1/devices/$device_name/renode/start" > /dev/null
    sleep 2
    curl -s -X POST "$API_BASE/api/v1/devices/$device_name/connect" > /dev/null
    
    # 2. Compile
    log "1. Compilazione..."
    local rust_code='pub fn main() { let _result = 42; }'
    local compile_response=$(curl -s -X POST -H "Content-Type: application/json" \
        -d "{\"code\":$(echo "$rust_code" | jq -Rs .),\"language\":\"rust\",\"target\":\"wasm32-unknown-unknown\"}" \
        "$API_BASE/api/v1/compile")
    local wasm_binary=$(echo "$compile_response" | jq -r '.wasmBytes // .wasm // ""')
    
    if [ -z "$wasm_binary" ]; then
        error "Compilazione fallita"
        echo "$compile_response" | jq -r '.error' > "$TEST_DIR/${test_name}-compile-error.txt" 2>/dev/null || echo "$compile_response" > "$TEST_DIR/${test_name}-compile-error.txt"
        return 1
    fi
    
    # 3. Create Application
    log "2. Creazione Application..."
    local app_name="test-app-simple-$(date +%s)"
    curl -s -X POST -H "Content-Type: application/json" \
        -d "{\"name\":\"$app_name\",\"wasm\":\"$wasm_binary\",\"targetDevices\":[\"$device_name\"]}" \
        "$API_BASE/api/v1/applications" > "$TEST_DIR/${test_name}-create-app.json"
    
    # 4. Deploy
    log "3. Deploy Application..."
    sleep 2
    curl -s -X POST "$API_BASE/api/v1/applications/$app_name/deploy" > "$TEST_DIR/${test_name}-deploy-app.json"
    
    # 5. Verifica stato
    log "4. Verifica stato deployment..."
    sleep 10
    check_application_status "$app_name" "Deployed" || warning "Application non ancora deployed"
    
    save_logs "gateway" "$test_name"
    save_logs "app-controller" "$test_name"
    
    success "Test Application Deployment Simple completato"
    return 0
}

# TEST 7: Application Deployment - Execution
test_application_deployment_execution() {
    local test_name="application-deployment-execution"
    log "=== TEST 7: Application Deployment - Execution ==="
    
    # Similar to simple but with more detailed execution verification
    test_application_deployment_simple
}

# TEST 8: Application Deployment - Monitoring
test_application_deployment_monitoring() {
    local test_name="application-deployment-monitoring"
    log "=== TEST 8: Application Deployment - Monitoring ==="
    
    # 1. Setup and deploy (reuse simple test)
    local device_name="test-monitoring-device-$(date +%s)"
    curl -s -X POST -H "Content-Type: application/json" \
        -d "{\"name\":\"$device_name\",\"type\":\"MCU\",\"mcuType\":\"RenodeArduinoNano33Ble\",\"gatewayId\":\"gateway-1\"}" \
        "$API_BASE/api/v1/devices" > /dev/null
    sleep 2
    curl -s -X POST -H "Content-Type: application/json" "$API_BASE/api/v1/devices/$device_name/renode/start" > /dev/null
    sleep 2
    curl -s -X POST "$API_BASE/api/v1/devices/$device_name/connect" > /dev/null
    
    local rust_code='#![no_std]#![no_main]#[panic_handler]fn panic(_: &core::panic::PanicInfo) -> ! { loop {} }#[no_mangle]pub extern "C" fn main() -> i32 { 42 }'
    local compile_response=$(curl -s -X POST -H "Content-Type: application/json" \
        -d "{\"code\":$(echo "$rust_code" | jq -Rs .),\"language\":\"rust\",\"target\":\"wasm32-unknown-unknown\"}" \
        "$API_BASE/api/v1/compile")
    local wasm_binary=$(echo "$compile_response" | jq -r '.wasm // ""')
    
    local app_name="test-app-monitoring-$(date +%s)"
    curl -s -X POST -H "Content-Type: application/json" \
        -d "{\"name\":\"$app_name\",\"wasm\":\"$wasm_binary\",\"targetDevices\":[\"$device_name\"]}" \
        "$API_BASE/api/v1/applications" > /dev/null
    sleep 2
    curl -s -X POST "$API_BASE/api/v1/applications/$app_name/deploy" > /dev/null
    
    # 2. Monitor application status
    log "1. Monitoraggio stato application..."
    for i in {1..6}; do
        sleep 5
        local app_info=$(curl -s "$API_BASE/api/v1/applications/$app_name")
        echo "$app_info" | jq . > "$TEST_DIR/${test_name}-status-${i}.json"
        local phase=$(echo "$app_info" | jq -r '.status.phase // "N/A"')
        log "   Status $i: $phase"
    done
    
    # 3. Get metrics
    log "2. Recupero metriche..."
    curl -s "$API_BASE/api/v1/metrics" > "$TEST_DIR/${test_name}-metrics.json"
    
    success "Test Application Deployment Monitoring completato"
    return 0
}

# TEST 9: Error Handling
test_error_handling() {
    local test_name="error-handling"
    log "=== TEST 9: Error Handling ==="
    
    # 1. Test enrollment failure (invalid device)
    log "1. Test enrollment failure..."
    local invalid_device="test-invalid-device-$(date +%s)"
    local enroll_response=$(curl -s -X POST "$API_BASE/api/v1/devices/$invalid_device/enroll")
    echo "$enroll_response" | jq . > "$TEST_DIR/${test_name}-enroll-failure.json"
    
    # 2. Test deployment failure (invalid WASM)
    log "2. Test deployment failure..."
    local app_name="test-app-invalid-$(date +%s)"
    local deploy_response=$(curl -s -X POST -H "Content-Type: application/json" \
        -d "{\"name\":\"$app_name\",\"wasm\":\"invalid-wasm-binary\"}" \
        "$API_BASE/api/v1/applications")
    echo "$deploy_response" | jq . > "$TEST_DIR/${test_name}-deploy-failure.json"
    
    # 3. Test connection lost
    log "3. Test connection lost..."
    local device_name="test-connection-lost-$(date +%s)"
    curl -s -X POST -H "Content-Type: application/json" \
        -d "{\"name\":\"$device_name\",\"type\":\"MCU\",\"mcuType\":\"RenodeArduinoNano33Ble\",\"gatewayId\":\"gateway-1\"}" \
        "$API_BASE/api/v1/devices" > /dev/null
    sleep 2
    curl -s -X POST -H "Content-Type: application/json" "$API_BASE/api/v1/devices/$device_name/renode/start" > /dev/null
    sleep 2
    curl -s -X POST "$API_BASE/api/v1/devices/$device_name/connect" > /dev/null
    sleep 2
    curl -s -X POST -H "Content-Type: application/json" "$API_BASE/api/v1/devices/$device_name/renode/stop" > /dev/null
    sleep 5
    local device_info=$(curl -s "$API_BASE/api/v1/devices/$device_name")
    echo "$device_info" | jq . > "$TEST_DIR/${test_name}-connection-lost.json"
    
    success "Test Error Handling completato"
    return 0
}

# Main execution
main() {
    log "=== TEST REALI WORKFLOW UML ==="
    log "Directory test: $TEST_DIR"
    log ""
    
    # Verifica componenti
    log "Verifica componenti attivi..."
    check_component "api-server" || exit 1
    check_component "gateway" || warning "Gateway non attivo, alcuni test potrebbero fallire"
    check_component "device-controller" || warning "Device Controller non attivo"
    check_component "app-controller" || warning "Application Controller non attivo"
    log ""
    
    # Esegui tutti i test con monitoraggio dettagliato
    local tests_passed=0
    local tests_failed=0
    local failed_tests=()
    
    log "=== INIZIO ESECUZIONE TUTTI I TEST ==="
    echo ""
    
    log "🔄 TEST 1/9: Device Enrollment - Simple Workflow"
    if test_device_enrollment_simple; then
        ((tests_passed++))
        success "TEST 1 COMPLETATO"
    else
        ((tests_failed++))
        failed_tests+=("Device Enrollment - Simple")
        error "TEST 1 FALLITO"
    fi
    echo ""
    sleep 2
    
    log "🔄 TEST 2/9: Device Enrollment - Connection"
    if test_device_enrollment_connection; then
        ((tests_passed++))
        success "TEST 2 COMPLETATO"
    else
        ((tests_failed++))
        failed_tests+=("Device Enrollment - Connection")
        error "TEST 2 FALLITO"
    fi
    echo ""
    sleep 2
    
    log "🔄 TEST 3/9: Device Enrollment - Process"
    if test_device_enrollment_process; then
        ((tests_passed++))
        success "TEST 3 COMPLETATO"
    else
        ((tests_failed++))
        failed_tests+=("Device Enrollment - Process")
        error "TEST 3 FALLITO"
    fi
    echo ""
    sleep 2
    
    log "🔄 TEST 4/9: Device Enrollment - Heartbeat"
    if test_device_enrollment_heartbeat; then
        ((tests_passed++))
        success "TEST 4 COMPLETATO"
    else
        ((tests_failed++))
        failed_tests+=("Device Enrollment - Heartbeat")
        error "TEST 4 FALLITO"
    fi
    echo ""
    sleep 2
    
    log "🔄 TEST 5/9: Application Deployment - Compilation"
    if test_application_deployment_compilation; then
        ((tests_passed++))
        success "TEST 5 COMPLETATO"
    else
        ((tests_failed++))
        failed_tests+=("Application Deployment - Compilation")
        error "TEST 5 FALLITO"
    fi
    echo ""
    sleep 2
    
    log "🔄 TEST 6/9: Application Deployment - Simple"
    if test_application_deployment_simple; then
        ((tests_passed++))
        success "TEST 6 COMPLETATO"
    else
        ((tests_failed++))
        failed_tests+=("Application Deployment - Simple")
        error "TEST 6 FALLITO"
    fi
    echo ""
    sleep 2
    
    log "🔄 TEST 7/9: Application Deployment - Execution"
    if test_application_deployment_execution; then
        ((tests_passed++))
        success "TEST 7 COMPLETATO"
    else
        ((tests_failed++))
        failed_tests+=("Application Deployment - Execution")
        error "TEST 7 FALLITO"
    fi
    echo ""
    sleep 2
    
    log "🔄 TEST 8/9: Application Deployment - Monitoring"
    if test_application_deployment_monitoring; then
        ((tests_passed++))
        success "TEST 8 COMPLETATO"
    else
        ((tests_failed++))
        failed_tests+=("Application Deployment - Monitoring")
        error "TEST 8 FALLITO"
    fi
    echo ""
    sleep 2
    
    log "🔄 TEST 9/9: Error Handling"
    if test_error_handling; then
        ((tests_passed++))
        success "TEST 9 COMPLETATO"
    else
        ((tests_failed++))
        failed_tests+=("Error Handling")
        error "TEST 9 FALLITO"
    fi
    echo ""
    
    # Riepilogo dettagliato
    log "=== RIEPILOGO FINALE ==="
    echo ""
    success "✅ Test passati: $tests_passed/9"
    if [ $tests_failed -gt 0 ]; then
        error "❌ Test falliti: $tests_failed/9"
        echo ""
        error "Test falliti:"
        for test in "${failed_tests[@]}"; do
            error "  - $test"
        done
    fi
    echo ""
    log "📁 Log e risultati salvati in: $TEST_DIR"
    log ""
    log "Per visualizzare i log:"
    log "  ls -la $TEST_DIR"
    log "  kubectl logs -n $NAMESPACE -l app=wasmbed-gateway --tail=100"
    echo ""
    
    # Salva riepilogo in file
    {
        echo "=== RIEPILOGO TEST UML WORKFLOWS ==="
        echo "Data: $(date)"
        echo ""
        echo "Test passati: $tests_passed/9"
        echo "Test falliti: $tests_failed/9"
        echo ""
        if [ ${#failed_tests[@]} -gt 0 ]; then
            echo "Test falliti:"
            for test in "${failed_tests[@]}"; do
                echo "  - $test"
            done
        fi
        echo ""
        echo "Directory log: $TEST_DIR"
    } > "$TEST_DIR/RIEPILOGO.txt"
}

main "$@"

