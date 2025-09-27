#!/bin/bash

# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

set -e

# Wasmbed Platform - Monitoring Script
# This script provides monitoring and observability features

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

show_help() {
    echo "Wasmbed Monitoring Script"
    echo ""
    echo "Usage: $0 [COMMAND] [OPTIONS]"
    echo ""
    echo "Commands:"
    echo "  overview               Show system overview"
    echo "  devices                Show device metrics"
    echo "  applications           Show application metrics"
    echo "  gateways               Show gateway metrics"
    echo "  infrastructure         Show infrastructure metrics"
    echo "  logs                   Show system logs"
    echo "  health                 Check system health"
    echo "  watch                  Watch resources in real-time"
    echo ""
    echo "Examples:"
    echo "  $0 overview"
    echo "  $0 devices"
    echo "  $0 health"
    echo "  $0 watch"
}

show_overview() {
    print_status "INFO" "Wasmbed Platform Overview"
    echo ""
    
    # Cluster status
    print_status "INFO" "Cluster Status:"
    kubectl cluster-info --request-timeout=5s 2>/dev/null || print_status "ERROR" "Cluster not accessible"
    echo ""
    
    # Resource counts
    print_status "INFO" "Resource Counts:"
    DEVICE_COUNT=$(kubectl get devices -n wasmbed --no-headers 2>/dev/null | wc -l || echo "0")
    APP_COUNT=$(kubectl get applications -n wasmbed --no-headers 2>/dev/null | wc -l || echo "0")
    GATEWAY_COUNT=$(kubectl get gateways -n wasmbed --no-headers 2>/dev/null | wc -l || echo "0")
    
    echo "  Devices: $DEVICE_COUNT"
    echo "  Applications: $APP_COUNT"
    echo "  Gateways: $GATEWAY_COUNT"
    echo ""
    
    # Service status
    print_status "INFO" "Service Status:"
    if curl -s "http://localhost:30460/api/v1/status" >/dev/null 2>&1; then
        print_status "SUCCESS" "Infrastructure: Running"
    else
        print_status "ERROR" "Infrastructure: Not responding"
    fi
    
    if curl -s "http://localhost:30451/api/v1/devices" >/dev/null 2>&1; then
        print_status "SUCCESS" "Gateway: Running"
    else
        print_status "ERROR" "Gateway: Not responding"
    fi
    
    if curl -s "http://localhost:30470/api/status" >/dev/null 2>&1; then
        print_status "SUCCESS" "Dashboard: Running"
    else
        print_status "ERROR" "Dashboard: Not responding"
    fi
}

show_device_metrics() {
    print_status "INFO" "Device Metrics"
    echo ""
    
    kubectl get devices -n wasmbed -o custom-columns="NAME:.metadata.name,PHASE:.status.phase,GATEWAY:.status.gateway,CONNECTED:.status.connectedSince,HEARTBEAT:.status.lastHeartbeat" 2>/dev/null || print_status "ERROR" "No devices found"
}

show_application_metrics() {
    print_status "INFO" "Application Metrics"
    echo ""
    
    kubectl get applications -n wasmbed -o custom-columns="NAME:.metadata.name,PHASE:.status.phase,TOTAL:.status.statistics.totalDevices,RUNNING:.status.statistics.runningDevices,FAILED:.status.statistics.failedDevices" 2>/dev/null || print_status "ERROR" "No applications found"
}

show_gateway_metrics() {
    print_status "INFO" "Gateway Metrics"
    echo ""
    
    kubectl get gateways -n wasmbed -o custom-columns="NAME:.metadata.name,PHASE:.status.phase,CONNECTED:.status.connectedDevices,ENROLLED:.status.enrolledDevices,HEARTBEAT:.status.lastHeartbeat" 2>/dev/null || print_status "ERROR" "No gateways found"
}

show_infrastructure_metrics() {
    print_status "INFO" "Infrastructure Metrics"
    echo ""
    
    # Infrastructure API status
    if curl -s "http://localhost:30460/api/v1/status" >/dev/null 2>&1; then
        print_status "SUCCESS" "Infrastructure API: Healthy"
    else
        print_status "ERROR" "Infrastructure API: Not responding"
    fi
    
    # Monitoring service
    if curl -s "http://localhost:9090/metrics" >/dev/null 2>&1; then
        print_status "SUCCESS" "Monitoring Service: Running"
    else
        print_status "WARNING" "Monitoring Service: Not responding"
    fi
    
    # Logging service
    if curl -s "http://localhost:8080/logs" >/dev/null 2>&1; then
        print_status "SUCCESS" "Logging Service: Running"
    else
        print_status "WARNING" "Logging Service: Not responding"
    fi
}

show_logs() {
    print_status "INFO" "System Logs"
    echo ""
    
    # Get recent events
    print_status "INFO" "Recent Events:"
    kubectl get events -n wasmbed --sort-by='.lastTimestamp' | tail -20
}

check_health() {
    print_status "INFO" "System Health Check"
    echo ""
    
    local health_score=0
    local total_checks=0
    
    # Check cluster
    total_checks=$((total_checks + 1))
    if kubectl cluster-info --request-timeout=5s >/dev/null 2>&1; then
        print_status "SUCCESS" "Kubernetes cluster: Healthy"
        health_score=$((health_score + 1))
    else
        print_status "ERROR" "Kubernetes cluster: Unhealthy"
    fi
    
    # Check CRDs
    total_checks=$((total_checks + 1))
    if kubectl get crd devices.wasmbed.github.io >/dev/null 2>&1 && \
       kubectl get crd applications.wasmbed.github.io >/dev/null 2>&1 && \
       kubectl get crd gateways.wasmbed.io >/dev/null 2>&1; then
        print_status "SUCCESS" "Custom Resource Definitions: Healthy"
        health_score=$((health_score + 1))
    else
        print_status "ERROR" "Custom Resource Definitions: Missing"
    fi
    
    # Check services
    total_checks=$((total_checks + 1))
    if curl -s "http://localhost:30460/api/v1/status" >/dev/null 2>&1 && \
       curl -s "http://localhost:30451/api/v1/devices" >/dev/null 2>&1 && \
       curl -s "http://localhost:30470/api/status" >/dev/null 2>&1; then
        print_status "SUCCESS" "Core Services: Healthy"
        health_score=$((health_score + 1))
    else
        print_status "ERROR" "Core Services: Some not responding"
    fi
    
    echo ""
    print_status "INFO" "Health Score: $health_score/$total_checks"
    
    if [ $health_score -eq $total_checks ]; then
        print_status "SUCCESS" "System is fully healthy!"
    elif [ $health_score -ge $((total_checks * 3 / 4)) ]; then
        print_status "WARNING" "System is mostly healthy with minor issues"
    else
        print_status "ERROR" "System has significant health issues"
    fi
}

watch_resources() {
    print_status "INFO" "Watching resources in real-time (Press Ctrl+C to stop)"
    echo ""
    
    # Watch all resources
    kubectl get devices,applications,gateways -n wasmbed -w
}

# Main script logic
case "$1" in
    "overview")
        show_overview
        ;;
    "devices")
        show_device_metrics
        ;;
    "applications")
        show_application_metrics
        ;;
    "gateways")
        show_gateway_metrics
        ;;
    "infrastructure")
        show_infrastructure_metrics
        ;;
    "logs")
        show_logs
        ;;
    "health")
        check_health
        ;;
    "watch")
        watch_resources
        ;;
    "help"|"-h"|"--help")
        show_help
        ;;
    *)
        print_status "ERROR" "Unknown command: $1"
        echo ""
        show_help
        exit 1
        ;;
esac