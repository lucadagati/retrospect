#!/bin/bash
# Enhanced Scalability Test with Metrics Collection

API_BASE="http://localhost:3001/api/v1"
OUTPUT_FILE="scalability_metrics_$(date +%Y%m%d_%H%M%S).json"

echo "Starting RETROSPECT Scalability Test..."
echo ""

# Utility functions
get_timestamp_ms() {
  echo $(($(date +%s%N)/1000000))
}

# Collect system metrics
collect_system_metrics() {
  local label=$1
  echo "  \"$label\": {" >> $OUTPUT_FILE
  echo "    \"timestamp\": \"$(date -Iseconds)\"," >> $OUTPUT_FILE
  echo "    \"devices\": $(curl -sS ${API_BASE}/devices 2>/dev/null | jq '.devices | length')," >> $OUTPUT_FILE
  echo "    \"applications\": $(curl -sS ${API_BASE}/applications 2>/dev/null | jq '.applications | length')," >> $OUTPUT_FILE
  echo "    \"gateways\": $(curl -sS ${API_BASE}/gateways 2>/dev/null | jq '.gateways | length')," >> $OUTPUT_FILE
  
  # Device status breakdown
  local status_data=$(curl -sS ${API_BASE}/devices 2>/dev/null | jq -c '[.devices | group_by(.status) | .[] | {status: .[0].status, count: length}]')
  echo "    \"device_status_breakdown\": $status_data," >> $OUTPUT_FILE
  
  # Process count
  local renode_processes=$(ps aux | grep -E 'renode' | grep -v grep | wc -l)
  echo "    \"renode_processes\": $renode_processes," >> $OUTPUT_FILE
  
  # Memory usage (API server)
  local api_mem=$(ps aux | grep 'wasmbed-api-server' | grep -v grep | awk '{print $6}' | head -1)
  echo "    \"api_server_memory_kb\": ${api_mem:-0}" >> $OUTPUT_FILE
  
  echo "  }," >> $OUTPUT_FILE
}

# Start JSON output
echo "{" > $OUTPUT_FILE
echo "  \"test_metadata\": {" >> $OUTPUT_FILE
echo "    \"start_time\": \"$(date -Iseconds)\"," >> $OUTPUT_FILE
echo "    \"platform\": \"RETROSPECT\"," >> $OUTPUT_FILE
echo "    \"test_type\": \"scalability_stress\"" >> $OUTPUT_FILE
echo "  }," >> $OUTPUT_FILE
echo "  \"metrics\": {" >> $OUTPUT_FILE

# Baseline
echo "Collecting baseline metrics..."
collect_system_metrics "baseline"

# Test 1: Burst device creation (20 devices in parallel)
echo "Test 1: Creating 20 devices in parallel..."
echo "    \"test1_parallel_burst\": {" >> $OUTPUT_FILE
echo "      \"description\": \"20 devices created simultaneously\"," >> $OUTPUT_FILE

TEMP_DIR=$(mktemp -d)
START=$(get_timestamp_ms)

for i in {1..20}; do
  (
    req_start=$(get_timestamp_ms)
    response=$(curl -sS -X POST ${API_BASE}/devices \
      -H "Content-Type: application/json" \
      -d "{\"name\":\"burst-device-$i\",\"mcuType\":\"RenodeArduinoNano33Ble\"}" 2>/dev/null)
    req_end=$(get_timestamp_ms)
    
    success=$(echo "$response" | jq -r '.success // false')
    device_id=$(echo "$response" | jq -r '.devices[0].id // "null"')
    
    echo "{\"device\":\"burst-device-$i\",\"latency_ms\":$((req_end - req_start)),\"success\":$success,\"id\":\"$device_id\"}" > "$TEMP_DIR/burst_$i.json"
  ) &
done

wait
END=$(get_timestamp_ms)

echo "      \"total_time_ms\": $((END - START))," >> $OUTPUT_FILE
echo "      \"results\": [" >> $OUTPUT_FILE

for i in {1..20}; do
  cat "$TEMP_DIR/burst_$i.json" >> $OUTPUT_FILE
  if [ $i -lt 20 ]; then echo "," >> $OUTPUT_FILE; fi
done

echo "" >> $OUTPUT_FILE
echo "      ]" >> $OUTPUT_FILE
echo "    }," >> $OUTPUT_FILE
rm -rf "$TEMP_DIR"

sleep 2
collect_system_metrics "after_test1"

# Test 2: Sustained sequential creation (30 devices)
echo "Test 2: Creating 30 devices sequentially..."
echo "    \"test2_sequential_sustained\": {" >> $OUTPUT_FILE
echo "      \"description\": \"30 devices created one by one\"," >> $OUTPUT_FILE

START=$(get_timestamp_ms)
success_count=0
latencies=()

echo "      \"results\": [" >> $OUTPUT_FILE

for i in {1..30}; do
  req_start=$(get_timestamp_ms)
  response=$(curl -sS -X POST ${API_BASE}/devices \
    -H "Content-Type: application/json" \
    -d "{\"name\":\"seq-device-$i\",\"mcuType\":\"RenodeArduinoNano33Ble\"}" 2>/dev/null)
  req_end=$(get_timestamp_ms)
  latency=$((req_end - req_start))
  
  success=$(echo "$response" | jq -r '.success // false')
  [ "$success" = "true" ] && ((success_count++))
  
  latencies+=($latency)
  
  if [ $((i % 10)) -eq 0 ] || [ $i -eq 30 ]; then
    echo "        {\"device\":\"seq-device-$i\",\"latency_ms\":$latency,\"success\":$success}" >> $OUTPUT_FILE
    if [ $i -lt 30 ]; then echo "," >> $OUTPUT_FILE; fi
  fi
  
  sleep 0.2
done

