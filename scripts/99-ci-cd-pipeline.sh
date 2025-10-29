#!/bin/bash

# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

set -e

# Wasmbed Platform - Full Deployment Script
# This script executes the complete deployment sequence

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    local status=$1
    local message=$2
    case $status in
        "SUCCESS")
            echo -e "${GREEN}✓ $message${NC}"
            ;;
        "ERROR")
            echo -e "${RED}✗ $message${NC}"
            ;;
        "WARNING")
            echo -e "${YELLOW}⚠ $message${NC}"
            ;;
        "INFO")
            echo -e "${BLUE}ℹ $message${NC}"
            ;;
    esac
}

print_header() {
    local title=$1
    echo ""
    echo "========================================"
    echo "  $title"
    echo "========================================"
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_status "ERROR" "Please run this script from the project root directory"
    exit 1
fi

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

print_header "WASMBED PLATFORM - FULL DEPLOYMENT SEQUENCE"

print_status "INFO" "Starting complete Wasmbed Platform deployment..."
print_status "INFO" "Script directory: $SCRIPT_DIR"
print_status "INFO" "Project root: $PROJECT_ROOT"

# Change to project root
cd "$PROJECT_ROOT"

# Step 0: Cleanup Environment
print_header "STEP 0: CLEANUP ENVIRONMENT"
./scripts/01-cleanup-environment.sh
print_status "SUCCESS" "Environment cleanup completed"

# Step 0.5: Fix kubectl configuration
print_header "STEP 0.5: FIX KUBECTL CONFIGURATION"
./scripts/02-fix-kubectl-config.sh
print_status "SUCCESS" "kubectl configuration fixed"

# Step 1: Build Components
print_header "STEP 1: BUILD COMPONENTS"
./scripts/03-build-components.sh
print_status "SUCCESS" "Components build completed"

# Step 2: Deploy Infrastructure
print_header "STEP 2: DEPLOY INFRASTRUCTURE"
./scripts/04-deploy-infrastructure.sh
print_status "SUCCESS" "Infrastructure deployment completed"

# Step 3: Check System Status
print_header "STEP 3: CHECK SYSTEM STATUS"
./scripts/05-check-system-status.sh
print_status "SUCCESS" "System status check completed"

# Step 4: Test ARM Cortex-M
print_header "STEP 4: TEST ARM CORTEX-M"
./scripts/09-test-arm-cortex-m.sh
print_status "SUCCESS" "ARM Cortex-M test completed"

# Step 5: Test All Workflows
print_header "STEP 5: TEST ALL WORKFLOWS"
./scripts/10-test-workflows.sh
print_status "SUCCESS" "Workflow testing completed"

# Step 6: Test Dashboard
print_header "STEP 6: TEST DASHBOARD"
./scripts/11-test-dashboard.sh
print_status "SUCCESS" "Dashboard testing completed"

# Step 7: Test QEMU Integration
print_header "STEP 7: TEST QEMU INTEGRATION"
./scripts/12-test-renode-dashboard.sh
print_status "SUCCESS" "QEMU integration testing completed"

# Step 8: Test Device Connection Workflow
print_header "STEP 8: TEST DEVICE CONNECTION WORKFLOW"
echo "Testing device creation, connection, and disconnection..."
curl -4 -X POST "http://localhost:3001/api/v1/devices" \
  -H "Content-Type: application/json" \
  -d '{"name": "test-deployment-device", "type": "MCU", "mcuType": "Mps2An385", "gatewayId": "gateway-1"}' \
  | jq '.'
curl -4 -X POST "http://localhost:3001/api/v1/devices/test-deployment-device/connect" \
  -H "Content-Type: application/json" \
  -d '{}' | jq '.'
curl -4 -X POST "http://localhost:3001/api/v1/devices/test-deployment-device/disconnect" \
  -H "Content-Type: application/json" \
  -d '{}' | jq '.'
print_status "SUCCESS" "Device connection workflow testing completed"

# Final Summary
print_header "DEPLOYMENT COMPLETE"
print_status "SUCCESS" "Wasmbed Platform deployment sequence completed successfully!"
print_status "INFO" "=== SYSTEM ENDPOINTS ==="
print_status "INFO" "Infrastructure API: http://localhost:30460"
print_status "INFO" "API Server (Backend): http://localhost:3001"
print_status "INFO" "Dashboard UI (Frontend): http://localhost:3000"
print_status "INFO" "Kubernetes API: Available via kubectl"
print_status "INFO" "=== MANAGEMENT COMMANDS ==="
print_status "INFO" "Stop all services: ./scripts/06-stop-services.sh"
print_status "INFO" "Check status: ./scripts/05-check-system-status.sh"
print_status "INFO" "Test workflows: ./scripts/10-test-workflows.sh"
print_status "INFO" "Full cleanup: ./scripts/01-cleanup-environment.sh"
print_status "INFO" "=== NEXT STEPS ==="
print_status "INFO" "1. Access Dashboard UI at http://localhost:3000"
print_status "INFO" "2. Configure gateways and devices via dashboard"
print_status "INFO" "3. Deploy applications using the dashboard interface"
print_status "INFO" "4. Test workflows manually: ./scripts/10-test-workflows.sh"

print_status "SUCCESS" "🎉 Wasmbed Platform is ready for use!"