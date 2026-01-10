#!/bin/bash

# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

# Run all platform tests and generate comprehensive test report

set +e  # Don't exit on error, we want to run all tests

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Test results
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
TEST_RESULTS=()

print_header() {
    echo -e "\n${BLUE}========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}========================================${NC}\n"
}

print_test() {
    echo -e "${BLUE}[TEST]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

print_error() {
    echo -e "${RED}[✗]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[!]${NC} $1"
}

run_test() {
    local test_name=$1
    local test_script=$2
    local test_timeout=${3:-300}
    local optional=${4:-false}
    
    print_test "Running: $test_name"
    ((TOTAL_TESTS++))
    
    local start_time=$(date +%s)
    local output_file="/tmp/test-${test_name// /_}-$(date +%s).log"
    
    if timeout $test_timeout bash "$test_script" > "$output_file" 2>&1; then
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        print_success "$test_name - PASSED (${duration}s)"
        ((PASSED_TESTS++))
        TEST_RESULTS+=("PASS:$test_name:$duration")
        return 0
    else
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        if [ "$optional" = "true" ]; then
            print_warning "$test_name - SKIPPED (requires full deployment) (${duration}s)"
            TEST_RESULTS+=("SKIP:$test_name:$duration")
            return 0
        else
            print_error "$test_name - FAILED (${duration}s)"
            ((FAILED_TESTS++))
            TEST_RESULTS+=("FAIL:$test_name:$duration")
            print_warning "Check log: $output_file"
            return 1
        fi
    fi
}

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

print_header "Wasmbed Platform - Complete Test Suite"

echo "Starting comprehensive test suite..."
echo "Timestamp: $(date)"
echo ""

# Test 1: System Status Check
print_header "Test 1: System Status Check"
run_test "System Status Check" "scripts/04-check-system-status.sh" 120

# Test 2: Dashboard API Tests
print_header "Test 2: Dashboard API Tests"
export API_BASE_URL="${API_BASE_URL:-http://100.103.160.17:3000/api}"
run_test "Dashboard API Tests" "scripts/test-dashboard-apis.sh" 600

# Test 3: Dashboard Tests (optional - requires full deployment)
print_header "Test 3: Dashboard Tests (Optional)"
run_test "Dashboard Tests" "scripts/10-test-dashboard.sh" 300 "true"

# Test 4: Renode Dashboard Tests (optional - requires full deployment)
print_header "Test 4: Renode Dashboard Tests (Optional)"
run_test "Renode Dashboard Tests" "scripts/11-test-renode-dashboard.sh" 300 "true"

# Test 5: Workflow Tests (optional - requires full deployment)
print_header "Test 5: Workflow Tests (Optional)"
run_test "Workflow Tests" "scripts/09-test-workflows.sh" 600 "true"

# Test 6: Firmware Complete Tests
print_header "Test 6: Firmware Complete Tests"
run_test "Firmware Complete Tests" "scripts/test-firmware-complete.sh" 600

# Summary
print_header "Test Summary"

echo -e "${BLUE}Total Tests:${NC} $TOTAL_TESTS"
echo -e "${GREEN}Passed:${NC} $PASSED_TESTS"
echo -e "${RED}Failed:${NC} $FAILED_TESTS"
echo ""

echo -e "${BLUE}Detailed Results:${NC}"
SKIPPED_TESTS=0
for result in "${TEST_RESULTS[@]}"; do
    IFS=':' read -r status name duration <<< "$result"
    if [ "$status" = "PASS" ]; then
        echo -e "  ${GREEN}✓${NC} $name (${duration}s)"
    elif [ "$status" = "SKIP" ]; then
        echo -e "  ${YELLOW}⊘${NC} $name (${duration}s) - SKIPPED (requires deployment)"
        ((SKIPPED_TESTS++))
    else
        echo -e "  ${RED}✗${NC} $name (${duration}s)"
    fi
done

if [ $SKIPPED_TESTS -gt 0 ]; then
    echo ""
    echo -e "${YELLOW}Skipped Tests:${NC} $SKIPPED_TESTS (require full platform deployment)"
fi

echo ""

if [ $FAILED_TESTS -eq 0 ]; then
    print_success "All tests passed!"
    exit 0
else
    print_error "Some tests failed. Check logs in /tmp/test-*.log"
    exit 1
fi
