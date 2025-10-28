#!/usr/bin/env bash
set -euo pipefail

# Usage: setup_scale.sh <devices> <devices_per_gateway>
DEVICES=${1:-50}
PER=${2:-20}
NS=wasmbed

if ! kubectl get ns "$NS" >/dev/null 2>&1; then
  echo "Namespace $NS not found" >&2
  exit 1
fi

# Calculate number of gateways
GATEWAYS=$(( (DEVICES + PER - 1) / PER ))

echo "Creating $GATEWAYS gateways for $DEVICES devices (≈$PER per gateway)"

# Create Gateways
for i in $(seq 1 "$GATEWAYS"); do
  GW_NAME="gateway-$i"
  GW_ENDPOINT="127.0.0.1:$((30470 + i))"
  cat <<EOF | kubectl apply -f -
apiVersion: wasmbed.io/v1
kind: Gateway
metadata:
  name: ${GW_NAME}
  namespace: ${NS}
spec:
  endpoint: "${GW_ENDPOINT}"
  capabilities:
    - enroll
    - heartbeat
  config:
    heartbeatInterval: "5s"
    enrollmentTimeout: "15s"
EOF

done

# Wait for gateway deployments to become ready
for i in $(seq 1 "$GATEWAYS"); do
  kubectl rollout status deploy/gateway-$i-deployment -n "$NS" --timeout=120s || true
  kubectl get pods -n "$NS" -l app=wasmbed-gateway,gateway=gateway-$i -o name

done

# Create Devices
for j in $(seq 1 "$DEVICES"); do
  DEV_NAME="scale-device-$j"
  PUBKEY=$(openssl rand -hex 16)
  cat <<EOF | kubectl apply -f -
apiVersion: wasmbed.github.io/v0
kind: Device
metadata:
  name: ${DEV_NAME}
  namespace: ${NS}
spec:
  publicKey: ${PUBKEY}
  mcuType: RenodeArduinoNano33Ble
EOF

done

echo "Submitted $DEVICES devices and $GATEWAYS gateways."
