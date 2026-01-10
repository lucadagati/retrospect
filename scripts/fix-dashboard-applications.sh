#!/bin/bash

# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

# Fix Dashboard Applications Visibility
# Verifica e risolve problemi di visualizzazione applicazioni

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_status() {
    local status=$1
    local message=$2
    case $status in
        "SUCCESS") echo -e "${GREEN}✓${NC} $message" ;;
        "ERROR") echo -e "${RED}✗${NC} $message" ;;
        "WARNING") echo -e "${YELLOW}⚠${NC} $message" ;;
        "INFO") echo -e "${BLUE}ℹ${NC} $message" ;;
    esac
}

print_header() {
    echo ""
    echo "========================================"
    echo "  $1"
    echo "========================================"
}

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

print_header "DASHBOARD APPLICATIONS FIX"

# Check API Server
print_header "STEP 1: VERIFY API SERVER"

if curl -4 -s -f http://localhost:3001/health >/dev/null 2>&1; then
    print_status "SUCCESS" "API Server is running"
else
    print_status "ERROR" "API Server is not running"
    exit 1
fi

# Check Applications in API
print_status "INFO" "Checking applications in API..."
API_APPS=$(curl -s http://localhost:3001/api/v1/applications 2>/dev/null || echo "{}")
APP_COUNT=$(echo "$API_APPS" | jq '.applications | length' 2>/dev/null || echo "0")

if [ "$APP_COUNT" -gt 0 ]; then
    print_status "SUCCESS" "Found $APP_COUNT applications in API"
    echo "$API_APPS" | jq '.applications[] | {name, app_id, id, status}' | head -20
else
    print_status "WARNING" "No applications found in API"
fi

# Check Dashboard
print_header "STEP 2: VERIFY DASHBOARD"

if curl -4 -s -f http://localhost:3000 >/dev/null 2>&1; then
    print_status "SUCCESS" "Dashboard is running"
else
    print_status "WARNING" "Dashboard may not be running"
fi

# Check Dashboard Proxy
print_status "INFO" "Checking dashboard proxy..."
PROXY_APPS=$(curl -s http://localhost:3000/api/v1/applications 2>/dev/null || echo "{}")
PROXY_COUNT=$(echo "$PROXY_APPS" | jq '.applications | length' 2>/dev/null || echo "0")

if [ "$PROXY_COUNT" -gt 0 ]; then
    print_status "SUCCESS" "Dashboard proxy working - Found $PROXY_COUNT applications"
else
    print_status "WARNING" "Dashboard proxy may have issues"
fi

# Check React Dev Server
print_header "STEP 3: CHECK REACT DEV SERVER"

if ps aux | grep -q "[r]eact-scripts start"; then
    print_status "SUCCESS" "React dev server is running"
    print_status "INFO" "Dashboard should auto-reload on file changes"
else
    print_status "WARNING" "React dev server may not be running"
    print_status "INFO" "To start: cd dashboard-react && npm start"
fi

# Recommendations
print_header "RECOMMENDATIONS"

echo ""
print_status "INFO" "To see applications in dashboard:"
echo "  1. Open http://localhost:3000 in browser"
echo "  2. Open DevTools (F12)"
echo "  3. Go to Console tab"
echo "  4. Look for: '=== APPLICATIONS FETCH ==='"
echo "  5. Go to 'Application Management' page"
echo "  6. Check Network tab for API calls"
echo "  7. If still not visible, click 'Refresh' button"
echo ""
print_status "INFO" "If applications still not visible:"
echo "  - Check browser console for errors"
echo "  - Verify API response in Network tab"
echo "  - Try hard refresh: Ctrl+Shift+R (Linux) or Cmd+Shift+R (Mac)"
echo "  - Clear browser cache"
echo "  - Restart dashboard: cd dashboard-react && npm start"

print_header "SUMMARY"

if [ "$APP_COUNT" -gt 0 ]; then
    print_status "SUCCESS" "Applications exist in API ($APP_COUNT)"
    print_status "INFO" "Dashboard should display them after refresh"
else
    print_status "WARNING" "No applications found - create one first"
fi