END=$(get_timestamp_ms)

echo "" >> $OUTPUT_FILE
echo "      ]," >> $OUTPUT_FILE
echo "      \"total_time_ms\": $((END - START))," >> $OUTPUT_FILE
echo "      \"success_count\": $success_count," >> $OUTPUT_FILE

# Calculate latency stats
IFS=$'\n' sorted=($(sort -n <<<"${latencies[*]}"))
min_lat=${sorted[0]}
max_lat=${sorted[-1]}
sum=0
for lat in "${latencies[@]}"; do sum=$((sum + lat)); done
avg_lat=$((sum / 30))
p50_lat=${sorted[14]}
p95_lat=${sorted[28]}

echo "      \"latency_stats\": {" >> $OUTPUT_FILE
echo "        \"min_ms\": $min_lat," >> $OUTPUT_FILE
echo "        \"max_ms\": $max_lat," >> $OUTPUT_FILE
echo "        \"avg_ms\": $avg_lat," >> $OUTPUT_FILE
echo "        \"p50_ms\": $p50_lat," >> $OUTPUT_FILE
echo "        \"p95_ms\": $p95_lat" >> $OUTPUT_FILE
echo "      }" >> $OUTPUT_FILE
echo "    }," >> $OUTPUT_FILE

sleep 2
collect_system_metrics "after_test2"

# Test 3: Read performance under load
echo "Test 3: Testing read performance (100 list operations)..."
echo "    \"test3_read_performance\": {" >> $OUTPUT_FILE
echo "      \"description\": \"100 consecutive device list requests\"," >> $OUTPUT_FILE

START=$(get_timestamp_ms)
read_latencies=()

for i in {1..100}; do
  req_start=$(get_timestamp_ms)
  curl -sS ${API_BASE}/devices >/dev/null 2>&1
  req_end=$(get_timestamp_ms)
  read_latencies+=($((req_end - req_start)))
done

END=$(get_timestamp_ms)

# Calculate read latency stats
IFS=$'\n' sorted=($(sort -n <<<"${read_latencies[*]}"))
min_read=${sorted[0]}
max_read=${sorted[-1]}
sum=0
for lat in "${read_latencies[@]}"; do sum=$((sum + lat)); done
avg_read=$((sum / 100))
p50_read=${sorted[49]}
p95_read=${sorted[94]}
p99_read=${sorted[98]}

echo "      \"total_requests\": 100," >> $OUTPUT_FILE
echo "      \"total_time_ms\": $((END - START))," >> $OUTPUT_FILE
echo "      \"throughput_req_per_sec\": $(echo "scale=2; 100000 / ($END - $START)" | bc)," >> $OUTPUT_FILE
echo "      \"latency_stats\": {" >> $OUTPUT_FILE
echo "        \"min_ms\": $min_read," >> $OUTPUT_FILE
echo "        \"max_ms\": $max_read," >> $OUTPUT_FILE
echo "        \"avg_ms\": $avg_read," >> $OUTPUT_FILE
echo "        \"p50_ms\": $p50_read," >> $OUTPUT_FILE
echo "        \"p95_ms\": $p95_read," >> $OUTPUT_FILE
echo "        \"p99_ms\": $p99_read" >> $OUTPUT_FILE
echo "      }" >> $OUTPUT_FILE
echo "    }," >> $OUTPUT_FILE

collect_system_metrics "after_test3"

# Test 4: Application deployment at scale
echo "Test 4: Deploying 10 applications..."
echo "    \"test4_application_deployment\": {" >> $OUTPUT_FILE
echo "      \"description\": \"10 applications deployed to existing devices\"," >> $OUTPUT_FILE

START=$(get_timestamp_ms)
echo "      \"results\": [" >> $OUTPUT_FILE

for i in {1..10}; do
  req_start=$(get_timestamp_ms)
  response=$(curl -sS -X POST ${API_BASE}/applications \
    -H "Content-Type: application/json" \
    -d "{\"name\":\"scale-app-$i\",\"targetDevices\":[\"seq-device-$i\"],\"wasmBytes\":\"AGFzbQEAAAABBQFgAAF/AwIBAAcHAQNhZGQAAAoJAQcAQQFBAWoLCwEAQQALBQIDBAU=\"}" 2>/dev/null)
  req_end=$(get_timestamp_ms)
  
  success=$(echo "$response" | jq -r '.success // false')
  
  echo "        {\"app\":\"scale-app-$i\",\"latency_ms\":$((req_end - req_start)),\"success\":$success}" >> $OUTPUT_FILE
  if [ $i -lt 10 ]; then echo "," >> $OUTPUT_FILE; fi
  
  sleep 0.3
done

END=$(get_timestamp_ms)

echo "" >> $OUTPUT_FILE
echo "      ]," >> $OUTPUT_FILE
echo "      \"total_time_ms\": $((END - START))" >> $OUTPUT_FILE
echo "    }," >> $OUTPUT_FILE

collect_system_metrics "final_state"

# Close JSON
sed -i '$ s/,$//' $OUTPUT_FILE
echo "  }," >> $OUTPUT_FILE
echo "  \"test_completion\": \"$(date -Iseconds)\"" >> $OUTPUT_FILE
echo "}" >> $OUTPUT_FILE

echo ""
echo "Test completed! Results saved to: $OUTPUT_FILE"
echo ""
echo "Summary:"
cat $OUTPUT_FILE | jq '{
  total_devices: .metrics.final_state.devices,
  total_apps: .metrics.final_state.applications,
  device_status: .metrics.final_state.device_status_breakdown,
  renode_processes: .metrics.final_state.renode_processes,
  test2_latency: .metrics.test2_sequential_sustained.latency_stats,
  test3_read_throughput: .metrics.test3_read_performance.throughput_req_per_sec
}'
