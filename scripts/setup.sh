#!/bin/bash
# Convenience script for quick setup
set -euo pipefail

echo "Wasmbed - Quick Setup"
echo "====================="
echo "1. Platform setup..."
./scripts/setup/01-setup-platform.sh

echo "2. Build images..."
./scripts/setup/03-build-images.sh

echo "3. Deploy to Kubernetes..."
./scripts/setup/02-deploy-to-k8s.sh

echo "Setup completed!"