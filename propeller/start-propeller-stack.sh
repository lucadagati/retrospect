#!/usr/bin/env bash
# start-propeller-stack.sh — avvia Magistrala + Propeller e applica il fix DNS nginx
set -e

PROPELLER_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$PROPELLER_DIR"

echo "[1/4] Stopping mosquitto snap (frees port 1883)..."
sudo snap stop mosquitto 2>/dev/null || true
sudo systemctl stop mosquitto 2>/dev/null || true

echo "[2/4] Starting Magistrala..."
docker compose -f docker/compose.yaml --env-file docker/.env up -d

echo "[3/4] Waiting for magistrala-base-net and nginx..."
until docker ps --format "{{.Names}}" | grep -q "magistrala-nginx"; do sleep 3; done
sleep 3

# Fix: nginx is started without being attached to magistrala-base-net by compose.
# Reconnect it with the correct service alias so internal DNS works.
echo "[3/4b] Fixing nginx network alias..."
docker network disconnect magistrala-base-net magistrala-nginx 2>/dev/null || true
docker network connect --alias nginx magistrala-base-net magistrala-nginx

echo "[3/4c] Waiting for Magistrala services to stabilize..."
until [ "$(docker ps --format "{{.Status}}" | grep -c Restarting 2>/dev/null)" -le 2 ]; do sleep 10; done

echo "[4/4] Starting Propeller (Manager + Proplet + Proxy)..."
docker compose -f docker/compose.propeller.yaml --env-file docker/.env up -d

echo ""
echo "Waiting for Manager health endpoint..."
until curl -sf http://localhost:7070/health > /dev/null 2>&1; do sleep 3; done
curl -s http://localhost:7070/health
echo ""
echo ""
echo "Proplets registered:"
curl -s http://localhost:7070/proplets | python3 -c "
import sys, json
d = json.load(sys.stdin)
for p in d.get('proplets', []):
    print(f\"  id={p['id']} name={p['name']} alive={p['alive']}\")
print(f\"  total: {d['total']}\")
" 2>/dev/null || curl -s http://localhost:7070/proplets

echo ""
echo "Stack ready. Manager: http://localhost:7070"
