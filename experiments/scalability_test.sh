#!/bin/bash
# RETROSPECT Platform Scalability Test
# Tests device creation, application deployment, and concurrent operations

API_BASE="http://localhost:3001/api/v1"
RESULTS_FILE="scalability_results_$(date +%Y%m%d_%H%M%S).json"

echo "{"
echo "  \"test_start\": \"$(date -Iseconds)\","
echo "  \"baseline\": {"

# Baseline metrics
echo "    \"devices\": $(curl -sS ${API_BASE}/devices 2>/dev/null | jq '.devices | length'),"
echo "    \"applications\": $(curl -sS ${API_BASE}/applications 2>/dev/null | jq '.applications | length'),"
echo "    \"gateways\": $(curl -sS ${API_BASE}/gateways 2>/dev/null | jq '.gateways | length')"
echo "  },"

echo "  \"tests\": {"

# Test 1: Sequential device creation (10 devices)
echo "    \"sequential_device_creation\": {"
echo "      \"count\": 10,"
echo "      \"results\": ["

START_TIME=$(date +%s%3N)
for i in {1..10}; do
  DEVICE_NAME="scale-test-device-seq-$i"
  REQ_START=$(date +%s%3N)
  RESPONSE=$(curl -sS -X POST ${API_BASE}/devices \
    -H "Content-Type: application/json" \
    -d "{\"name\":\"$DEVICE_NAME\",\"mcuType\":\"RenodeArduinoNano33Ble\"}" 2>/dev/null)
  REQ_END=$(date +%s%3N)
  LATENCY=$((REQ_END - REQ_START))
  
  STATUS=$(echo "$RESPONSE" | jq -r '.id // "error"')
  
  if [ "$i" -lt 10 ]; then
    echo "        {\"device\":\"$DEVICE_NAME\", \"latency_ms\":$LATENCY, \"status\":\"$STATUS\"},"
  else
    echo "        {\"device\":\"$DEVICE_NAME\", \"latency_ms\":$LATENCY, \"status\":\"$STATUS\"}"
  fi
  sleep 0.5
done
END_TIME=$(date +%s%3N)
TOTAL_TIME=$((END_TIME - START_TIME))

echo "      ],"
echo "      \"total_time_ms\": $TOTAL_TIME"
echo "    },"

# Test 2: Parallel device creation (10 devices)
echo "    \"parallel_device_creation\": {"
echo "      \"count\": 10,"
echo "      \"results\": ["

PIDS=()
TEMP_DIR=$(mktemp -d)
START_TIME=$(date +%s%3N)

for i in {1..10}; do
  DEVICE_NAME="scale-test-device-par-$i"
  (
    REQ_START=$(date +%s%3N)
    RESPONSE=$(curl -sS -X POST ${API_BASE}/devices \
      -H "Content-Type: application/json" \
      -d "{\"name\":\"$DEVICE_NAME\",\"mcuType\":\"RenodeArduinoNano33Ble\"}" 2>/dev/null)
    REQ_END=$(date +%s%3N)
    LATENCY=$((REQ_END - REQ_START))
    STATUS=$(echo "$RESPONSE" | jq -r '.id // "error"')
    echo "{\"device\":\"$DEVICE_NAME\", \"latency_ms\":$LATENCY, \"status\":\"$STATUS\"}" > "$TEMP_DIR/result_$i.json"
  ) &
  PIDS+=($!)
done

# Wait for all parallel requests
for pid in "${PIDS[@]}"; do
  wait $pid
done
END_TIME=$(date +%s%3N)
TOTAL_TIME=$((END_TIME - START_TIME))

# Collect results
for i in {1..10}; do
  if [ "$i" -lt 10 ]; then
    cat "$TEMP_DIR/result_$i.json" | tr -d '\n'
    echo ","
  else
    cat "$TEMP_DIR/result_$i.json"
  fi
done

rm -rf "$TEMP_DIR"

echo "      ],"
echo "      \"total_time_ms\": $TOTAL_TIME"
echo "    },"

# Test 3: Application deployment stress (5 applications)
echo "    \"application_deployment\": {"
echo "      \"count\": 5,"
echo "      \"results\": ["

START_TIME=$(date +%s%3N)
for i in {1..5}; do
  APP_NAME="scale-test-app-$i"
  REQ_START=$(date +%s%3N)
  RESPONSE=$(curl -sS -X POST ${API_BASE}/applications \
    -H "Content-Type: application/json" \
    -d "{\"name\":\"$APP_NAME\",\"targetDevices\":[\"scale-test-device-seq-$i\"],\"wasmBytes\":\"AGFzbQEAAAA=\"}" 2>/dev/null)
  REQ_END=$(date +%s%3N)
  LATENCY=$((REQ_END - REQ_START))
  
  STATUS=$(echo "$RESPONSE" | jq -r '.name // "error"')
  
  if [ "$i" -lt 5 ]; then
    echo "        {\"app\":\"$APP_NAME\", \"latency_ms\":$LATENCY, \"status\":\"$STATUS\"},"
  else
    echo "        {\"app\":\"$APP_NAME\", \"latency_ms\":$LATENCY, \"status\":\"$STATUS\"}"
  fi
  sleep 0.5
done
END_TIME=$(date +%s%3N)
TOTAL_TIME=$((END_TIME - START_TIME))

echo "      ],"
echo "      \"total_time_ms\": $TOTAL_TIME"
echo "    },"

# Test 4: Read operations under load (50 list requests)
echo "    \"read_operations_load\": {"
echo "      \"device_list_requests\": 50,"
echo "      \"results\": ["

START_TIME=$(date +%s%3N)
MIN_LATENCY=999999
MAX_LATENCY=0
SUM_LATENCY=0

for i in {1..50}; do
  REQ_START=$(date +%s%3N)
  curl -sS ${API_BASE}/devices >/dev/null 2>&1
  REQ_END=$(date +%s%3N)
  LATENCY=$((REQ_END - REQ_START))
  
  SUM_LATENCY=$((SUM_LATENCY + LATENCY))
  [ $LATENCY -lt $MIN_LATENCY ] && MIN_LATENCY=$LATENCY
  [ $LATENCY -gt $MAX_LATENCY ] && MAX_LATENCY=$LATENCY
  
  if [ "$i" -eq 1 ] || [ "$i" -eq 25 ] || [ "$i" -eq 50 ]; then
    if [ "$i" -eq 50 ]; then
      echo "        {\"request\":$i, \"latency_ms\":$LATENCY}"
    else
      echo "        {\"request\":$i, \"latency_ms\":$LATENCY},"
    fi
  fi
done
END_TIME=$(date +%s%3N)
TOTAL_TIME=$((END_TIME - START_TIME))
AVG_LATENCY=$((SUM_LATENCY / 50))

echo "      ],"
echo "      \"total_time_ms\": $TOTAL_TIME,"
echo "      \"min_latency_ms\": $MIN_LATENCY,"
echo "      \"max_latency_ms\": $MAX_LATENCY,"
echo "      \"avg_latency_ms\": $AVG_LATENCY"
echo "    }"

echo "  },"

# Final system state
echo "  \"final_state\": {"
echo "    \"devices\": $(curl -sS ${API_BASE}/devices 2>/dev/null | jq '.devices | length'),"
echo "    \"applications\": $(curl -sS ${API_BASE}/applications 2>/dev/null | jq '.applications | length'),"
echo "    \"gateways\": $(curl -sS ${API_BASE}/gateways 2>/dev/null | jq '.gateways | length')"
echo "  },"

echo "  \"test_end\": \"$(date -Iseconds)\""
echo "}"
