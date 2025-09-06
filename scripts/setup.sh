#!/bin/bash
# Script di convenienza per setup rapido
set -euo pipefail

echo "🚀 Wasmbed - Setup Rapido"
echo "========================="
echo "1. Setup piattaforma..."
./scripts/setup/01-setup-platform.sh

echo "2. Build immagini..."
./scripts/setup/03-build-images.sh

echo "3. Deploy su Kubernetes..."
./scripts/setup/02-deploy-to-k8s.sh

echo "✅ Setup completato!"
