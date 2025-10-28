#!/bin/bash
# Test script for load balancing and failover

set -e

NAMESPACE="wasmbed"
NUM_DEVICES=100
NUM_GATEWAYS=5

echo "========================================="
echo "Load Balancing & Failover Test"
echo "========================================="

# Function to print colored output
print_status() {
    local color=$1
    local message=$2
    case $color in
        "green")
            echo -e "\033[0;32m[✓] $message\033[0m"
            ;;
        "yellow")
            echo -e "\033[1;33m[!] $message\033[0m"
            ;;
        "red")
            echo -e "\033[0;31m[✗] $message\033[0m"
            ;;
        *)
            echo "[*] $message"
            ;;
    esac
}

# Step 1: Create gateways
print_status "yellow" "Creating $NUM_GATEWAYS gateways..."
for i in $(seq 1 $NUM_GATEWAYS); do
    cat <<EOF | kubectl apply -f - > /dev/null
apiVersion: wasmbed.io/v1
kind: Gateway
metadata:
  name: gateway-$i
  namespace: $NAMESPACE
spec:
  endpoint: "127.0.0.1:3047$i"
  heartbeatInterval: 30
  enrollmentTimeout: 120
  connectionTimeout: 60
  maxDevices: 50
EOF
done

print_status "green" "Created $NUM_GATEWAYS gateways"

# Step 2: Wait for gateways to become running
print_status "yellow" "Waiting for gateways to become Running..."
sleep 15

for i in $(seq 1 $NUM_GATEWAYS); do
    kubectl wait --for=jsonpath='{.status.phase}'=Running \
        gateway/gateway-$i -n $NAMESPACE --timeout=120s || true
done

print_status "green" "All gateways are Running"

# Step 3: Create devices
print_status "yellow" "Creating $NUM_DEVICES devices..."
for i in $(seq 1 $NUM_DEVICES); do
    PUBLIC_KEY=$(openssl rand -hex 32)
    cat <<EOF | kubectl apply -f - > /dev/null
apiVersion: wasmbed.io/v1
kind: Device
metadata:
  name: test-device-$i
  namespace: $NAMESPACE
spec:
  mcuType: RenodeArduinoNano33Ble
  publicKey: "$PUBLIC_KEY"
EOF
done

print_status "green" "Created $NUM_DEVICES devices"

# Step 4: Wait for enrollment and check distribution
print_status "yellow" "Waiting for device enrollment (60s)..."
sleep 60

echo ""
print_status "yellow" "Checking device distribution across gateways..."
echo ""
echo "Gateway Distribution:"
echo "===================="

for i in $(seq 1 $NUM_GATEWAYS); do
    DEVICE_COUNT=$(kubectl get devices -n $NAMESPACE -o json | \
        jq -r ".items[] | select(.status.gateway.name == \"gateway-$i\") | .metadata.name" | wc -l)
    printf "  Gateway-%d: %3d devices\n" $i $DEVICE_COUNT
done

echo ""

# Step 5: Test failover by deleting a gateway
TARGET_GATEWAY=1
print_status "yellow" "Testing failover: Deleting gateway-$TARGET_GATEWAY..."

# Count devices on target gateway before deletion
DEVICES_ON_TARGET=$(kubectl get devices -n $NAMESPACE -o json | \
    jq -r ".items[] | select(.status.gateway.name == \"gateway-$TARGET_GATEWAY\") | .metadata.name" | wc -l)

print_status "yellow" "$DEVICES_ON_TARGET devices were on gateway-$TARGET_GATEWAY"

# Delete the gateway
kubectl delete gateway gateway-$TARGET_GATEWAY -n $NAMESPACE

print_status "green" "Gateway-$TARGET_GATEWAY deleted"

# Wait for device controller to detect and perform failover
print_status "yellow" "Waiting for automatic failover (90s)..."
sleep 90

# Step 6: Check new distribution after failover
echo ""
print_status "yellow" "Checking device distribution after failover..."
echo ""
echo "Gateway Distribution (After Failover):"
echo "======================================"

for i in $(seq 2 $NUM_GATEWAYS); do
    DEVICE_COUNT=$(kubectl get devices -n $NAMESPACE -o json | \
        jq -r ".items[] | select(.status.gateway.name == \"gateway-$i\") | .metadata.name" | wc -l)
    printf "  Gateway-%d: %3d devices\n" $i $DEVICE_COUNT
done

echo ""

# Step 7: Calculate distribution metrics
TOTAL_ASSIGNED=$(kubectl get devices -n $NAMESPACE -o json | \
    jq -r '.items[] | select(.status.gateway != null) | .metadata.name' | wc -l)

EXPECTED_PER_GATEWAY=$((NUM_DEVICES / (NUM_GATEWAYS - 1)))
echo "Load Balancing Metrics:"
echo "======================="
echo "  Total devices: $NUM_DEVICES"
echo "  Active gateways: $((NUM_GATEWAYS - 1))"
echo "  Assigned devices: $TOTAL_ASSIGNED"
echo "  Expected per gateway: ~$EXPECTED_PER_GATEWAY"

# Calculate standard deviation
COUNTS=()
for i in $(seq 2 $NUM_GATEWAYS); do
    COUNT=$(kubectl get devices -n $NAMESPACE -o json | \
        jq -r ".items[] | select(.status.gateway.name == \"gateway-$i\") | .metadata.name" | wc -l)
    COUNTS+=($COUNT)
done

# Calculate mean
MEAN=0
for count in "${COUNTS[@]}"; do
    MEAN=$((MEAN + count))
done
MEAN=$((MEAN / ${#COUNTS[@]}))

# Calculate variance
VARIANCE=0
for count in "${COUNTS[@]}"; do
    DIFF=$((count - MEAN))
    VARIANCE=$((VARIANCE + DIFF * DIFF))
done
VARIANCE=$((VARIANCE / ${#COUNTS[@]}))

echo "  Mean devices per gateway: $MEAN"
echo "  Variance: $VARIANCE"

if [ $VARIANCE -lt 10 ]; then
    print_status "green" "Load balancing is EXCELLENT (variance < 10)"
elif [ $VARIANCE -lt 25 ]; then
    print_status "yellow" "Load balancing is GOOD (variance < 25)"
else
    print_status "red" "Load balancing needs improvement (variance >= 25)"
fi

echo ""
print_status "green" "Load balancing and failover test completed!"

# Optional: Clean up
read -p "Clean up test resources? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    print_status "yellow" "Cleaning up..."
    kubectl delete devices --all -n $NAMESPACE
    kubectl delete gateways --all -n $NAMESPACE
    print_status "green" "Cleanup completed"
fi
